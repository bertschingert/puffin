use std::collections::HashMap;

use crate::ast::*;
use crate::program_state::ProgramState;
use crate::scanner::*;
use crate::variables::*;

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    current: Token,
    next: Token,
    /// When array identifiers (e.g., `arr` in `arr["key"]`) are encountered during compiling, they
    /// are added to the `known_arrays` map. Identifiers that are not followed by a subscript
    /// expression are resolved to a type (`Arr` or `Scalar`) in a later pass since it's not known
    /// immediately which type they refer to.
    known_arrays: HashMap<String, usize>,
    num_arrays: usize,
}

impl<'a> Compiler<'a> {
    pub fn new(scanner: Scanner<'a>) -> Self {
        Compiler {
            scanner,
            current: Token::Error("uninitialized".to_string()),
            next: Token::Error("uninitialized".to_string()),
            known_arrays: HashMap::new(),
            num_arrays: 0,
        }
    }

    pub fn compile<'b, 'c, T: crate::SyncWrite>(
        &mut self,
        out: &'b mut T,
    ) -> crate::Result<Program<'b, 'c, T>> {
        self.next();

        let mut begin = None;
        let mut end = None;
        let mut routines = Vec::new();
        loop {
            match self.peek() {
                Token::Eof => break,
                Token::Begin => {
                    self.next();
                    self.eat(Token::LeftBrace);
                    begin = Some(self.action()?);
                }
                Token::End => {
                    self.next();
                    self.eat(Token::LeftBrace);
                    end = Some(self.action()?);
                }
                _ => routines.push(self.routine()?),
            };
        }

        // If no routines were provided in the input, then create a single default routine:
        let mut routines = match routines.len() {
            0 => {
                vec![Routine::new(None, Action { statements: None })]
            }
            _ => routines,
        };

        let num_scalars =
            analysis::analyze(&self.known_arrays, &mut begin, &mut end, &mut routines)?;

        Ok(Program {
            begin,
            end,
            routines,
            prog_state: ProgramState::new(num_scalars, self.num_arrays, out),
        })
    }

    fn eat(&mut self, tok: Token) {
        let next = self.next();
        if *next != tok {
            panic!("Unexpected token: {:?}", next);
        }
    }

    fn peek(&self) -> &Token {
        &self.next
    }

    fn next(&mut self) -> &Token {
        self.current = std::mem::replace(&mut self.next, self.scanner.next_token());
        &self.current
    }

    fn routine(&mut self) -> crate::Result<Routine> {
        let cond = match self.peek() {
            Token::LeftBrace => None,
            _ => Some(Condition {
                expr: self.expression(0)?,
            }),
        };

        let action = match self.peek() {
            Token::LeftBrace => {
                self.next();
                self.action()?
            }
            Token::Eof => Action::new(None),
            tok => panic!("Unexpected token: {:?}", tok),
        };

        Ok(Routine::new(cond, action))
    }

    fn action(&mut self) -> crate::Result<Action> {
        Ok(match self.peek() {
            Token::RightBrace => {
                self.next();
                Action::new(None)
            }
            _ => {
                let action = Action::new(Some(self.statements()?));
                self.eat(Token::RightBrace);
                action
            }
        })
    }

    fn statements(&mut self) -> crate::Result<Vec<Statement>> {
        let mut statements = Vec::new();
        loop {
            if let Some(st) = self.statement()? {
                statements.push(st);
            };
            match self.peek() {
                Token::RightBrace => break,
                // XXX: allow newline to separate statement?
                Token::Semicolon => self.next(),
                _ => panic!("Expected either ';' or '}}' after a statement."),
            };
        }

        Ok(statements)
    }

    fn statement(&mut self) -> crate::Result<Option<Statement>> {
        let statement = match self.peek() {
            Token::Identifier(name) => {
                let name = name.clone();
                self.next();
                let lhs = self.variable(name)?;
                let rhs = match self.next() {
                    Token::Equal => self.expression(0)?,
                    Token::PlusEqual => self.compound_assignment(lhs.clone(), Token::PlusEqual)?,
                    Token::MinusEqual => {
                        self.compound_assignment(lhs.clone(), Token::MinusEqual)?
                    }
                    tok => panic!("Unexpected token: {:?}", tok),
                };
                Some(Statement::Assignment(Assignment { lhs, rhs }))
            }
            Token::Print => {
                self.next();
                Some(Statement::Print(self.expressions()?))
            }
            Token::RightBrace => None,
            Token::Semicolon => None,
            tok => panic!("Unexpected token: {:?}", tok),
        };

        Ok(statement)
    }

    fn compound_assignment(&mut self, var: Variable, tok: Token) -> crate::Result<Expression> {
        let kind = match tok {
            Token::PlusEqual => OpKind::Plus,
            Token::MinusEqual => OpKind::Minus,
            _ => unreachable!(),
        };

        Ok(Expression::Bin(BinaryOp {
            kind,
            left: Box::new(Expression::Var(var)),
            right: Box::new(self.expression(0)?),
        }))
    }

    fn expressions(&mut self) -> crate::Result<Vec<Expression>> {
        let mut exprs = Vec::new();
        loop {
            match self.peek() {
                Token::Comma => {
                    self.next();
                    continue;
                }
                Token::RightBrace => return Ok(exprs),
                Token::Semicolon => return Ok(exprs),
                _ => exprs.push(self.expression(0)?),
            }
        }
    }

    fn expression(&mut self, min_precedence: u8) -> crate::Result<Expression> {
        let mut left = self.factor()?;

        let mut next = self.peek();
        loop {
            match next {
                Token::BinOp(op) => {
                    let op = *op;
                    if Self::op_precedence(op) < min_precedence {
                        break;
                    }

                    self.next();

                    let right = self.expression(Self::op_precedence(op))?;
                    left = Expression::Bin(BinaryOp {
                        kind: op,
                        left: Box::new(left),
                        right: Box::new(right),
                    });

                    next = self.peek();
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn op_precedence(op: OpKind) -> u8 {
        match op {
            OpKind::Multiply => 3,
            OpKind::Divide => 3,
            OpKind::Plus => 2,
            OpKind::Minus => 2,
            OpKind::Greater => 1,
            OpKind::GreaterEqual => 1,
            OpKind::Less => 1,
            OpKind::LessEqual => 1,
            OpKind::EqualEqual => 1,
        }
    }

    fn factor(&mut self) -> crate::Result<Expression> {
        match self.next() {
            Token::Value(v) => Ok(Expression::Atom(v.clone())),
            Token::Attr(a) => Ok(Expression::Attr(*a)),
            Token::Identifier(name) => {
                let name = name.clone();
                Ok(Expression::Var(self.variable(name)?))
            }
            t => Err(crate::Error::compile_error(
                "Expected value, attribute, or identifier",
                t,
            )),
        }
    }

    fn variable(&mut self, name: String) -> crate::Result<Variable> {
        Ok(match self.peek() {
            Token::LeftBracket => {
                let id = self.add_array(name);
                self.next();
                let e = self.expression(0)?;
                self.eat(Token::RightBracket);
                Variable::ArrSub(ArraySubscript {
                    id: id,
                    subscript: Box::new(e),
                })
            }
            _ => Variable::NotYetKnown(name.to_string()),
        })
    }

    fn add_array(&mut self, new_array: String) -> usize {
        *self.known_arrays.entry(new_array).or_insert_with(|| {
            self.num_arrays += 1;
            self.num_arrays - 1
        })
    }
}
