use crate::ast::*;
use crate::types::*;

// TODO: implement Display for Token to get better error messages
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Begin,
    End,
    Equal,
    PlusEqual,
    MinusEqual,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    Semicolon,
    Comma,
    Print,
    Value(Value),
    BinOp(OpKind),
    Attr(Attribute),
    Identifier(String),
    Eof,
    Error(String),
}

pub struct Scanner<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    start: usize,
    current: usize,
    num_vars: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Scanner<'a> {
        Scanner {
            source,
            chars: source.char_indices().peekable(),
            start: 0,
            current: 0,
            num_vars: 0,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let Some((ind, ch)) = self.chars.next() else {
            return Token::Eof;
        };

        match ch {
            '=' => self.oneplus_token('=', Token::BinOp(OpKind::EqualEqual), Token::Equal),
            '>' => self.oneplus_token(
                '=',
                Token::BinOp(OpKind::GreaterEqual),
                Token::BinOp(OpKind::Greater),
            ),
            '<' => self.oneplus_token(
                '=',
                Token::BinOp(OpKind::LessEqual),
                Token::BinOp(OpKind::Less),
            ),
            '+' => self.oneplus_token('=', Token::PlusEqual, Token::BinOp(OpKind::Plus)),
            '-' => self.oneplus_token('=', Token::MinusEqual, Token::BinOp(OpKind::Minus)),
            '*' => Token::BinOp(OpKind::Multiply),
            '/' => Token::BinOp(OpKind::Divide),
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            ';' => Token::Semicolon,
            ',' => Token::Comma,
            '"' => {
                self.start = ind + 1;
                self.current = ind;
                self.string()
            }
            '.' => {
                self.start = ind;
                self.current = ind;
                self.attribute()
            }
            _ if ch.is_numeric() => {
                self.start = ind;
                self.current = ind;
                self.number()
            }
            _ if ch.is_alphabetic() => {
                self.start = ind;
                self.current = ind;
                self.word()
            }
            _ => self.error(&format!("Unexpected character: {}", ch)),
        }
    }

    fn oneplus_token(&mut self, next_char: char, yes: Token, no: Token) -> Token {
        match self.chars.peek() {
            Some((_, ch)) if *ch == next_char => {
                self.chars.next();
                yes
            }
            _ => no,
        }
    }

    fn string(&mut self) -> Token {
        loop {
            match self.chars.peek() {
                Some((_, '"')) => {
                    self.chars.next();
                    let s = &self.source[self.start..self.current + 1];
                    return Token::Value(Value::String(s.to_string()));
                }
                Some((ind, _)) => {
                    self.current = *ind;
                    self.chars.next();
                }
                None => {
                    return Token::Error(
                        "Unexpected end of input while parsing a string".to_string(),
                    )
                }
            }
        }
    }

    fn attribute(&mut self) -> Token {
        loop {
            match self.chars.peek() {
                Some((ind, ch)) if ch.is_alphanumeric() => {
                    self.current = *ind;
                    self.chars.next();
                }
                _ => {
                    let a = &self.source[self.start..self.current + 1];

                    return match Attribute::from_str(a) {
                        Some(a) => Token::Attr(a),
                        None => Token::Error(format!("Unknown attribute '{a}'")),
                    };
                }
            }
        }
    }

    fn word(&mut self) -> Token {
        loop {
            match self.chars.peek() {
                Some((ind, ch)) if ch.is_alphanumeric() => {
                    self.current = *ind;
                    self.chars.next();
                }
                _ => {
                    let a = &self.source[self.start..self.current + 1];
                    return match a {
                        "BEGIN" => Token::Begin,
                        "begin" => Token::Begin,
                        "END" => Token::End,
                        "end" => Token::End,
                        "print" => Token::Print,
                        "and" => Token::BinOp(OpKind::And),
                        "or" => Token::BinOp(OpKind::Or),
                        a => self.identifier(a),
                    };
                }
            }
        }
    }

    fn identifier(&mut self, s: &'a str) -> Token {
        Token::Identifier(s.to_string())
    }

    fn number(&mut self) -> Token {
        loop {
            match self.chars.peek() {
                Some((ind, ch)) if ch.is_alphanumeric() => {
                    self.current = *ind;
                    self.chars.next();
                }
                _ => {
                    let num = match self.source[self.start..self.current + 1].parse::<i64>() {
                        Ok(num) => num,
                        Err(e) => {
                            return self.error(&format!(
                                "Could not parse number from '{}': {e}",
                                self.current_token_text()
                            ));
                        }
                    };

                    return Token::Value(Value::Int(num));
                }
            }
        }
    }

    fn current_token_text(&self) -> &'a str {
        &self.source[self.start..self.current + 1]
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.chars.peek() {
                Some((_, ch)) if ch.is_whitespace() => self.chars.next(),
                _ => break,
            };
        }
    }

    fn error(&self, msg: &str) -> Token {
        Token::Error(msg.to_string())
    }

    pub fn num_vars(&self) -> usize {
        self.num_vars
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::*;

    fn is_error_token(t: Token) -> bool {
        match t {
            Token::Error(_) => true,
            _ => false,
        }
    }

    #[test]
    fn numbers() {
        let mut s = Scanner::new("1 2 123a ");

        assert_eq!(s.next_token(), Token::Value(Value::Int(1)));
        assert_eq!(s.next_token(), Token::Value(Value::Int(2)));
        assert!(is_error_token(s.next_token()));
        assert_eq!(s.next_token(), Token::Eof);
    }

    #[test]
    fn binary_operators() {
        let mut s = Scanner::new("+ - */ > == =");

        assert_eq!(s.next_token(), Token::BinOp(OpKind::Plus));
        assert_eq!(s.next_token(), Token::BinOp(OpKind::Minus));
        assert_eq!(s.next_token(), Token::BinOp(OpKind::Multiply));
        assert_eq!(s.next_token(), Token::BinOp(OpKind::Divide));
        assert_eq!(s.next_token(), Token::BinOp(OpKind::Greater));
        assert_eq!(s.next_token(), Token::BinOp(OpKind::EqualEqual));
        assert_eq!(s.next_token(), Token::Equal);
        assert_eq!(s.next_token(), Token::Eof);
    }

    #[test]
    fn compound_assignment_operators() {
        let mut s = Scanner::new("+ += - -=");

        assert_eq!(s.next_token(), Token::BinOp(OpKind::Plus));
        assert_eq!(s.next_token(), Token::PlusEqual);
        assert_eq!(s.next_token(), Token::BinOp(OpKind::Minus));
        assert_eq!(s.next_token(), Token::MinusEqual);
        assert_eq!(s.next_token(), Token::Eof);
    }

    #[test]
    fn keywords() {
        let mut s = Scanner::new("BEGIN begin END end print");

        assert_eq!(s.next_token(), Token::Begin);
        assert_eq!(s.next_token(), Token::Begin);
        assert_eq!(s.next_token(), Token::End);
        assert_eq!(s.next_token(), Token::End);
        assert_eq!(s.next_token(), Token::Print);
        assert_eq!(s.next_token(), Token::Eof);
    }

    #[test]
    fn attributes() {
        let mut s = Scanner::new(".size .invalid");

        assert_eq!(s.next_token(), Token::Attr(Attribute::Size));
        assert!(is_error_token(s.next_token()));
        assert_eq!(s.next_token(), Token::Eof);
    }

    #[test]
    fn identifiers() {
        let mut s = Scanner::new("id id2 id .size id2");

        assert_eq!(s.next_token(), Token::Identifier("id".to_string()));
        assert_eq!(s.next_token(), Token::Identifier("id2".to_string()));
        assert_eq!(s.next_token(), Token::Identifier("id".to_string()));
        assert_eq!(s.next_token(), Token::Attr(Attribute::Size));
        assert_eq!(s.next_token(), Token::Identifier("id2".to_string()));
        assert_eq!(s.next_token(), Token::Eof);
    }

    #[test]
    fn strings() {
        let mut s = Scanner::new("\"hey\" \"there\" \"error");

        assert_eq!(s.next_token(), Token::Value(Value::String("hey".to_string())));
        assert_eq!(s.next_token(), Token::Value(Value::String("there".to_string())));
        assert!(is_error_token(s.next_token()));
        assert_eq!(s.next_token(), Token::Eof);
    }

    #[test]
    fn other_tokens() {
        let mut s = Scanner::new("{ } ;, []");
        assert_eq!(s.next_token(), Token::LeftBrace);
        assert_eq!(s.next_token(), Token::RightBrace);
        assert_eq!(s.next_token(), Token::Semicolon);
        assert_eq!(s.next_token(), Token::Comma);
        assert_eq!(s.next_token(), Token::LeftBracket);
        assert_eq!(s.next_token(), Token::RightBracket);
        assert_eq!(s.next_token(), Token::Eof);
    }
}
