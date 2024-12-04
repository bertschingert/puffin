use crossbeam::deque::{Steal, Stealer, Worker};
use std::path::{Path, PathBuf};

use crate::ast::{run_routines, FileState, Routine};
use crate::program_state::ProgramState;
use crate::Args;

pub fn treewalk<'a, 'b, T: crate::SyncWrite>(
    args: &Args,
    routines: &Vec<Routine>,
    f: FileState,
    p: &ProgramState<'a, 'b, T>,
) -> Result<(), crate::RuntimeError> {
    // If there was an error running the program on the root, there is no point in continuing.
    // Either an I/O to the root failed (maybe because it no longer exists), or there was a
    // runtime error in the program on the root, and the policy for runtime errors is to stop
    // program operation.
    run_routines(routines, &f, p)?;

    match args.n_threads {
        1 => treewalk_single_threaded(routines, f, p),
        _ => treewalk_multi_threaded(args, routines, f, p),
    }
}

fn treewalk_single_threaded<'a, 'b, T: crate::SyncWrite>(
    routines: &Vec<Routine>,
    f: FileState,
    p: &ProgramState<'a, 'b, T>,
) -> Result<(), crate::RuntimeError> {
    let mut stack: Vec<std::path::PathBuf> = Vec::new();
    stack.push(f.path);

    while let Some(path) = stack.pop() {
        // XXX: flatten() instead of unwrap()?
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

            let f = FileState::new(ent.path(), None);
            let _ = run_routines(routines, &f, p)?;
        }
    }

    Ok(())
}

struct State<'a, 'p1, 'p2, T: crate::SyncWrite> {
    n_workers: usize,
    stealers: &'a [Stealer<PathBuf>],
    routines: &'p1 Vec<Routine>,
    prog_state: &'p1 ProgramState<'p1, 'p2, T>,
}

fn treewalk_multi_threaded<'p1, 'p2, T: crate::SyncWrite>(
    args: &Args,
    routines: &'p1 Vec<Routine>,
    f: FileState,
    p: &'p1 ProgramState<'p1, 'p2, T>,
) -> Result<(), crate::RuntimeError> {
    let mut workers: Vec<Worker<PathBuf>> = Vec::new();
    let mut stealers: Vec<Stealer<PathBuf>> = Vec::new();

    for _ in 0..args.n_threads {
        let worker = Worker::new_fifo();
        stealers.push(worker.stealer());
        workers.push(worker);
    }

    let state = State {
        n_workers: args.n_threads,
        stealers: &stealers,
        routines,
        prog_state: p,
    };

    workers[0].push(f.path);

    std::thread::scope(|s| {
        (0..args.n_threads)
            .map(|_| {
                let worker = workers.pop().unwrap();
                let state = &state;
                s.spawn(move || worker_main(&worker, state))
            })
            .map(|t| t.join().unwrap())
            // If any of the threads had an error, return the first error, otherwise, Ok():
            .find(|r| r.is_err())
            .unwrap_or(Ok(()))
    })
}

fn worker_main<T: crate::SyncWrite>(
    w: &Worker<PathBuf>,
    state: &State<T>,
) -> Result<(), crate::RuntimeError> {
    loop {
        if state.prog_state.check_runtime_error() {
            break;
        };

        match find_task(w, state) {
            Some(path) => process_directory(&path, w, state)
                .inspect_err(|e| state.prog_state.set_runtime_error(e.clone()))?,
            // TODO: proper termination detecton.
            None => break,
        };
    }

    Ok(())
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

fn process_directory<T: crate::SyncWrite>(
    path: &Path,
    w: &Worker<PathBuf>,
    state: &State<T>,
) -> Result<(), crate::RuntimeError> {
    let Ok(dir) = std::fs::read_dir(path) else {
        return Ok(());
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

        let f = FileState::new(ent.path(), None);

        run_routines(state.routines, &f, state.prog_state)?;

        let Ok(ty) = ent.file_type() else {
            continue;
        };

        if ty.is_dir() {
            w.push(ent.path());
        }
    }

    Ok(())
}
