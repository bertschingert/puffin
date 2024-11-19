use std::os::unix::fs::MetadataExt;

pub mod ast;
pub mod compiler;
pub mod scanner;

use crate::compiler::Compiler;
use crate::scanner::Scanner;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Value {
    Integer(u64),
    Boolean(bool),
}

impl Value {
    fn is_truthy(self) -> bool {
        match self {
            Value::Integer(i) => i != 0,
            Value::Boolean(b) => b,
        }
    }

    fn to_integer(self) -> u64 {
        match self {
            Value::Integer(i) => i,
            Value::Boolean(b) => {
                if b {
                    1
                } else {
                    0
                }
            }
        }
    }
}

struct FileState {
    path: std::path::PathBuf,
    md: std::fs::Metadata,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Attribute {
    Size,
    Owner,
}

impl Attribute {
    fn evaluate(&self, f: &FileState) -> Value {
        match self {
            Attribute::Size => Value::Integer(f.md.size()),
            Attribute::Owner => Value::Integer(f.md.uid() as u64),
        }
    }
}

fn main() {
    let mut args = std::env::args();
    let path = args.nth(1).unwrap();

    let prog = args.nth(0).unwrap();
    let scanner = Scanner::new(&prog);
    let mut comp = Compiler::new(scanner);
    let prog = comp.compile();

    prog.run(&path);
}
