use std::cell::OnceCell;

use crate::filter_non_fatal_errors;
use crate::program_state::ProgramState;
use crate::treewalk::*;
use crate::types::*;
use crate::variables::*;

pub mod analysis;

pub struct FileState {
    pub path: std::path::PathBuf,
    /// A file's metadata is checked lazily, so that the extra stat() syscall can be avoided if
    /// the metadata is never queried.
    md: OnceCell<std::io::Result<std::fs::Metadata>>,
}

impl FileState {
    /// Construct a new FileState. If the metadata is already available, pass Some(md) to set it,
    /// otherwise, None means it will be queried from the filesystem later if needed.
    pub fn new(path: std::path::PathBuf, md: Option<std::fs::Metadata>) -> Self {
        let md_cell = OnceCell::new();
        match md {
            Some(md) => md_cell.set(Ok(md)).unwrap(),
            None => {}
        };

        FileState { path, md: md_cell }
    }

    pub fn get_metadata(&self) -> &Result<std::fs::Metadata, std::io::Error> {
        self.md.get_or_init(|| std::fs::metadata(&self.path))
    }
}

pub struct Program<'a, 'b, T: crate::SyncWrite> {
    pub begin: Option<Action>,
    pub end: Option<Action>,
    pub routines: Vec<Routine>,
    pub prog_state: ProgramState<'a, 'b, T>,
}

impl<'a, 'b, T: crate::SyncWrite> Program<'a, 'b, T> {
    pub fn run(&'a self, args: &crate::Args) -> Result<(), crate::RuntimeError> {
        let path = &args.path;

        let md = std::fs::metadata(path).unwrap();

        self.begin_or_end(&self.begin)?;

        if md.is_dir() {
            let f = FileState::new(path.into(), Some(md));
            treewalk(args, &self.routines, f, &self.prog_state)?;
        } else {
            let f = FileState::new(path.into(), Some(md));
            run_routines(&self.routines, &f, &self.prog_state)?;
        }

        self.begin_or_end(&self.end)
    }

    fn begin_or_end(&self, action: &Option<Action>) -> Result<(), crate::RuntimeError> {
        filter_non_fatal_errors(if let Some(action) = action {
            action
                .interpret(None, &self.prog_state)
                .inspect_err(|e| eprintln!("{e}"))
        } else {
            Ok(())
        })
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

pub fn run_routines<'a, 'b, T: crate::SyncWrite>(
    routines: &Vec<Routine>,
    f: &FileState,
    p: &ProgramState<'a, 'b, T>,
) -> Result<(), crate::RuntimeError> {
    filter_non_fatal_errors(
        run_routines_inner(routines, f, p)
            .inspect_err(|e| eprintln!("Could not run program on {:?}: {e}", f.path.display())),
    )
}

fn run_routines_inner<'a, 'b, T: crate::SyncWrite>(
    routines: &Vec<Routine>,
    f: &FileState,
    p: &ProgramState<'a, 'b, T>,
) -> crate::Result<()> {
    for routine in routines.iter() {
        match &routine.cond {
            Some(cond) => {
                if cond.expr.evaluate(Some(f), p.vars())?.is_truthy()? {
                    routine.action.interpret(Some(f), p)?;
                }
            }
            None => routine.action.interpret(Some(f), p)?,
        }
    }

    Ok(())
}

#[derive(Debug)]
pub struct Condition {
    pub expr: Expression,
}

#[derive(Debug)]
pub enum Statement {
    Assignment(Assignment),
    Print(Vec<Expression>),
}

impl Statement {
    fn interpret<T: crate::SyncWrite>(
        &self,
        f: Option<&FileState>,
        p: &ProgramState<T>,
    ) -> crate::Result<()> {
        match self {
            Statement::Assignment(a) => {
                p.vars().set_variable_expression(&a.lhs, f, &a.rhs)?;
            }
            Statement::Print(exprs) => {
                let mut exprs = exprs.iter();
                let mut s = match exprs.nth(0) {
                    Some(expr) => format!("{}", expr.evaluate(f, p.vars())?),
                    None => {
                        let _ = p.out.write("\n".as_bytes());
                        return Ok(());
                    }
                };
                for expr in exprs {
                    s.push_str(&format!(" {}", expr.evaluate(f, p.vars())?));
                }
                s.push('\n');
                let _ = p.out.write(s.as_bytes());
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Assignment {
    pub lhs: Variable,
    pub rhs: Expression,
}

#[derive(Debug)]
pub struct Action {
    pub statements: Option<Vec<Statement>>,
}

impl Action {
    pub fn new(statement: Option<Vec<Statement>>) -> Self {
        match statement {
            Some(s) => Action {
                statements: Some(s),
            },
            None => Action { statements: None },
        }
    }

    fn interpret<T: crate::SyncWrite>(
        &self,
        f: Option<&FileState>,
        p: &ProgramState<T>,
    ) -> crate::Result<()> {
        match &self.statements {
            Some(statements) => {
                for st in statements.iter() {
                    st.interpret(f, p)?
                }
            }
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

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct BinaryOp {
    pub kind: OpKind,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

impl BinaryOp {
    fn evaluate(&self, f: Option<&FileState>, vars: &VariableState) -> crate::Result<Value> {
        let l = self.left.evaluate(f, vars)?;
        let r = self.right.evaluate(f, vars)?;

        Ok(l.binary_op(r, self.kind)?)
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
    And,
    Or,
}

#[derive(Clone, Debug)]
pub enum Expression {
    Bin(BinaryOp),
    Attr(Attribute),
    Atom(Value),
    Var(Variable),
}

impl Expression {
    /// Evaluate an expression within the context of the given `FileState` and `VariableState`.
    pub fn evaluate(&self, f: Option<&FileState>, vars: &VariableState) -> crate::Result<Value> {
        Ok(match self {
            Expression::Bin(op) => op.evaluate(f, vars)?,
            Expression::Attr(attr) => attr.evaluate(f)?,
            Expression::Atom(v) => v.clone(),
            Expression::Var(var) => var.evaluate(f, vars)?,
        })
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Atom(val) => write!(f, "{:?}", val),
            Expression::Attr(attr) => write!(f, "{:?}", attr),
            Expression::Var(var) => write!(f, "{}", var),
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
                        OpKind::And => "and",
                        OpKind::Or => "or",
                    }
                )?;
                write!(f, "{} ", op.left)?;
                write!(f, "{} ", op.right)?;
                write!(f, ")")
            }
        }
    }
}
