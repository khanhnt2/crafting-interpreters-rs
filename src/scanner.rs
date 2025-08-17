use std::{iter::Peekable, str::Chars};

use crate::token::{Token, TokenIdentity, TokenValue};

pub struct Scanner<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
    is_finish: bool,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            chars: source.chars().peekable(),
            line: 1,
            column: 1,
            is_finish: false,
        }
    }
}

impl Iterator for Scanner<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.chars.next() {
            Some(c) => match c {
                '(' => {
                    self.column += 1;
                    Some(Token::new(
                        TokenIdentity::LeftParen,
                        TokenValue::Nil,
                        self.line,
                        self.column - 1,
                    ))
                }
                ')' => {
                    self.column += 1;
                    Some(Token::new(
                        TokenIdentity::RightParen,
                        TokenValue::Nil,
                        self.line,
                        self.column - 1,
                    ))
                }
                '{' => {
                    self.column += 1;
                    Some(Token::new(
                        TokenIdentity::LeftBrace,
                        TokenValue::Nil,
                        self.line,
                        self.column - 1,
                    ))
                }
                '}' => {
                    self.column += 1;
                    Some(Token::new(
                        TokenIdentity::RightBrace,
                        TokenValue::Nil,
                        self.line,
                        self.column - 1,
                    ))
                }
                ',' => {
                    self.column += 1;
                    Some(Token::new(
                        TokenIdentity::Comma,
                        TokenValue::Nil,
                        self.line,
                        self.column - 1,
                    ))
                }
                '.' => {
                    self.column += 1;
                    Some(Token::new(
                        TokenIdentity::Dot,
                        TokenValue::Nil,
                        self.line,
                        self.column - 1,
                    ))
                }
                '-' => {
                    self.column += 1;
                    Some(Token::new(
                        TokenIdentity::Minus,
                        TokenValue::Nil,
                        self.line,
                        self.column - 1,
                    ))
                }
                '+' => {
                    self.column += 1;
                    Some(Token::new(
                        TokenIdentity::Plus,
                        TokenValue::Nil,
                        self.line,
                        self.column - 1,
                    ))
                }
                ';' => {
                    self.column += 1;
                    Some(Token::new(
                        TokenIdentity::Semicolon,
                        TokenValue::Nil,
                        self.line,
                        self.column - 1,
                    ))
                }
                '*' => {
                    self.column += 1;
                    Some(Token::new(
                        TokenIdentity::Star,
                        TokenValue::Nil,
                        self.line,
                        self.column - 1,
                    ))
                }
                ':' => {
                    self.column += 1;
                    Some(Token::new(
                        TokenIdentity::Colon,
                        TokenValue::Nil,
                        self.line,
                        self.column - 1,
                    ))
                }
                '?' => {
                    self.column += 1;
                    Some(Token::new(
                        TokenIdentity::Question,
                        TokenValue::Nil,
                        self.line,
                        self.column - 1,
                    ))
                }
                '!' => {
                    self.column += 1;
                    if self.chars.next_if_eq(&'=').is_some() {
                        self.column += 1;
                        Some(Token::new(
                            TokenIdentity::BangEqual,
                            TokenValue::Nil,
                            self.line,
                            self.column - 2,
                        ))
                    } else {
                        Some(Token::new(
                            TokenIdentity::Bang,
                            TokenValue::Nil,
                            self.line,
                            self.column - 1,
                        ))
                    }
                }
                '=' => {
                    self.column += 1;
                    if self.chars.next_if_eq(&'=').is_some() {
                        self.column += 1;
                        Some(Token::new(
                            TokenIdentity::EqualEqual,
                            TokenValue::Nil,
                            self.line,
                            self.column - 2,
                        ))
                    } else {
                        Some(Token::new(
                            TokenIdentity::Equal,
                            TokenValue::Nil,
                            self.line,
                            self.column - 1,
                        ))
                    }
                }
                '<' => {
                    self.column += 1;
                    if self.chars.next_if_eq(&'=').is_some() {
                        self.column += 1;
                        Some(Token::new(
                            TokenIdentity::LessEqual,
                            TokenValue::Nil,
                            self.line,
                            self.column - 2,
                        ))
                    } else {
                        Some(Token::new(
                            TokenIdentity::Less,
                            TokenValue::Nil,
                            self.line,
                            self.column - 1,
                        ))
                    }
                }
                '>' => {
                    self.column += 1;
                    if self.chars.next_if_eq(&'=').is_some() {
                        self.column += 1;
                        Some(Token::new(
                            TokenIdentity::GreaterEqual,
                            TokenValue::Nil,
                            self.line,
                            self.column - 2,
                        ))
                    } else {
                        Some(Token::new(
                            TokenIdentity::Greater,
                            TokenValue::Nil,
                            self.line,
                            self.column - 1,
                        ))
                    }
                }
                '/' => {
                    self.column += 1;
                    if self.chars.next_if_eq(&'/').is_some() {
                        self.column += 1;
                        let mut text = String::new();
                        while let Some(c) = self.chars.next_if(|c| *c != '\n') {
                            text.push(c);
                        }
                        Some(Token::new(
                            TokenIdentity::Comment,
                            TokenValue::String(text),
                            self.line,
                            self.column - 2,
                        ))
                    } else {
                        Some(Token::new(
                            TokenIdentity::Slash,
                            TokenValue::Nil,
                            self.line,
                            self.column - 1,
                        ))
                    }
                }
                ' ' | '\r' | '\t' => {
                    self.column += 1;
                    self.next()
                }
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.next()
                }
                '"' => {
                    let column = self.column;
                    self.column += 1;
                    let mut value = String::new();
                    while let Some(c) = self.chars.next_if(|c| *c != '"') {
                        value.push(c);
                    }
                    if self.chars.next_if_eq(&'"').is_none() {
                        panic!(
                            "Unterminated string literal at line {}:{}",
                            self.line, column
                        );
                    }
                    self.column += value.len() + 1;
                    Some(Token::new(
                        TokenIdentity::String,
                        TokenValue::String(value),
                        self.line,
                        column,
                    ))
                }
                _ => {
                    if c.is_numeric() {
                        let column = self.column;
                        let mut value = String::from(c);
                        while let Some(c) = self.chars.next_if(|c| c.is_ascii_digit()) {
                            value.push(c);
                        }

                        if self.chars.next_if_eq(&'.').is_some_and(|c| {
                            value.push(c);
                            true
                        }) && self.chars.next_if(|c| c.is_ascii_digit()).is_some_and(|c| {
                            value.push(c);
                            true
                        }) {
                            while let Some(c) = self.chars.next_if(|c| c.is_ascii_digit()) {
                                value.push(c);
                            }
                        }
                        self.column += value.len();
                        Some(Token::new(
                            TokenIdentity::Number,
                            TokenValue::Number(
                                value.parse().unwrap_or_else(|_| {
                                    panic!("Can't parse '{value}' into a number")
                                }),
                            ),
                            self.line,
                            column,
                        ))
                    } else if c.is_alphabetic() {
                        let column = self.column;
                        let mut value = String::from(c);
                        while let Some(c) = self.chars.next_if(|c| c.is_alphabetic() || *c == '_') {
                            value.push(c);
                        }
                        self.column += value.len();
                        match value.as_str() {
                            "and" => Some(Token::new(
                                TokenIdentity::And,
                                TokenValue::Nil,
                                self.line,
                                column,
                            )),
                            "break" => Some(Token::new(
                                TokenIdentity::Break,
                                TokenValue::Nil,
                                self.line,
                                column,
                            )),
                            "continue" => Some(Token::new(
                                TokenIdentity::Continue,
                                TokenValue::Nil,
                                self.line,
                                column,
                            )),
                            "class" => Some(Token::new(
                                TokenIdentity::Class,
                                TokenValue::Nil,
                                self.line,
                                column,
                            )),
                            "else" => Some(Token::new(
                                TokenIdentity::Else,
                                TokenValue::Nil,
                                self.line,
                                column,
                            )),
                            "false" => Some(Token::new(
                                TokenIdentity::False,
                                TokenValue::Bool(false),
                                self.line,
                                column,
                            )),
                            "for" => Some(Token::new(
                                TokenIdentity::For,
                                TokenValue::Nil,
                                self.line,
                                column,
                            )),
                            "fun" => Some(Token::new(
                                TokenIdentity::Fun,
                                TokenValue::Nil,
                                self.line,
                                column,
                            )),
                            "if" => Some(Token::new(
                                TokenIdentity::If,
                                TokenValue::Nil,
                                self.line,
                                column,
                            )),
                            "nil" => Some(Token::new(
                                TokenIdentity::Nil,
                                TokenValue::Nil,
                                self.line,
                                column,
                            )),
                            "or" => Some(Token::new(
                                TokenIdentity::Or,
                                TokenValue::Nil,
                                self.line,
                                column,
                            )),
                            "print" => Some(Token::new(
                                TokenIdentity::Print,
                                TokenValue::Nil,
                                self.line,
                                column,
                            )),
                            "return" => Some(Token::new(
                                TokenIdentity::Return,
                                TokenValue::Nil,
                                self.line,
                                column,
                            )),
                            "super" => Some(Token::new(
                                TokenIdentity::Super,
                                TokenValue::String("super".to_string()),
                                self.line,
                                column,
                            )),
                            "this" => Some(Token::new(
                                TokenIdentity::This,
                                TokenValue::String("this".to_string()),
                                self.line,
                                column,
                            )),
                            "true" => Some(Token::new(
                                TokenIdentity::True,
                                TokenValue::Bool(true),
                                self.line,
                                column,
                            )),
                            "var" => Some(Token::new(
                                TokenIdentity::Var,
                                TokenValue::Nil,
                                self.line,
                                column,
                            )),
                            "while" => Some(Token::new(
                                TokenIdentity::While,
                                TokenValue::Nil,
                                self.line,
                                column,
                            )),
                            _ => Some(Token::new(
                                TokenIdentity::Identifier,
                                TokenValue::String(value),
                                self.line,
                                column,
                            )),
                        }
                    } else {
                        panic!(
                            "Unexpected character at line {}:{}: {}",
                            self.line, self.column, c
                        );
                    }
                }
            },
            None => {
                if !self.is_finish {
                    self.is_finish = true;
                    Some(Token::new(
                        TokenIdentity::Eof,
                        TokenValue::Nil,
                        self.line,
                        self.column,
                    ))
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oneline() {
        let input = "class Foo { var x = 1; }";
        let scanner = Scanner::new(input);
        let tokens: Vec<Token> = scanner.into_iter().collect();
        assert_eq!(tokens.len(), 10);
        assert_eq!(tokens[0].id, TokenIdentity::Class);
        assert_eq!(tokens[1].id, TokenIdentity::Identifier);
        assert_eq!(tokens[2].id, TokenIdentity::LeftBrace);
        assert_eq!(tokens[3].id, TokenIdentity::Var);
        assert_eq!(tokens[4].id, TokenIdentity::Identifier);
        assert_eq!(tokens[5].id, TokenIdentity::Equal);
        assert_eq!(tokens[6].id, TokenIdentity::Number);
        assert_eq!(tokens[7].id, TokenIdentity::Semicolon);
        assert_eq!(tokens[8].id, TokenIdentity::RightBrace);
        assert_eq!(tokens[9].id, TokenIdentity::Eof);
    }

    // #[test]
    // fn test_2lines() {
    //     let input = r#"// The comment
    //         class Foo { var x = 1; }"#;
    //     let scanner = Scanner::new(input);
    //     let tokens: Vec<Token> = scanner.into_iter().collect();
    // }
}
