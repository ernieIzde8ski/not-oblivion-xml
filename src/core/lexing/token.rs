use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum Operator {
    Dollar,
    Colon,
    Period,
    Bang,

    LeftBracket,
    RightBracket,

    Slash,
    Asterisk,
    Minus,
    Plus,
    Mod,

    EqualsSign,
    LeftAngle,
    RightAngle,

    NotEqual,
    EqualTo,
    GreaterThanEqual,
    LessThanEqual,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    /// Miscellaneous one or two char constants
    Op(Operator),
    /// An ascii newline. Inner content represents indent level.
    NewLine(String),
    StringLiteral(String),
    Number(f32),
    Identifier(String),
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dollar => write!(f, "$"),
            Self::Colon => write!(f, ":"),
            Self::Period => write!(f, "."),
            Self::Bang => write!(f, "!"),
            Self::LeftBracket => write!(f, "("),
            Self::RightBracket => write!(f, ")"),
            Self::Slash => write!(f, "/"),
            Self::Asterisk => write!(f, "*"),
            Self::Minus => write!(f, "-"),
            Self::Plus => write!(f, "+"),
            Self::Mod => write!(f, "%"),
            Self::EqualsSign => write!(f, "="),
            Self::LeftAngle => write!(f, "<"),
            Self::RightAngle => write!(f, ">"),
            Self::NotEqual => write!(f, "!="),
            Self::EqualTo => write!(f, "=="),
            Self::GreaterThanEqual => write!(f, ">="),
            Self::LessThanEqual => write!(f, "<="),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Op(op) => write!(f, "{op}"),
            Self::NewLine(s) => write!(f, "\n{s}"),
            Self::Identifier(s) => write!(f, "{s}"),
            Self::StringLiteral(s) => write!(f, "\"{}\"", s.replace("\"", "\\\"")),
            Self::Number(num) => write!(f, "{num}"),
        }
    }
}

#[derive(Debug)]
pub enum TokenError {
    UnterminatedStringLiteral(String),
    InvalidChar(char),
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnterminatedStringLiteral(s) => write!(f, "UnterminatedStringLiteral: {s}"),
            Self::InvalidChar(s) => write!(f, "InvalidChar: {s}"),
        }
    }
}

impl std::error::Error for TokenError {}
