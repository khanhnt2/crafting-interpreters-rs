use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenValue {
    Nil,
    Bool(bool),
    String(String),
    Number(f64),
}

impl fmt::Display for TokenValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenValue::Nil => write!(f, "nil"),
            TokenValue::Bool(b) => write!(f, "{b}"),
            TokenValue::String(s) => write!(f, "{s}"),
            TokenValue::Number(n) => write!(f, "{n}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub id: TokenIdentity,
    pub value: TokenValue,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(id: TokenIdentity, value: TokenValue, line: usize, column: usize) -> Self {
        Token {
            id,
            value,
            line,
            column,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self.id {
            TokenIdentity::LeftParen => "(",
            TokenIdentity::RightParen => ")",
            TokenIdentity::LeftBrace => "{",
            TokenIdentity::RightBrace => "}",
            TokenIdentity::Colon => ":",
            TokenIdentity::Comma => ",",
            TokenIdentity::Dot => ".",
            TokenIdentity::Minus => "-",
            TokenIdentity::Plus => "+",
            TokenIdentity::Semicolon => ";",
            TokenIdentity::Slash => "/",
            TokenIdentity::Star => "*",
            TokenIdentity::Question => "?",
            TokenIdentity::Bang => "!",
            TokenIdentity::BangEqual => "!=",
            TokenIdentity::Equal => "=",
            TokenIdentity::EqualEqual => "==",
            TokenIdentity::Greater => ">",
            TokenIdentity::GreaterEqual => ">=",
            TokenIdentity::Less => "<",
            TokenIdentity::LessEqual => "<=",
            TokenIdentity::Comment => "// Comment",
            TokenIdentity::Identifier => &self.value.to_string(),
            TokenIdentity::String => &self.value.to_string(),
            TokenIdentity::Number => &self.value.to_string(),
            TokenIdentity::And => "and",
            TokenIdentity::Break => "break",
            TokenIdentity::Continue => "continue",
            TokenIdentity::Class => "class",
            TokenIdentity::Else => "else",
            TokenIdentity::False => "false",
            TokenIdentity::Fun => "fun",
            TokenIdentity::For => "for",
            TokenIdentity::If => "if",
            TokenIdentity::Nil => "nil",
            TokenIdentity::Or => "or",
            TokenIdentity::Print => "print",
            TokenIdentity::Return => "return",
            TokenIdentity::Super => "super",
            TokenIdentity::This => "this",
            TokenIdentity::True => "true",
            TokenIdentity::Var => "var",
            TokenIdentity::While => "while",
            TokenIdentity::Eof => "eof",
        };

        write!(f, "{value}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenIdentity {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Colon,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Question,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Comment,
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Break,
    Continue,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}
