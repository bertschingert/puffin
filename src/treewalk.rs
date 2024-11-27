use crossbeam::deque::{Steal, Stealer, Worker};
use std::path::{Path, PathBuf};

use crate::ast::{run_routines, FileState, Routine};
use crate::program_state::ProgramState;

pub fn treewalk<'a, T: crate::SyncWrite>(
    routines: &Vec<Routine>,
    f: FileState,
    p: &mut ProgramState<'a, T>,
) {
    run_routines(routines, &f, p);

    let mut stack: Vec<std::path::PathBuf> = Vec::new();
    stack.push(f.path);

    while let Some(path) = stack.pop() {
        for ent in std::fs::read_dir(path).unwrap() {
            let Ok(ent) = ent else {
                continue;
            };

            match ent.file_name().to_str() {
                Some(".") => continue,
                Some("..") => continue,
                _ => {}
            };

            let Ok(ty) = ent.file_type() else {
                continue;
            };

            if ty.is_dir() {
                stack.push(ent.path());
            }

            let Ok(md) = ent.metadata() else {
                continue;
            };

            let f = FileState {
                path: ent.path(),
                md,
            };
            run_routines(routines, &f, p);
        }
    }
}

struct State<'a, 'p, T: crate::SyncWrite> {
    n_workers: usize,
    stealers: &'a [Stealer<PathBuf>],
    routines: &'p Vec<Routine>,
    prog_state: &'p ProgramState<'p, T>,
}

pub fn treewalk_concurrent<'p, T: crate::SyncWrite>(
    routines: &'p Vec<Routine>,
    f: FileState,
    p: &'p ProgramState<'p, T>,
) {
    let mut workers: Vec<Worker<PathBuf>> = Vec::new();
    let mut stealers: Vec<Stealer<PathBuf>> = Vec::new();

    // TODO: command line argument for number of threads
    for _ in 0..4 {
        let worker = Worker::new_fifo();
        stealers.push(worker.stealer());
        workers.push(worker);
    }

    let state = State {
        n_workers: 4,
        stealers: &stealers,
        routines,
        prog_state: p,
    };

    workers[0].push(f.path);

    std::thread::scope(|s| {
        for _ in 0..4 {
            let worker = workers.pop().unwrap();
            let state = &state;
            s.spawn(move || {
                worker_main(&worker, state);
            });
        }
    });
}

fn worker_main<T: crate::SyncWrite>(w: &Worker<PathBuf>, state: &State<T>) {
    loop {
        match find_task(w, state) {
            Some(path) => process_directory(&path, w, state),
            // TODO: proper termination detecton.
            None => break,
        };
    }
}

fn find_task<T: crate::SyncWrite>(local: &Worker<PathBuf>, state: &State<T>) -> Option<PathBuf> {
    if let Some(task) = local.pop() {
        return Some(task);
    }

    for i in 0..state.n_workers {
        match state.stealers[i].steal() {
            Steal::Success(task) => {
                return Some(task);
            }
            _ => {}
        };
    }

    None
}

fn process_directory<T: crate::SyncWrite>(path: &Path, w: &Worker<PathBuf>, state: &State<T>) {
    let Ok(dir) = std::fs::read_dir(path) else {
        return;
    };

    for ent in dir {
        let Ok(ent) = ent else {
            continue;
        };

        match ent.file_name().to_str() {
            Some(".") => continue,
            Some("..") => continue,
            _ => {}
        };

        let Ok(md) = ent.metadata() else {
            continue;
        };

        let f = FileState {
            path: ent.path(),
            md,
        };

        run_routines(state.routines, &f, state.prog_state);

        let Ok(ty) = ent.file_type() else {
            continue;
        };

        if ty.is_dir() {
            w.push(ent.path());
        }
    }
}