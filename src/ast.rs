use crate::Attribute;
use crate::FileState;
use crate::Value;

pub struct Program {
    pub begin: Option<Action>,
    pub end: Option<Action>,
    pub routines: Vec<Routine>,
}

impl Program {
    pub fn run(&self, path: &str) {
        let md = std::fs::metadata(&path).unwrap();

        self.begin();

        if md.is_dir() {
            self.treewalk(path);
        } else {
            let file_state = FileState {
                path: path.into(),
                md,
            };
            self.run_routines(&file_state);
        }

        self.end();
    }

    // TODO: pull into own module
    fn treewalk(&self, path: &str) {
        let mut stack: Vec<std::path::PathBuf> = Vec::new();
        stack.push(path.into());

        while let Some(path) = stack.pop() {
            for ent in std::fs::read_dir(path).unwrap() {
                let Ok(ent) = ent else {
                    continue;
                };

                match ent.file_name().to_str() {
                    Some(".") => continue,
                    Some("..") => continue,
                    _ => {}
                };

                let Ok(ty) = ent.file_type() else {
                    continue;
                };

                if ty.is_dir() {
                    stack.push(ent.path());
                }

                let Ok(md) = ent.metadata() else {
                    continue;
                };

                let f = FileState {
                    path: ent.path(),
                    md,
                };
                self.run_routines(&f);
            }
        }
    }

    fn begin(&self) {
        if let Some(begin) = &self.begin {
            begin.interpret(None);
        }
    }

    fn end(&self) {
        if let Some(end) = &self.end {
            end.interpret(None);
        }
    }

    pub(crate) fn run_routines(&self, f: &FileState) {
        for routine in self.routines.iter() {
            match &routine.cond {
                Some(cond) => {
                    if cond.expr.evaluate(f).is_truthy() {
                        routine.action.interpret(Some(f));
                    }
                }
                None => routine.action.interpret(Some(f)),
            }
        }
    }
}

pub struct Routine {
    cond: Option<Condition>,
    action: Action,
}

impl Routine {
    pub fn new(cond: Option<Condition>, action: Action) -> Self {
        Routine { cond, action }
    }
}

pub struct Condition {
    pub expr: Expression,
}

pub struct Action {}

impl Action {
    pub fn new() -> Self {
        Action {}
    }

    fn interpret(&self, f: Option<&FileState>) {
        match f {
            Some(f) => println!("{}", f.path.display()),
            None => println!("BEGIN or END action"),
        };
    }
}

pub enum Expression {
    Bin(BinaryOp),
    Attr(Attribute),
    Atom(Value),
}

pub struct BinaryOp {
    pub kind: OpKind,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

impl BinaryOp {
    fn evaluate(&self, f: &FileState) -> Value {
        let l = self.left.evaluate(f);
        let r = self.right.evaluate(f);

        self.kind.evaluate(l, r)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OpKind {
    EqualEqual,
    Greater,
    Plus,
    Minus,
    Multiply,
    Divide,
}

impl OpKind {
    fn evaluate(&self, l: Value, r: Value) -> Value {
        match self {
            OpKind::EqualEqual => {
                if l == r {
                    Value::Boolean(true)
                } else {
                    Value::Boolean(false)
                }
            }
            OpKind::Greater => {
                let l = l.to_integer();
                let r = r.to_integer();
                if l > r {
                    Value::Boolean(true)
                } else {
                    Value::Boolean(false)
                }
            }
            OpKind::Plus => {
                let l = l.to_integer();
                let r = r.to_integer();
                Value::Integer(l + r)
            }
            OpKind::Minus => {
                let l = l.to_integer();
                let r = r.to_integer();
                Value::Integer(l - r)
            }
            OpKind::Multiply => {
                let l = l.to_integer();
                let r = r.to_integer();
                Value::Integer(l * r)
            }
            OpKind::Divide => {
                let l = l.to_integer();
                let r = r.to_integer();
                Value::Integer(l / r)
            }
        }
    }
}

impl Expression {
    fn evaluate(&self, f: &FileState) -> Value {
        match self {
            Expression::Bin(op) => op.evaluate(f),
            Expression::Attr(attr) => attr.evaluate(f),
            Expression::Atom(v) => *v,
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Atom(val) => write!(f, "{:?}", val),
            Expression::Attr(attr) => write!(f, "{:?}", attr),
            Expression::Bin(op) => {
                write!(
                    f,
                    "({} ",
                    match op.kind {
                        OpKind::EqualEqual => "==",
                        OpKind::Greater => ">",
                        OpKind::Plus => "+",
                        OpKind::Minus => "-",
                        OpKind::Multiply => "*",
                        OpKind::Divide => "/",
                    }
                )?;
                write!(f, "{} ", op.left)?;
                write!(f, "{} ", op.right)?;
                write!(f, ")")
            }
        }
    }
}
