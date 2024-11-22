pub mod ast;
pub mod compiler;
pub mod program_state;
pub mod scanner;

use crate::compiler::Compiler;
use crate::scanner::Scanner;

pub fn driver<T: std::io::Write>(path: &str, prog: &str, out: &mut T) {
    let scanner = Scanner::new(&prog);
    let mut comp = Compiler::new(scanner);
    let mut prog = comp.compile(out);

    prog.run(&path);
}