use std::iter::{Iterator, Peekable};
use std::result;
use std::str::Chars;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    OpenParen,
    CloseParen,
    Equal,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Plus,
    Minus,
    Times,
    Divide,
    Percent,
    Cond,
    Else,
    Let,
    List,
    Define,
    Lambda,
    BooleanLiteral(bool),
    Identifier(String),
    NumberLiteral(f64),
    StringLiteral(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenError {
    UnexpectedChar(char),
    IncompleteString,
    InvalidEscape,
}

pub type Result<T> = result::Result<T, TokenError>;

pub struct Tokenizer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Tokenizer {
            input: input.chars().peekable(),
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(&c) = self.input.peek() {
            if c.is_whitespace() {
                self.input.next();
            } else {
                break;
            }
        }
    }

    fn consume_whitespace_and_comments_until_next_input(&mut self) {
        while let Some(&c) = self.input.peek() {
            match c {
                c if c.is_whitespace() => {
                    self.input.next();
                }
                ';' => self.read_rest_of_line(),
                _ => break,
            };
        }

        self.consume_whitespace()
    }

    fn read_rest_of_line(&mut self) {
        while let Some(c) = self.input.next() {
            if c == '\n' {
                break;
            }
        }
    }

    fn read_word(&mut self) -> Token {
        let mut word = String::new();
        while let Some(&c) = self.input.peek() {
            if c.is_whitespace() {
                break;
            }

            match c {
                '(' | '[' | '{' | ')' | ']' | '}' => break,
                _ => {}
            };

            self.input.next();
            word.push(c);
        }

        match word.as_ref() {
            "cond" => Token::Cond,
            "else" => Token::Else,
            "let" => Token::Let,
            "list" => Token::List,
            "define" => Token::Define,
            "lambda" => Token::Lambda,
            _ => Token::Identifier(word),
        }
    }

    fn read_hash_value(&mut self) -> Token {
        let mut word = String::new();
        while let Some(&c) = self.input.peek() {
            if c.is_whitespace() {
                break;
            }

            self.input.next();
            word.push(c);
        }

        match word.as_ref() {
            "t" | "true" => Token::BooleanLiteral(true),
            "f" | "false" => Token::BooleanLiteral(false),
            _ => Token::Identifier(word), // TODO
        }
    }

    fn read_number(&mut self) -> f64 {
        let mut num = String::new();
        while let Some(&c) = self.input.peek() {
            if !c.is_numeric() {
                break;
            }

            self.input.next();
            num.push(c);
        }

        if let Some(&'.') = self.input.peek() {
            self.input.next();
            num.push('.');

            while let Some(&c) = self.input.peek() {
                if !c.is_numeric() {
                    break;
                }

                self.input.next();
                num.push(c);
            }
        }

        num.parse().unwrap()
    }

    fn read_string(&mut self) -> Result<Token> {
        // Skip the opening quote.
        self.input.next();

        let mut buf = String::new();
        while let Some(&c) = self.input.peek() {
            self.input.next();
            match c {
                '"' => return Ok(Token::StringLiteral(buf)),
                '\\' => match self.input.peek() {
                    Some(&c) if c == '"' || c == '\\' => {
                        self.input.next();
                        buf.push(c);
                    }
                    _ => return Err(TokenError::InvalidEscape),
                },
                _ => buf.push(c),
            }
        }

        buf.insert(0, '"');
        Err(TokenError::IncompleteString)
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.consume_whitespace_and_comments_until_next_input();

        match self.input.peek() {
            None => None,
            Some('(') | Some('[') | Some('{') => {
                self.input.next();
                Some(Ok(Token::OpenParen))
            }
            Some(')') | Some(']') | Some('}') => {
                self.input.next();
                Some(Ok(Token::CloseParen))
            }
            Some('=') => {
                self.input.next();
                if let Some(&c) = self.input.peek() {
                    if c.is_whitespace() {
                        return Some(Ok(Token::Equal));
                    }
                }

                Some(Err(TokenError::IncompleteString))
            }
            Some('<') => {
                self.input.next();
                if let Some(&'=') = self.input.peek() {
                    self.input.next();
                    Some(Ok(Token::LtEq))
                } else {
                    Some(Ok(Token::Lt))
                }
            }
            Some('>') => {
                self.input.next();
                if let Some(&'=') = self.input.peek() {
                    self.input.next();
                    Some(Ok(Token::GtEq))
                } else {
                    Some(Ok(Token::Gt))
                }
            }
            Some('+') => {
                self.input.next();
                match self.input.peek() {
                    Some(&c) if c.is_numeric() => {
                        Some(Ok(Token::NumberLiteral(self.read_number())))
                    }
                    _ => Some(Ok(Token::Plus)),
                }
            }
            Some('-') => {
                self.input.next();
                match self.input.peek() {
                    Some(&c) if c.is_numeric() => {
                        Some(Ok(Token::NumberLiteral(self.read_number() * -1.0)))
                    }
                    _ => Some(Ok(Token::Minus)),
                }
            }
            Some('*') => {
                self.input.next();
                Some(Ok(Token::Times))
            }
            Some('/') => {
                self.input.next();
                Some(Ok(Token::Divide))
            }
            Some('%') => {
                self.input.next();
                Some(Ok(Token::Percent))
            }
            Some('#') => {
                self.input.next();
                Some(Ok(self.read_hash_value()))
            }
            Some('"') => Some(self.read_string()),
            Some(c)
                if !c.is_whitespace() && (c.is_alphabetic() && !c.is_numeric()) || *c == '_' =>
            {
                Some(Ok(self.read_word()))
            }
            Some(c) if c.is_numeric() => Some(Ok(Token::NumberLiteral(self.read_number()))),
            Some(_) => {
                match self.input.next() {
                    Some(e) => Some(Err(TokenError::UnexpectedChar(e))),
                    _ => None,
                }

                // Some(Err(TokenError::UnexpectedChar(c)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use TokenError;

    use super::Token::*;
    use super::*;

    #[test]
    fn test_punctuation() {
        let mut s = Tokenizer::new("(,) = < <= > >= +-*/%");
        assert_eq!(s.next(), Some(Ok(OpenParen)));
        assert_eq!(s.next(), Some(Err(TokenError::UnexpectedChar(','))));
        assert_eq!(s.next(), Some(Ok(CloseParen)));
        assert_eq!(s.next(), Some(Ok(Equal)));
        assert_eq!(s.next(), Some(Ok(Lt)));
        assert_eq!(s.next(), Some(Ok(LtEq)));
        assert_eq!(s.next(), Some(Ok(Gt)));
        assert_eq!(s.next(), Some(Ok(GtEq)));
        assert_eq!(s.next(), Some(Ok(Plus)));
        assert_eq!(s.next(), Some(Ok(Minus)));
        assert_eq!(s.next(), Some(Ok(Times)));
        assert_eq!(s.next(), Some(Ok(Divide)));
        assert_eq!(s.next(), Some(Ok(Percent)));
        assert_eq!(s.next(), None);
    }

    #[test]
    fn test_unexpected_char() {
        let mut s = Tokenizer::new("($)");
        assert_eq!(s.next(), Some(Ok(OpenParen)));
        assert_eq!(s.next(), Some(Err(TokenError::UnexpectedChar('$'))));
    }

    #[test]
    fn test_words() {
        let mut s = Tokenizer::new("foo FOO _123_ Nil else #f #t");
        assert_eq!(s.next(), Some(Ok(Identifier("foo".to_owned()))));
        assert_eq!(s.next(), Some(Ok(Identifier("FOO".to_owned()))));
        assert_eq!(s.next(), Some(Ok(Identifier("_123_".to_owned()))));
        assert_eq!(s.next(), Some(Ok(Identifier("Nil".to_owned()))));
        // assert_eq!(s.next(), Some(Ok(If)));
        assert_eq!(s.next(), Some(Ok(Else)));
        assert_eq!(s.next(), Some(Ok(BooleanLiteral(false))));
        assert_eq!(s.next(), Some(Ok(BooleanLiteral(true))));
        assert_eq!(s.next(), None);
    }

    #[test]
    fn test_number() {
        let mut s = Tokenizer::new("0 -0 -1.2 +2.3 999 1.");
        assert_eq!(s.next(), Some(Ok(NumberLiteral(0.0))));
        assert_eq!(s.next(), Some(Ok(NumberLiteral(0.0))));
        assert_eq!(s.next(), Some(Ok(NumberLiteral(-1.2))));
        assert_eq!(s.next(), Some(Ok(NumberLiteral(2.3))));
        assert_eq!(s.next(), Some(Ok(NumberLiteral(999.0))));
        assert_eq!(s.next(), Some(Ok(NumberLiteral(1.0))));
        assert_eq!(s.next(), None);
    }

    #[test]
    fn test_string() {
        let mut s = Tokenizer::new(r#" "" "Foo bar" "\"\\" "#);
        assert_eq!(s.next(), Some(Ok(StringLiteral("".to_owned()))));
        assert_eq!(s.next(), Some(Ok(StringLiteral("Foo bar".to_owned()))));
        assert_eq!(s.next(), Some(Ok(StringLiteral(r#""\"#.to_owned()))));
        assert_eq!(s.next(), None);
    }

    #[test]
    fn test_comment() {
        let mut s = Tokenizer::new(";!/usr/bin/gate\n   ; foo\n");
        assert_eq!(s.next(), None);
    }

    #[test]
    fn scheme_statement() {
        let s = Tokenizer::new("(apples (function a b) (+ a b))");
        let res: Result<Vec<Token>> = s.collect();

        let expected: Vec<Token> = vec![
            OpenParen,
            Identifier("apples".to_string()),
            OpenParen,
            Identifier("function".to_string()),
            Identifier("a".to_string()),
            Identifier("b".to_string()),
            CloseParen,
            OpenParen,
            Plus,
            Identifier("a".to_string()),
            Identifier("b".to_string()),
            CloseParen,
            CloseParen,
        ];

        assert_eq!(res.unwrap(), expected);
    }
}
