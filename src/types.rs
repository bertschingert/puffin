use std::os::unix::fs::MetadataExt;

use crate::ast::*;

#[derive(Eq, Hash, Clone, PartialEq, Debug)]
pub enum Value {
    UInt(u64),
    Int(i64),
    String(String),
    Boolean(bool),
    Special(SpecialValue),
}

/// This is for types that are represented as integers, but are distinct types that should not
/// be treated as integers semantically because arithmetic operations do not make sense for them.
#[derive(Eq, Hash, Clone, PartialEq, Debug)]
pub enum SpecialValue {
    /// Inode numbers
    Ino(u64),

    /// Mode
    Mode(u32),

    /// Both UIDs and GIDs
    Uid(u32),

    /// Device ID:
    Devno(u64),
}

impl Value {
    pub fn is_truthy(self) -> bool {
        match self {
            Value::UInt(i) => i != 0,
            Value::Int(i) => i != 0,
            Value::String(s) => s != "",
            Value::Boolean(b) => b,
            // Don't allow trying to interpret an inode number, UID, etc. as a bool:
            Value::Special(s) => crate::runtime_error(&format!("Cannot evaluate a special value '{:?}' as a boolean", s)),
        }
    }

    pub fn to_signed_int(self) -> i64 {
        match self {
            Value::UInt(_) => panic!("Should I allow this?"),
            Value::Int(i) => i,
            Value::String(s) => s.parse::<i64>().unwrap_or(0),
            Value::Boolean(b) => {
                if b {
                    1
                } else {
                    0
                }
            }
            // Don't allow trying to interpret an inode number, UID, etc. as an int:
            Value::Special(s) => crate::runtime_error(&format!("Cannot evaluate a special value '{:?}' as an integer", s)),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::UInt(u) => write!(f, "{u}"),
            Value::Int(i) => write!(f, "{i}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Boolean(b) => match b {
                true => write!(f, "True"),
                false => write!(f, "False"),
            },
            Value::Special(s) => write!(f, "{s}"),
        }
    }
}

impl std::fmt::Display for SpecialValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecialValue::Ino(i) => write!(f, "{i}"),
            SpecialValue::Mode(m) => write!(f, "{m:#o}"),
            SpecialValue::Uid(u) => write!(f, "{u}"),
            SpecialValue::Devno(d) => write!(f, "{d}"),
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
            Attribute::Size => Value::Int(md.size().try_into().unwrap()),
            Attribute::Owner => Value::Int(md.uid().into()),
            Attribute::Name => unreachable!(),
            Attribute::Path => unreachable!(),
        })
    }
}
