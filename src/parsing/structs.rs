use std::fmt;

/// Basic math operators
#[derive(Debug, PartialEq, Clone)]
pub enum ArithmeticToken {
    /// A left square bracket.
    OpenBracket,
    /// A right square bracket.
    CloseBracket,
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

/// A subset of the RelationalOperator enum. For usage with RawToken,
/// at which point an angle bracket is not guaranteed to be one type
/// of operator or another.
#[derive(Clone, Debug, PartialEq)]
pub enum CompositeRelationalOperator {
    EqualTo,
    GreaterThanEqual,
    LessThanEqual,
    NotEqual,
}

impl Into<RelationalOperator> for &CompositeRelationalOperator {
    fn into(self) -> RelationalOperator {
        match self {
            CompositeRelationalOperator::EqualTo => RelationalOperator::EqualTo,
            CompositeRelationalOperator::GreaterThanEqual => RelationalOperator::GreaterThanEqual,
            CompositeRelationalOperator::LessThanEqual => RelationalOperator::LessThanEqual,
            CompositeRelationalOperator::NotEqual => RelationalOperator::NotEqual,
        }
    }
}

/// Each relational operator has an XML tag corresponding to its
/// abbreviation, and takes an operator as its argument
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RelationalOperator {
    EqualTo,
    GreaterThan,
    GreaterThanEqual,
    LessThan,
    LessThanEqual,
    NotEqual,
}

impl RelationalOperator {
    fn abbr(&self) -> &'static str {
        match self {
            RelationalOperator::EqualTo => "et",
            RelationalOperator::GreaterThan => "gt",
            RelationalOperator::GreaterThanEqual => "gte",
            RelationalOperator::LessThan => "lt",
            RelationalOperator::LessThanEqual => "lte",
            RelationalOperator::NotEqual => "ne",
        }
    }
}

/// A single unit from a line.
#[derive(Debug, Clone)]
pub enum RawToken {
    // Literals
    Equals,
    LeftAngle,
    RightAngle,
    Bang,
    Period,
    Colon,
    /// Relational operators with two characters in length
    Relational(CompositeRelationalOperator),
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
    /// A `key="value"` phrase
    Attribute { key: String, val: String },
    /// A `src.trait` phrase
    Trait { src: String, r#trait: String },
    /// A basic number
    Int(u16),
    /// An uppercase semicolon
    Colon,
    /// A binary arithmetic operator
    Arithmetic(ArithmeticToken),
    /// A binary relational operator
    Relational(RelationalOperator),
    /// Data that couldn't be parsed as any other type
    Raw(String),
}

impl TryFrom<RawToken> for Token {
    type Error = crate::errors::TokenUnitConversionError;
    /// Attempts to convert a RawToken to a Token.
    /// Does not work for certain types or if
    fn try_from(value: RawToken) -> Result<Self, Self::Error> {
        use crate::errors::TokenUnitConversionError::*;
        use RelationalOperator::*;
        use Token::*;
        let resp: Self = match value {
            RawToken::Equals => return Err(NotSupported(value)),
            RawToken::Period => return Err(NotSupported(value)),
            RawToken::Bang => return Err(ToDo(value)),
            RawToken::LeftAngle => Relational(LessThan),
            RawToken::RightAngle => Relational(GreaterThan),
            RawToken::Colon => Colon,
            RawToken::Arithmetic(op) => Arithmetic(op),
            RawToken::Relational(r) => Relational((&r).into()),
            RawToken::String(s) => match s.trim().parse::<u16>() {
                Ok(n) => Int(n),
                Err(_) => Raw(s),
            },
        };
        Ok(resp)
    }
}

#[derive(Debug, PartialEq)]
pub struct Line {
    /// Total whitespace characters leading into a string
    pub(crate) leading_whitespace: u8,
    /// Recognized tokens in a string
    pub(crate) tokens: Vec<Token>,
}

/*
    The following std::fmt::Display implementations attempt to display
    internal token structs in the .nox format.
*/

impl fmt::Display for ArithmeticToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ArithmeticToken::OpenBracket => "[",
                ArithmeticToken::CloseBracket => "]",
                ArithmeticToken::Div => "/",
                ArithmeticToken::Mult => "*",
                ArithmeticToken::Sub => "-",
                ArithmeticToken::Add => "+",
                ArithmeticToken::Mod => "%",
            }
        )
    }
}

impl fmt::Display for RelationalOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RelationalOperator::EqualTo => "==",
                RelationalOperator::GreaterThan => ">",
                RelationalOperator::GreaterThanEqual => ">=",
                RelationalOperator::LessThan => "<",
                RelationalOperator::LessThanEqual => "<=",
                RelationalOperator::NotEqual => "!=",
            }
        )
    }
}

impl fmt::Display for RawToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equals => write!(f, "="),
            Self::LeftAngle => write!(f, "<"),
            Self::RightAngle => write!(f, ">"),
            Self::Bang => write!(f, "!"),
            Self::Period => write!(f, "."),
            Self::Colon => write!(f, ":"),
            Self::Relational(r) => write!(f, "{}", &RelationalOperator::from(r.into())),
            Self::Arithmetic(a) => write!(f, "{}", a),
            Self::String(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Token::Attribute { key, val } => write!(f, "{}=\"{}\"", key, val),
            Token::Trait { src, r#trait } => write!(f, "{}.{}", src, r#trait),
            Token::Int(i) => write!(f, "{}", i),
            Token::Colon => write!(f, ":"),
            Token::Arithmetic(op) => write!(f, "{}", op),
            Token::Raw(s) => write!(f, "{}", s),
            Token::Relational(r) => write!(f, "{}", r),
        }
    }
}

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
