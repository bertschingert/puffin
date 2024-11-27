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
            Some(f) => match self {
                Attribute::Name => {
                    Value::String(f.path.file_name().unwrap().to_string_lossy().to_string())
                }
                Attribute::Path => Value::String(f.path.display().to_string()),
                Attribute::Size => Value::Integer(f.md.size().try_into().unwrap()),
                Attribute::Owner => Value::Integer(f.md.uid().into()),
            },
            None => panic!("Cannot evaluate attribute in BEGIN or END block"),
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
