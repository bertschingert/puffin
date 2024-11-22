use crate::ast::*;
use crate::program_state::ProgramState;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Begin,
    End,
    Equal,
    LeftBrace,
    RightBrace,
    Print,
    Value(u64),
    BinOp(OpKind),
    Attr(Attribute),
    Identifier(usize),
    Eof,
    Error(String),
}

pub struct Scanner<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    start: usize,
    current: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Scanner<'a> {
        Scanner {
            source,
            chars: source.char_indices().peekable(),
            start: 0,
            current: 0,
        }
    }

    pub fn next_token(&mut self, prog_state: &mut ProgramState) -> Token {
        self.skip_whitespace();

        let Some((ind, ch)) = self.chars.next() else {
            return Token::Eof;
        };

        match ch {
            '=' => Token::Equal,
            '>' => Token::BinOp(OpKind::Greater),
            '+' => Token::BinOp(OpKind::Plus),
            '-' => Token::BinOp(OpKind::Minus),
            '*' => Token::BinOp(OpKind::Multiply),
            '/' => Token::BinOp(OpKind::Divide),
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            _ if ch.is_numeric() => {
                self.start = ind;
                self.current = ind;
                self.number()
            }
            _ if ch.is_alphabetic() => {
                self.start = ind;
                self.current = ind;
                self.word(prog_state)
            }
            _ => self.error(&format!("Unexpected character: {}", ch)),
        }
    }

    fn word(&mut self, prog_state: &mut ProgramState) -> Token {
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
                        a => Self::attribute_or_identifier(a, prog_state),
                    };
                }
            }
        }
    }

    fn attribute_or_identifier(s: &str, prog_state: &mut ProgramState) -> Token {
        match s {
            "size" => Token::Attr(Attribute::Size),
            "owner" => Token::Attr(Attribute::Owner),
            a => Token::Identifier(prog_state.add_variable(a)),
        }
    }

    fn number(&mut self) -> Token {
        loop {
            match self.chars.peek() {
                Some((ind, ch)) if ch.is_alphanumeric() => {
                    self.current = *ind;
                    self.chars.next();
                }
                _ => {
                    let num = match self.source[self.start..self.current + 1].parse::<u64>() {
                        Ok(num) => num,
                        Err(e) => {
                            return self.error(&format!(
                                "Could not parse number from '{}': {e}",
                                self.current_token_text()
                            ));
                        }
                    };

                    return Token::Value(num);
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
}
