use crate::ast::*;
use crate::scanner::*;
use crate::Value;

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    previous: Token,
    current: Token,
}

impl<'a> Compiler<'a> {
    pub fn new(scanner: Scanner<'a>) -> Self {
        Compiler {
            scanner,
            previous: Token::Error("uninitialized".to_string()),
            current: Token::Error("uninitialized".to_string()),
        }
    }

    pub fn compile(&mut self) -> Program {
        self.next();

        let routine = self.routine();

        Program {
            begin: None,
            end: None,
            routines: vec![routine],
        }
    }

    fn peek(&self) -> &Token {
        &self.current
    }

    fn next(&mut self) -> &Token {
        self.previous = std::mem::replace(&mut self.current, self.scanner.next_token());
        &self.previous
    }

    fn routine(&mut self) -> Routine {
        let cond = Condition {
            expr: self.expression(0),
        };

        println!("{}", cond.expr);

        let action = match self.peek() {
            Token::LeftBrace => {
                self.next();
                self.action()
            }
            Token::Eof => Action::new(),
            tok => panic!("Unexpected token: {:?}", tok),
        };

        Routine::new(Some(cond), action)
    }

    fn action(&mut self) -> Action {
        match self.next() {
            Token::RightBrace => {}
            tok => panic!("Unexpected token: {:?}", tok),
        };

        Action::new()
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
