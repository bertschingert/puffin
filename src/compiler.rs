use crate::ast::*;
use crate::program_state::ProgramState;
use crate::scanner::*;
use crate::variables::*;

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    current: Token,
    next: Token,
}

impl<'a> Compiler<'a> {
    pub fn new(scanner: Scanner<'a>) -> Self {
        Compiler {
            scanner,
            current: Token::Error("uninitialized".to_string()),
            next: Token::Error("uninitialized".to_string()),
        }
    }

    pub fn compile<'b, 'c, T: crate::SyncWrite>(&mut self, out: &'b mut T) -> Program<'b, 'c, T> {
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
                    begin = Some(self.action());
                }
                Token::End => {
                    self.next();
                    self.eat(Token::LeftBrace);
                    end = Some(self.action());
                }
                _ => routines.push(self.routine()),
            };
        }

        // If no routines were provided in the input, then create a single default routine:
        let routines = match routines.len() {
            0 => {
                vec![Routine::new(None, Action { statements: None })]
            }
            _ => routines,
        };

        Program {
            begin,
            end,
            routines,
            // XXX: find a better way to move prog_state out...
            prog_state: ProgramState::new(self.scanner.num_vars(), out),
        }
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

    fn routine(&mut self) -> Routine {
        let cond = match self.peek() {
            Token::LeftBrace => None,
            _ => Some(Condition {
                expr: self.expression(0),
            }),
        };

        let action = match self.peek() {
            Token::LeftBrace => {
                self.next();
                self.action()
            }
            Token::Eof => Action::new(None),
            tok => panic!("Unexpected token: {:?}", tok),
        };

        Routine::new(cond, action)
    }

    fn action(&mut self) -> Action {
        match self.peek() {
            Token::RightBrace => {
                self.next();
                Action::new(None)
            }
            _ => {
                let action = Action::new(Some(self.statements()));
                self.eat(Token::RightBrace);
                action
            }
        }
    }

    fn statements(&mut self) -> Vec<Statement> {
        let mut statements = Vec::new();
        loop {
            statements.push(self.statement());
            match self.peek() {
                Token::RightBrace => break,
                // XXX: allow newline to separate statement?
                Token::Semicolon => self.next(),
                _ => panic!("Expected either ';' or '}}' after a statement."),
            };
        }

        statements
    }
    fn statement(&mut self) -> Statement {
        match self.next() {
            Token::Identifier(id) => {
                let id = *id;
                let lhs = self.variable(id);
                let rhs = match self.next() {
                    Token::Equal => self.expression(0),
                    Token::PlusEqual => self.compound_assignment(lhs.clone(), Token::PlusEqual),
                    Token::MinusEqual => self.compound_assignment(lhs.clone(), Token::MinusEqual),
                    tok => panic!("Unexpected token: {:?}", tok),
                };
                Statement::Assignment(Assignment { lhs, rhs })
            }
            Token::Print => Statement::Print(self.expressions()),
            tok => panic!("Unexpected token: {:?}", tok),
        }
    }

    fn compound_assignment(&mut self, var: Variable, tok: Token) -> Expression {
        let kind = match tok {
            Token::PlusEqual => OpKind::Plus,
            Token::MinusEqual => OpKind::Minus,
            _ => unreachable!(),
        };

        Expression::Bin(BinaryOp {
            kind,
            left: Box::new(Expression::Var(var)),
            right: Box::new(self.expression(0)),
        })
    }

    fn expressions(&mut self) -> Vec<Expression> {
        let mut exprs = Vec::new();
        loop {
            match self.peek() {
                Token::Comma => {
                    self.next();
                    continue;
                }
                Token::RightBrace => return exprs,
                Token::Semicolon => return exprs,
                _ => exprs.push(self.expression(0)),
            }
        }
    }

    fn expression(&mut self, min_precedence: u8) -> Expression {
        let mut left = self.factor();

        let mut next = self.peek();
        loop {
            match next {
                Token::BinOp(op) => {
                    let op = *op;
                    if Self::op_precedence(op) < min_precedence {
                        break;
                    }

                    self.next();

                    let right = self.expression(Self::op_precedence(op));
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

        left
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

    fn factor(&mut self) -> Expression {
        let e = match self.next() {
            Token::Value(v) => Expression::Atom(v.clone()),
            Token::Attr(a) => Expression::Attr(*a),
            Token::Identifier(id) => {
                let id = *id;
                Expression::Var(self.variable(id))
            }
            t => panic!("Unexpected token {:?}", t),
        };
        e
    }

    fn variable(&mut self, id: usize) -> Variable {
        match self.peek() {
            Token::LeftBracket => {
                self.next();
                let e = self.expression(0);
                self.eat(Token::RightBracket);
                Variable::ArrSub(ArraySubscript {
                    id: id,
                    subscript: Box::new(e),
                })
            }
            _ => Variable::Scalar(Identifier { id: id }),
        }
    }
}
