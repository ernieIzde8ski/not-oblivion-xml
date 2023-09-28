use std::fmt;

/// Basic math operators
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ArithmeticToken {
    /// A left square bracket.
    CloseBracket,
    /// A right square bracket.
    OpenBracket,
    /// A forward slash.
    Div,
    /// An asterisk.
    Mult,
    /// A minus symbol.
    Sub,
    /// A plus symbol.
    Add,
    /// A percentage sign.
    Mod,
}

impl fmt::Display for ArithmeticToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ArithmeticToken::CloseBracket => "[",
                ArithmeticToken::OpenBracket => "]",
                ArithmeticToken::Div => "/",
                ArithmeticToken::Mult => "*",
                ArithmeticToken::Sub => "-",
                ArithmeticToken::Add => "+",
                ArithmeticToken::Mod => "%",
            }
        )
    }
}

/// A single unit from a line.
#[derive(Debug)]
pub(crate) enum RawToken {
    /// An equals sign.
    Equals,
    /// A period.
    Period,
    /// An uppercase semicolon.
    Colon,

    /// Basic binary (mostly) operators.
    Arithmetic(ArithmeticToken),

    /// A string that couldn't be parsed as any other symbol.
    String(String),
}

/// A space/quote-separated member.
///
/// Only very basic parsing should be done at the RawToken -> Token stage.
/// for example:
/// - yes: parsing complex tokens which contain primitives, like Attribute
/// - no:  nesting tokens inside of parentheses
#[derive(Debug, PartialEq)]
pub(crate) enum Token {
    /// represents a `key="value"` phrase
    Attribute { key: String, val: String },
    /// represents a `src.trait` phrase
    Trait { src: String, r#trait: String },
    /// represents a basic number
    Int(u16),
    /// represents an uppercase semicolon
    Colon,
    /// represents one of the binary arithmetic operators
    Arithmetic(ArithmeticToken),
    /// Data that couldn't be parsed as any other type
    Raw(String),
}

/// Displays a token in the same format as it is read
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Token::Attribute { key, val } => write!(f, "{}=\"{}\"", key, val),
            Token::Trait { src, r#trait } => write!(f, "{}.{}", src, r#trait),
            Token::Int(i) => write!(f, "{}", i),
            Token::Colon => write!(f, ":"),
            Token::Arithmetic(op) => write!(f, "{}", op),
            Token::Raw(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Line {
    /// Total whitespace characters leading into a string
    pub(crate) leading_whitespace: u8,
    /// Recognized tokens in a string
    pub(crate) tokens: Vec<Token>,
}

/// Displays a list of tokens in a similar format as how it is read
impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..(self.leading_whitespace) {
            write!(f, " ")?
        }
        let len = self.tokens.len();
        if len == 0 {
            return Ok(());
        };
        for i in 0..(len - 1) {
            write!(f, "{} ", self.tokens[i])?
        }
        write!(f, "{}", self.tokens[len - 1])
    }
}
