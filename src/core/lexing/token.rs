use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum Operator {
    Dollar,
    Colon,
    Period,
    Bang,
    NewLine,

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
    /// An increment in indentation level.
    Indent,
    /// A decrement in indentation level.
    Dedent,
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
            Self::NewLine => write!(f, "\n"),
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
            Self::Indent => write!(f, "\t"),
            Self::Dedent => Ok(()),
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
    InconsistentLeadingWhitespaceChars,
    InconsistentLeadingWhitespaceCount,
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnterminatedStringLiteral(s) => write!(f, "UnterminatedStringLiteral: {s}"),
            Self::InvalidChar(s) => write!(f, "InvalidChar: {s}"),
            Self::InconsistentLeadingWhitespaceChars => {
                write!(f, "InconsistentLeadingWhitespaceChars")
            }
            Self::InconsistentLeadingWhitespaceCount => {
                write!(f, "InconsistentLeadingWhitespaceCount")
            }
        }
    }
}

impl std::error::Error for TokenError {}
