use std::fmt;

/// Basic math operators
#[derive(Debug, PartialEq, Clone)]
pub enum ArithmeticOperator {
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
pub enum RelationalOperator {
    EqualTo,
    GreaterThan,
    GreaterThanEqual,
    LessThan,
    LessThanEqual,
    NotEqual,
}

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
    /// Relational operators with two characters in length
    Relational(CompositeRelationalOperator),
    /// Basic binary (mostly) operators.
    Arithmetic(ArithmeticOperator),
    /// A string that couldn't be parsed as any other symbol.
    String(String),
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
    Attribute { key: String, val: String },
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
    /// A binary arithmetic operator
    Arithmetic(ArithmeticOperator),
    /// A binary relational operator
    Relational(RelationalOperator),
    /// Data that couldn't be parsed as any other type
    Raw(String),
}

/*
    The following std::fmt::Display implementations attempt to display
    internal token structs in the .nox format.
*/

impl fmt::Display for ArithmeticOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::OpenBracket => "[",
                Self::CloseBracket => "]",
                Self::Div => "/",
                Self::Mult => "*",
                Self::Sub => "-",
                Self::Add => "+",
                Self::Mod => "%",
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
            Self::Relational(r) => write!(f, "{}", &RelationalOperator::from(r.into())),
            Self::Arithmetic(a) => write!(f, "{}", a),
            Self::String(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Expr::Attribute { key, val } => write!(f, "{}=\"{}\"", key, val),
            Expr::Trait {
                src,
                arg: Some(arg),
                r#trait,
            } => write!(f, "${src}<{arg}>.{trait}"),
            Expr::Trait { src, r#trait, .. } => write!(f, "{}.{}", src, r#trait),
            Expr::Int(i) => write!(f, "{}", i),
            Expr::Colon => write!(f, ":"),
            Expr::Arithmetic(op) => write!(f, "{}", op),
            Expr::Raw(s) => write!(f, "{}", s),
            Expr::Relational(r) => write!(f, "{}", r),
        }
    }
}
