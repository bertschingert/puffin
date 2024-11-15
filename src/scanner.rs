use crate::Attribute;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Value(u64),
    EqualEqual,
    Greater,
    Attr(Attribute),
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
    pub fn new(source: &'a str) -> Scanner {
        Scanner {
            source,
            chars: source.char_indices().peekable(),
            start: 0,
            current: 0,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let Some((ind, ch)) = self.chars.next() else {
            return Token::Eof;
        };

        match ch {
            '>' => Token::Greater,
            _ if ch.is_numeric() => {
                self.start = ind;
                self.current = ind;
                self.number()
            }
            _ if ch.is_alphabetic() => {
                self.start = ind;
                self.current = ind;
                self.attribute()
            }
            _ => self.error(&format!("Unexpected character: {}", ch)),
        }
    }

    fn attribute(&mut self) -> Token {
        loop {
            match self.chars.peek(){
                Some((ind, ch)) if ch.is_alphanumeric() => {
                    self.current = *ind;
                    self.chars.next();
                }
                _ => {
                    // TODO: make this return an attribute enum, not a string
                    let a = &self.source[self.start..self.current + 1];
                    return attribute_from_str(a);
                }
            }
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
                            return self.error(&format!("Could not parse number from '{}': {e}", self.current_token_text()));
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

fn attribute_from_str(s: &str) -> Token {
    match s {
        "size" => Token::Attr(Attribute::Size),
        "owner" => Token::Attr(Attribute::Owner),
        a => Token::Error("Unknown attribute".to_string()),
    }
}
