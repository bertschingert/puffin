use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::FileTypeExt;

use crate::ast::*;

#[derive(Eq, Hash, Clone, PartialEq, Debug)]
pub enum Value {
    // TODO: probably need an unsigned int type, along with good rules on how/when to convert
    // between them
    Int(i64),
    String(String),
    Boolean(bool),
    Special(SpecialValue),
}

/// This is for types that are represented as integers, but are distinct types that should not
/// be treated as integers semantically because arithmetic operations do not make sense for them.
#[derive(Eq, Hash, Clone, PartialEq, Debug)]
pub struct SpecialValue {
    val: u64,
    kind: SpecialValueKind,
}

#[derive(Eq, Hash, Clone, PartialEq, Debug)]
pub enum SpecialValueKind {
    Ino,
    Mode,
    /// Both UIDs and GIDs
    Uid,
    Devno,
}

impl Value {
    pub fn is_truthy(self) -> bool {
        match self {
            Value::Int(i) => i != 0,
            Value::String(s) => s != "",
            Value::Boolean(b) => b,
            // Don't allow trying to interpret an inode number, UID, etc. as a bool:
            // XXX: can this be made a compile time error?
            Value::Special(s) => crate::runtime_error(&format!(
                "Cannot evaluate a special value '{:?}' as a boolean",
                s
            )),
        }
    }

    pub fn to_signed_int(self) -> i64 {
        match self {
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
            Value::Special(s) => crate::runtime_error(&format!(
                "Cannot evaluate a special value '{:?}' as an integer",
                s
            )),
        }
    }

    pub fn binary_op(self, other: Value, op: OpKind) -> Value {
        match op {
            OpKind::Plus => Self::integer_op(self, other, |l, r| l + r),
            OpKind::Minus => Self::integer_op(self, other, |l, r| l - r),
            OpKind::Multiply => Self::integer_op(self, other, |l, r| l * r),
            OpKind::Divide => Self::integer_op(self, other, |l, r| l / r),
            OpKind::Greater => Self::int_to_bool_op(self, other, |l, r| l > r),
            OpKind::GreaterEqual => Self::int_to_bool_op(self, other, |l, r| l >= r),
            OpKind::Less => Self::int_to_bool_op(self, other, |l, r| l < r),
            OpKind::LessEqual => Self::int_to_bool_op(self, other, |l, r| l <= r),
            op => Self::equality(op, self, other),
        }
    }

    fn int_to_bool_op(l: Value, r: Value, f: fn(i64, i64) -> bool) -> Value {
        let l = l.to_signed_int();
        let r = r.to_signed_int();
        Value::Boolean(f(l, r))
    }

    fn integer_op(l: Value, r: Value, f: fn(i64, i64) -> i64) -> Value {
        let l = l.to_signed_int();
        let r = r.to_signed_int();
        Value::Int(f(l, r))
    }

    fn equality(op: OpKind, val1: Value, val2: Value) -> Value {
        if let Value::Special(s) = val1 {
            return s.binary_op(op, val2);
        }

        if let Value::Special(s) = val2 {
            return s.binary_op(op, val1);
        }

        match op {
            OpKind::EqualEqual => {
                if val1 == val2 {
                    Value::Boolean(true)
                } else {
                    Value::Boolean(false)
                }
            }
            _ => todo!(),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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

impl SpecialValue {
    fn new(val: u64, kind: SpecialValueKind) -> Value {
        Value::Special(SpecialValue { val, kind })
    }

    fn binary_op(self, op: OpKind, other: Value) -> Value {
        match op {
            OpKind::EqualEqual => self.equality(other),
            // XXX: impl display for OpKind for better error message?
            op => {
                crate::runtime_error(&format!("Cannot apply operator '{:?}' to '{:?}'", op, self))
            }
        }
    }

    fn equality(self, other: Value) -> Value {
        match other {
            // Any special value can be compared for equality with an unsigned int:
            Value::Int(v) => {
                let v: u64 = match v.try_into() {
                    Ok(v) => v,
                    Err(_) => crate::runtime_error(&format!(
                        "Cannot compare '{:?}' to signed integer '{v}'",
                        self
                    )),
                };
                if self.val == v {
                    Value::Boolean(true)
                } else {
                    Value::Boolean(false)
                }
            }
            // Special values can be compared for equality with other special values of the
            // same type only:
            Value::Special(ref s) => {
                if self.kind == s.kind {
                    if self.val == s.val {
                        Value::Boolean(true)
                    } else {
                        Value::Boolean(false)
                    }
                } else {
                    crate::runtime_error(&format!("Cannot compare '{:?}' to '{:?}'", self, other))
                }
            }
            other => crate::runtime_error(&format!("Cannot compare '{:?}' to '{:?}'", self, other)),
        }
    }
}

impl std::fmt::Display for SpecialValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            SpecialValueKind::Mode => write!(f, "{:#o}", self.val),
            _ => write!(f, "{}", self.val),
        }
    }
}

/// Attributes are file metadata.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Attribute {
    BlkSize,
    Blocks,
    /// Device number for the devile the file resides on
    Dev,
    /// Inode number
    Ino,
    Mode,
    /// Filename
    Name,
    NLink,
    Owner,
    Group,
    /// Full path
    Path,
    /// Device number of the file itself (special files)
    RDev,
    Size,
    Atime,
    Mtime,
    Ctime,
    // XXX: include birthtime?
    Type,
}

impl Attribute {
    pub fn from_str(a: &str) -> Option<Self> {
        Some(match a {
            ".blksize" => Attribute::BlkSize,
            ".blocks" => Attribute::Blocks,
            ".dev" => Attribute::Dev,
            ".ino" => Attribute::Ino,
            ".mode" => Attribute::Mode,
            ".name" => Attribute::Name,
            ".nlink" => Attribute::NLink,
            ".owner" => Attribute::Owner,
            ".group" => Attribute::Group,
            ".path" => Attribute::Path,
            ".rdev" => Attribute::RDev,
            ".size" => Attribute::Size,
            ".atime" => Attribute::Atime,
            ".mtime" => Attribute::Mtime,
            ".ctime" => Attribute::Ctime,
            ".type" => Attribute::Type,
            _ => return None,
        })
    }

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
            Attribute::BlkSize => Value::Int(md.blksize().try_into().unwrap()),
            Attribute::Blocks => Value::Int(md.blocks().try_into().unwrap()),
            Attribute::Ino => SpecialValue::new(md.ino(), SpecialValueKind::Ino),
            Attribute::Dev => SpecialValue::new(md.dev(), SpecialValueKind::Devno),
            Attribute::RDev => SpecialValue::new(md.rdev(), SpecialValueKind::Devno),
            Attribute::Mode => SpecialValue::new(md.mode().into(), SpecialValueKind::Mode),
            Attribute::Size => Value::Int(md.size().try_into().unwrap()),
            Attribute::NLink => Value::Int(md.nlink().try_into().unwrap()),
            Attribute::Owner => SpecialValue::new(md.uid().into(), SpecialValueKind::Uid),
            Attribute::Group => SpecialValue::new(md.gid().into(), SpecialValueKind::Uid),
            Attribute::Atime => Value::Int(md.atime()),
            Attribute::Ctime => Value::Int(md.ctime()),
            Attribute::Mtime => Value::Int(md.mtime()),
            Attribute::Type => {
                let ty = md.file_type();
                Value::String(
                if ty.is_dir() {
                    "dir"
                } else if ty.is_file() {
                    "file"
                } else if ty.is_block_device() {
                    "block"
                } else if ty.is_char_device() {
                    "char"
                } else if ty.is_fifo() {
                    "fifo"
                } else if ty.is_socket() {
                    "socket"
                } else {
                    "unknown"
                }.to_string()
                )
            }
            Attribute::Name => unreachable!(),
            Attribute::Path => unreachable!(),
        })
    }
}
