use crate::ast::*;
use crate::scanner::*;

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

    pub fn compile(&mut self) -> Program {
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

        Program {
            begin,
            end,
            routines,
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
        match self.next() {
            Token::RightBrace => Action::new(None),
            Token::Print => {
                let expr = self.expression(0);
                let action = Action::new(Some(Statement::Print(expr)));
                self.eat(Token::RightBrace);
                action
            }
            tok => panic!("Unexpected token: {:?}", tok),
        }
    }

    fn expression(&mut self, min_precedence: u8) -> Expression {
        let mut left = self.factor();

        let mut next = self.peek();
        while let Token::BinOp(op) = next {
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

        left
    }

    fn op_precedence(op: OpKind) -> u8 {
        match op {
            OpKind::Multiply => 3,
            OpKind::Divide => 3,
            OpKind::Plus => 2,
            OpKind::Minus => 2,
            OpKind::Greater => 1,
            OpKind::EqualEqual => 1,
        }
    }

    fn factor(&mut self) -> Expression {
        let next = self.peek();
        let e = match next {
            Token::Value(v) => Expression::Atom(Value::Integer(*v)),
            Token::Attr(a) => Expression::Attr(*a),
            t => panic!("Unexpected token {:?}", t),
        };
        self.next();
        e
    }
}
