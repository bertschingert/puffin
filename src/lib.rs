pub mod ast;
pub mod compiler;
pub mod program_state;
pub mod scanner;
pub mod treewalk;
pub mod types;

pub mod test_libs;

use std::io::Write;

use crate::compiler::Compiler;
use crate::scanner::Scanner;

pub fn driver<T: crate::SyncWrite>(path: &std::path::Path, prog: &str, out: &mut T) {
    let scanner = Scanner::new(&prog);
    let mut comp = Compiler::new(scanner);
    let prog = comp.compile(out);

    prog.run(&path);
}

// Like std::io::Write but it requires that the writer be Sync.
// Assumes that the type implementing SyncWrite uses interior mutability
// so that write() doesn't require a mutable reference.
pub trait SyncWrite: Sync {
    fn write(&self, buf: &[u8]) -> std::io::Result<usize>;
}

impl SyncWrite for std::io::Stdout {
    fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.lock().write(buf)
    }
}
