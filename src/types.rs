use std::os::unix::fs::MetadataExt;

use crate::ast::*;
use crate::program_state::ProgramState;

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Integer(i64),
    String(String),
    Boolean(bool),
}

impl Value {
    pub fn is_truthy(self) -> bool {
        match self {
            Value::Integer(i) => i != 0,
            Value::String(s) => s != "",
            Value::Boolean(b) => b,
        }
    }

    pub fn to_integer(self) -> i64 {
        match self {
            Value::Integer(i) => i,
            Value::String(s) => s.parse::<i64>().unwrap_or(0),
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

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{i}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Boolean(b) => match b {
                true => write!(f, "True"),
                false => write!(f, "False"),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Attribute {
    Name,
    Path,
    Size,
    Owner,
}

impl Attribute {
    pub fn evaluate(&self, f: Option<&FileState>) -> Value {
        match f {
            Some(f) => self.evaluate_with_file(f),
            None => panic!("Cannot evaluate attribute in BEGIN or END block"),
        }
    }

    fn evaluate_with_file(&self, f: &FileState) -> Value {
        match self {
            Attribute::Name => {
                Value::String(f.path.file_name().unwrap().to_string_lossy().to_string())
            }
            Attribute::Path => Value::String(f.path.display().to_string()),
            _ => self.evaluate_needs_stat(f),
        }
    }

    fn evaluate_needs_stat(&self, f: &FileState) -> Value {
        let md = match f.get_metadata().as_ref() {
            Ok(md) => md,
            // TODO: just return an error type here
            Err(e) => panic!("could not stat {:?}: {e}", f.path.display()),
        };

        match self {
            Attribute::Size => Value::Integer(md.size().try_into().unwrap()),
            Attribute::Owner => Value::Integer(md.uid().into()),
            Attribute::Name => unreachable!(),
            Attribute::Path => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Identifier {
    /// Index into variables vector.
    pub id: usize,
}

impl Identifier {
    pub fn evaluate<T: crate::SyncWrite>(&self, p: &ProgramState<T>) -> Value {
        Value::Integer(p.get_variable(self.id))
    }
}
