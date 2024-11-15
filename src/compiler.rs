use crate::ast::*;
use crate::scanner::*;
use crate::Attribute;
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
        let cond = Condition {
            expr: self.expression(),
        };

        Program {
            begin: None,
            end: None,
            routines: vec![(Some(cond), Action::new())],
        }
    }

    fn expression(&mut self) -> Expression {
        let left = match self.scanner.next_token() {
            Token::Attr(a) => Expression::Attr(a),
            Token::Value(v) => Expression::Atom(Value::Integer(v)),
            tok => panic!("Unexpected token: {:?}", tok),
        };

        let op = match self.scanner.next_token() {
            Token::Greater => OpKind::Greater,
            tok => panic!("Unexpected token: {:?}", tok),
        };

        let right = match self.scanner.next_token() {
            Token::Attr(a) => Expression::Attr(a),
            Token::Value(v) => Expression::Atom(Value::Integer(v)),
            tok => panic!("Unexpected token: {:?}", tok),
        };

        Expression::Bin(BinaryOp {
            kind: op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }
}
