pub mod ast;
pub mod compiler;
pub mod program_state;
pub mod scanner;
pub mod treewalk;
pub mod types;
pub mod variables;

pub mod test_libs;

pub use errors::*;

use std::io::Write;

use crate::compiler::Compiler;
use crate::scanner::{Scanner, Token};

pub struct Args {
    pub path: std::path::PathBuf,
    pub prog: String,
    pub n_threads: usize,
}

pub fn driver<T: crate::SyncWrite>(args: &crate::Args, out: &mut T) -> Result<()> {
    let scanner = Scanner::new(&args.prog);
    let mut comp = Compiler::new(scanner);
    let prog = comp.compile(out).inspect_err(|e| {
        eprintln!("{e}");
    })?;

    Ok(prog.run(args)?)
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

/// Error types used by puffin.
mod errors {
    use crate::Token;

    #[derive(Debug)]
    pub enum Error {
        CompileError((String, Token)),
        // XXX: make this a RuntimeError, not a unique type?
        AttributeInBeginOrEnd,
        IoError(std::io::ErrorKind),
        Runtime(RuntimeError),
    }

    impl std::error::Error for Error {}

    impl std::convert::From<&std::io::Error> for Error {
        fn from(e: &std::io::Error) -> Self {
            Error::IoError(e.kind())
        }
    }

    impl std::convert::From<RuntimeError> for Error {
        fn from(r: RuntimeError) -> Self {
            Error::Runtime(r)
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
                Error::Runtime(r) => write!(f, "{r}"),
            }
        }
    }

    pub fn filter_non_fatal_errors(
        res: std::result::Result<(), Error>,
    ) -> std::result::Result<(), RuntimeError> {
        match res {
            Err(Error::Runtime(r)) => Err(r),
            _ => Ok(()),
        }
    }

    #[derive(Clone, Debug)]
    pub struct RuntimeError {
        msg: String,
    }

    impl std::error::Error for RuntimeError {}

    impl RuntimeError {
        pub fn new(msg: &str) -> Self {
            RuntimeError {
                msg: msg.to_string(),
            }
        }
    }

    impl std::fmt::Display for RuntimeError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "Runtime error: {}", self.msg)
        }
    }

    pub type Result<T> = std::result::Result<T, Error>;
}
