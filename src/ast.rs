use std::os::unix::fs::MetadataExt;

use crate::program_state::ProgramState;

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
    fn evaluate(&self, f: Option<&FileState>) -> Value {
        match f {
            Some(f) => match self {
                Attribute::Size => Value::Integer(f.md.size()),
                Attribute::Owner => Value::Integer(f.md.uid() as u64),
            },
            None => panic!("Cannot evaluate attribute in BEGIN or END block"),
        }
    }
}

pub struct Program<'a, T: std::io::Write> {
    pub begin: Option<Action>,
    pub end: Option<Action>,
    pub routines: Vec<Routine>,
    pub prog_state: ProgramState<'a, T>,
}

impl<'a, T: std::io::Write> Program<'a, T> {
    pub fn run(&mut self, path: &std::path::Path) {
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
    fn treewalk(&mut self, path: &std::path::Path) {
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

    fn begin(&mut self) {
        if let Some(begin) = &self.begin {
            begin.interpret(None, &mut self.prog_state);
        }
    }

    fn end(&mut self) {
        if let Some(end) = &self.end {
            end.interpret(None, &mut self.prog_state);
        }
    }

    fn run_routines(&mut self, f: &FileState) {
        for routine in self.routines.iter() {
            match &routine.cond {
                Some(cond) => {
                    if cond
                        .expr
                        .evaluate(Some(f), &mut self.prog_state)
                        .is_truthy()
                    {
                        routine.action.interpret(Some(f), &mut self.prog_state);
                    }
                }
                None => routine.action.interpret(Some(f), &mut self.prog_state),
            }
        }
    }
}

#[derive(Debug)]
pub struct Routine {
    cond: Option<Condition>,
    action: Action,
}

impl Routine {
    pub fn new(cond: Option<Condition>, action: Action) -> Self {
        Routine { cond, action }
    }
}

#[derive(Debug)]
pub struct Condition {
    pub expr: Expression,
}

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

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{i}"),
            Value::Boolean(b) => match b {
                true => write!(f, "True"),
                false => write!(f, "False"),
            },
        }
    }
}

#[derive(Debug)]
pub enum Statement {
    Assignment(Assignment),
    Print(Expression),
}

impl Statement {
    fn interpret<T: std::io::Write>(&self, f: Option<&FileState>, p: &mut ProgramState<T>) {
        match self {
            Statement::Assignment(a) => {
                p.set_variable(a.id.id, a.val.evaluate(f, p).to_integer());
            }
            Statement::Print(expr) => {
                let _ = p.out.write(format!("{}\n", expr.evaluate(f, p)).as_bytes());
            }
        }
    }
}

#[derive(Debug)]
pub struct Assignment {
    pub id: Identifier,
    pub val: Expression,
}

#[derive(Debug)]
pub struct Identifier {
    /// Index into variables vector.
    pub id: usize,
}

impl Identifier {
    fn evaluate<T: std::io::Write>(&self, p: &ProgramState<T>) -> Value {
        Value::Integer(p.get_variable(self.id))
    }
}

#[derive(Debug)]
pub struct Action {
    pub statements: Option<Vec<Statement>>,
}

impl Action {
    pub fn new(statement: Option<Statement>) -> Self {
        match statement {
            Some(s) => Action {
                statements: Some(vec![s]),
            },
            None => Action { statements: None },
        }
    }

    fn interpret<T: std::io::Write>(&self, f: Option<&FileState>, p: &mut ProgramState<T>) {
        match &self.statements {
            Some(statements) => statements.iter().for_each(|s| s.interpret(f, p)),
            // Default action is to print filename:
            None => {
                match f {
                    Some(f) => {
                        let _ = p.out.write(&format!("{}\n", f.path.display()).as_bytes());
                    }
                    None => {}
                };
            }
        };
    }
}

#[derive(Debug)]
pub struct BinaryOp {
    pub kind: OpKind,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

impl BinaryOp {
    fn evaluate<T: std::io::Write>(&self, f: Option<&FileState>, p: &ProgramState<T>) -> Value {
        let l = self.left.evaluate(f, p);
        let r = self.right.evaluate(f, p);

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
            OpKind::Plus => Self::integer_op(l, r, |l, r| l + r),
            OpKind::Minus => Self::integer_op(l, r, |l, r| l - r),
            OpKind::Multiply => Self::integer_op(l, r, |l, r| l * r),
            OpKind::Divide => Self::integer_op(l, r, |l, r| l / r),
        }
    }

    fn integer_op(l: Value, r: Value, f: fn(u64, u64) -> u64) -> Value {
        let l = l.to_integer();
        let r = r.to_integer();
        Value::Integer(f(l, r))
    }
}

#[derive(Debug)]
pub enum Expression {
    Bin(BinaryOp),
    Attr(Attribute),
    Atom(Value),
    Id(Identifier),
}

impl Expression {
    fn evaluate<T: std::io::Write>(&self, f: Option<&FileState>, p: &ProgramState<T>) -> Value {
        match self {
            Expression::Bin(op) => op.evaluate(f, p),
            Expression::Attr(attr) => attr.evaluate(f),
            Expression::Atom(v) => *v,
            Expression::Id(id) => id.evaluate(p),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Atom(val) => write!(f, "{:?}", val),
            Expression::Attr(attr) => write!(f, "{:?}", attr),
            Expression::Id(id) => write!(f, "{}", id.id),
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
