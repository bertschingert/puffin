use crate::program_state::ProgramState;
use crate::treewalk::*;
use crate::types::*;

pub struct FileState {
    pub path: std::path::PathBuf,
    pub md: std::fs::Metadata,
}
pub struct Program<'a, T: crate::SyncWrite> {
    pub begin: Option<Action>,
    pub end: Option<Action>,
    pub routines: Vec<Routine>,
    pub prog_state: ProgramState<'a, T>,
}

impl<'a, T: crate::SyncWrite> Program<'a, T> {
    pub fn run(&'a self, args: &crate::Args) {
        let path = &args.path;

        let md = std::fs::metadata(path).unwrap();

        self.begin();

        if md.is_dir() {
            let f = FileState {
                path: path.into(),
                md,
            };
            treewalk(args, &self.routines, f, &self.prog_state);
        } else {
            let file_state = FileState {
                path: path.into(),
                md,
            };
            run_routines(&self.routines, &file_state, &self.prog_state);
        }

        self.end();
    }

    fn begin(&self) {
        if let Some(begin) = &self.begin {
            begin.interpret(None, &self.prog_state);
        }
    }

    fn end(&self) {
        if let Some(end) = &self.end {
            end.interpret(None, &self.prog_state);
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

pub fn run_routines<'a, T: crate::SyncWrite>(
    routines: &Vec<Routine>,
    f: &FileState,
    p: &ProgramState<'a, T>,
) {
    for routine in routines.iter() {
        match &routine.cond {
            Some(cond) => {
                if cond.expr.evaluate(Some(f), p).is_truthy() {
                    routine.action.interpret(Some(f), p);
                }
            }
            None => routine.action.interpret(Some(f), p),
        }
    }
}

#[derive(Debug)]
pub struct Condition {
    pub expr: Expression,
}

#[derive(Debug)]
pub enum Statement {
    Assignment(Assignment),
    Print(Expression),
}

impl Statement {
    fn interpret<T: crate::SyncWrite>(&self, f: Option<&FileState>, p: &ProgramState<T>) {
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

    fn interpret<T: crate::SyncWrite>(&self, f: Option<&FileState>, p: &ProgramState<T>) {
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
    fn evaluate<T: crate::SyncWrite>(&self, f: Option<&FileState>, p: &ProgramState<T>) -> Value {
        let l = self.left.evaluate(f, p);
        let r = self.right.evaluate(f, p);

        self.kind.evaluate(l, r)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OpKind {
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
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
            OpKind::Greater => Self::integer_to_boolean_op(l, r, |l, r| l > r),
            OpKind::GreaterEqual => Self::integer_to_boolean_op(l, r, |l, r| l >= r),
            OpKind::Less => Self::integer_to_boolean_op(l, r, |l, r| l < r),
            OpKind::LessEqual => Self::integer_to_boolean_op(l, r, |l, r| l <= r),
            OpKind::Plus => Self::integer_op(l, r, |l, r| l + r),
            OpKind::Minus => Self::integer_op(l, r, |l, r| l - r),
            OpKind::Multiply => Self::integer_op(l, r, |l, r| l * r),
            OpKind::Divide => Self::integer_op(l, r, |l, r| l / r),
        }
    }

    fn integer_to_boolean_op(l: Value, r: Value, f: fn(i64, i64) -> bool) -> Value {
        let l = l.to_integer();
        let r = r.to_integer();
        Value::Boolean(f(l, r))
    }

    fn integer_op(l: Value, r: Value, f: fn(i64, i64) -> i64) -> Value {
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
    fn evaluate<T: crate::SyncWrite>(&self, f: Option<&FileState>, p: &ProgramState<T>) -> Value {
        match self {
            Expression::Bin(op) => op.evaluate(f, p),
            Expression::Attr(attr) => attr.evaluate(f),
            Expression::Atom(v) => v.clone(),
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
                        OpKind::GreaterEqual => ">=",
                        OpKind::Less => "<",
                        OpKind::LessEqual => "<=",
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
