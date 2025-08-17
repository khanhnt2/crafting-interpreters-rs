use std::fmt;

use crate::{
    object::Object,
    token::{Token, TokenIdentity},
};

pub enum RuntimeException {
    Break,
    Continue,
    Error(RuntimeError),
    Return(RuntimeReturn),
}

impl fmt::Display for RuntimeException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error(err) => write!(f, "{err}"),
            Self::Return(ret) => write!(f, "{ret}"),
            Self::Break => write!(f, "break"),
            Self::Continue => write!(f, "continue"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeReturn {
    pub value: Object,
}

impl fmt::Display for RuntimeReturn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl RuntimeReturn {
    pub fn new(value: Object) -> Self {
        Self { value }
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
    token: Token,
}

impl RuntimeError {
    pub fn new(token: Token, message: &str) -> Self {
        Self {
            message: message.to_string(),
            token,
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.token.id == TokenIdentity::Eof {
            write!(
                f,
                "[line {}:{}] Runtime error at end: {}",
                self.token.line, self.token.column, self.message
            )
        } else {
            write!(
                f,
                "[line {}:{}] Runtime error at '{}': {}",
                self.token.line, self.token.column, self.token, self.message
            )
        }
    }
}

#[derive(Debug)]
pub struct ParsingError {
    message: String,
    token: Token,
}

impl ParsingError {
    pub fn new(token: Token, message: &str) -> Self {
        Self {
            message: message.to_string(),
            token,
        }
    }
}

impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.token.id == TokenIdentity::Eof {
            write!(
                f,
                "[line {}:{}] Parsing error at end: {}",
                self.token.line, self.token.column, self.message
            )
        } else {
            write!(
                f,
                "[line {}:{}] Parsing error at '{}': {}",
                self.token.line, self.token.column, self.token, self.message
            )
        }
    }
}
