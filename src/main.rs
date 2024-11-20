pub mod ast;
pub mod compiler;
pub mod scanner;

use crate::compiler::Compiler;
use crate::scanner::Scanner;

fn main() {
    let mut args = std::env::args();
    let path = args.nth(1).unwrap();

    let prog = args.nth(0).unwrap();
    let scanner = Scanner::new(&prog);
    let mut comp = Compiler::new(scanner);
    let prog = comp.compile();

    prog.run(&path);
}
