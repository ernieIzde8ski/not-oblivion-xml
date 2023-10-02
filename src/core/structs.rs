use std::fmt;

/// A single unit from a line.
#[derive(Debug, Clone)]
pub enum Token {
    // Literals
    Equals,
    LeftAngle,
    RightAngle,
    Bang,
    Period,
    Colon,
    Dollar,

    // One-char math literals
    OpenBracket,
    CloseBracket,
    Div,
    Mult,
    Sub,
    Add,
    Mod,

    // Two-char math literals
    EqualTo,
    GreaterThanEqual,
    LessThanEqual,
    NotEqual,

    /// A string that couldn't be parsed as any other symbol.
    Literal(String),
}

/// A space/quote-separated member.
///
/// Only very basic parsing should be done at the Token -> Expr stage.
/// for example:
/// - yes: parsing complex tokens which contain primitives, like Attribute
/// - no:  nesting tokens inside of parentheses
#[derive(Debug, PartialEq)]
pub enum Expr {
    /// A `key="value"` phrase
    Attribute {
        key: String,
        val: String,
    },
    /// A `src.trait` phrase
    Trait {
        src: String,
        arg: Option<String>,
        r#trait: String,
    },
    /// A basic number
    Int(u16),
    /// An uppercase semicolon
    Colon,

    OpenBracket,
    Div,
    Mult,
    Sub,
    Add,
    Mod,
    CloseBracket,

    EqualTo,
    GreaterThan,
    GreaterThanEqual,
    LessThan,
    LessThanEqual,
    NotEqual,

    /// Data that couldn't be parsed as any other type
    Raw(String),
}

/*
    The following std::fmt::Display implementations attempt to display
    internal token structs in the .nox format.
*/

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equals => write!(f, "="),
            Self::LeftAngle => write!(f, "<"),
            Self::RightAngle => write!(f, ">"),
            Self::Bang => write!(f, "!"),
            Self::Period => write!(f, "."),
            Self::Colon => write!(f, ":"),
            Self::Dollar => write!(f, "$"),

            Self::OpenBracket => write!(f, "("),
            Self::Div => write!(f, "/"),
            Self::Mult => write!(f, "*"),
            Self::Sub => write!(f, "-"),
            Self::Add => write!(f, "+"),
            Self::Mod => write!(f, "%"),
            Self::CloseBracket => write!(f, ")"),

            Self::EqualTo => write!(f, "=="),
            Self::GreaterThanEqual => write!(f, ">="),
            Self::LessThanEqual => write!(f, "<="),
            Self::NotEqual => write!(f, "!="),

            Self::Literal(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Attribute { key, val } => write!(f, "{}=\"{}\"", key, val),
            Self::Trait {
                src,
                arg: Some(arg),
                r#trait,
            } => write!(f, "${src}<{arg}>.{trait}"),
            Self::Trait { src, r#trait, .. } => write!(f, "{}.{}", src, r#trait),
            Self::Int(i) => write!(f, "{}", i),
            Self::Colon => write!(f, ":"),

            Self::OpenBracket => write!(f, "("),
            Self::Div => write!(f, "/"),
            Self::Mult => write!(f, "*"),
            Self::Sub => write!(f, "-"),
            Self::Add => write!(f, "+"),
            Self::Mod => write!(f, "%"),
            Self::CloseBracket => write!(f, ")"),

            Self::EqualTo => write!(f, "=="),
            Self::GreaterThan => write!(f, ">"),
            Self::GreaterThanEqual => write!(f, ">="),
            Self::LessThan => write!(f, "<"),
            Self::LessThanEqual => write!(f, "<="),
            Self::NotEqual => write!(f, "!="),

            Self::Raw(s) => write!(f, "{}", s),
        }
    }
}
