use crate::ast::{run_routines, FileState, Routine};
use crate::program_state::ProgramState;

pub fn treewalk<'a, T: std::io::Write>(
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
