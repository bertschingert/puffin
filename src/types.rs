use std::os::unix::fs::MetadataExt;

use crate::ast::*;
use crate::program_state::VariableState;

#[derive(Eq, Hash, Clone, PartialEq, Debug)]
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

/// Attributes are file metadata.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Attribute {
    Name,
    Path,
    Size,
    Owner,
}

impl Attribute {
    pub fn evaluate(&self, f: Option<&FileState>) -> crate::Result<Value> {
        match f {
            Some(f) => self.evaluate_with_file(f),
            None => Err(crate::Error::AttributeInBeginOrEnd),
        }
    }

    fn evaluate_with_file(&self, f: &FileState) -> crate::Result<Value> {
        Ok(match self {
            Attribute::Name => Value::String(match f.path.file_name() {
                Some(s) => s.to_string_lossy().to_string(),
                None => f.path.display().to_string(),
            }),
            Attribute::Path => Value::String(f.path.display().to_string()),
            _ => self.evaluate_needs_stat(f)?,
        })
    }

    fn evaluate_needs_stat(&self, f: &FileState) -> crate::Result<Value> {
        let md = f.get_metadata().as_ref()?;

        Ok(match self {
            Attribute::Size => Value::Integer(md.size().try_into().unwrap()),
            Attribute::Owner => Value::Integer(md.uid().into()),
            Attribute::Name => unreachable!(),
            Attribute::Path => unreachable!(),
        })
    }
}

/// A Variable can be either a simple identifier, or an array name together with a subscript.
#[derive(Debug)]
pub enum Variable {
    Id(Identifier),
    Arr(ArraySubscript),
}

impl Variable {
    pub fn evaluate(&self, f: Option<&FileState>, vars: &VariableState) -> crate::Result<Value> {
        vars.get_variable(f, &self)
    }
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variable::Id(id) => write!(f, "Var({})", id.id),
            Variable::Arr(arr) => write!(f, "Var({})[{}]", arr.id, arr.subscript),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Identifier {
    /// Index into variables vector.
    pub id: usize,
}

#[derive(Debug)]
pub struct ArraySubscript {
    pub id: usize,
    pub subscript: Box<Expression>,
}
