pub mod ast;
pub mod compiler;
pub mod program_state;
pub mod scanner;
pub mod treewalk;
pub mod types;
pub mod variables;

pub mod test_libs;

use std::io::Write;

use crate::compiler::Compiler;
use crate::scanner::{Scanner, Token};

pub struct Args {
    pub path: std::path::PathBuf,
    pub prog: String,
    pub n_threads: usize,
}

pub fn driver<T: crate::SyncWrite>(args: &crate::Args, out: &mut T) {
    let scanner = Scanner::new(&args.prog);
    let mut comp = Compiler::new(scanner);
    let prog = match comp.compile(out) {
        Ok(prog) => prog,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let _ = prog.run(args);
}

#[derive(Debug)]
pub enum Error {
    CompileError((String, Token)),
    AttributeInBeginOrEnd,
    IoError(std::io::ErrorKind),
}

impl std::error::Error for Error {}

impl std::convert::From<&std::io::Error> for Error {
    fn from(e: &std::io::Error) -> Self {
        Error::IoError(e.kind())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::CompileError((msg, tok)) => {
                write!(f, "Error: {msg}\nUnexpected token: {:?}", tok)
            }
            Error::AttributeInBeginOrEnd => write!(
                f,
                "Error: attempt to query a file attribute in a BEGIN or END block."
            ),
            Error::IoError(e) => write!(f, "{e}"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

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
