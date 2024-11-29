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

#[derive(Copy, Clone, Debug)]
pub struct Identifier {
    /// Index into variables vector.
    pub id: usize,
}

impl Identifier {
    /// Evaluate a global variable within the context of the `ProgramState`.
    ///
    /// If this identifier appears on the right-hand side of an assignment to another global
    /// variable, then `with_vars` will contain a reference to the already-unlocked globals vector,
    /// which the Value can be taken directly from.
    fn evaluate<T: crate::SyncWrite>(
        &self,
        p: &ProgramState<T>,
        with_vars: Option<&Vec<i64>>,
    ) -> Value {
        Value::Integer(match with_vars {
            Some(vars) => vars[self.id],
            None => p.get_variable(self.id),
        })
    }
}

#[derive(Debug)]
pub struct ArraySubscript {
    pub id: usize,
    pub subscript: Box<Expression>,
}

#[derive(Debug)]
pub enum Variable {
    Id(Identifier),
    Arr(ArraySubscript),
}

impl Variable {
    pub fn evaluate<T: crate::SyncWrite>(
        &self,
        p: &ProgramState<T>,
        with_vars: Option<&Vec<i64>>,
    ) -> Value {
        match self {
            Variable::Id(id) => id.evaluate(p, with_vars),
            Variable::Arr(_arr) => todo!(),
        }
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
