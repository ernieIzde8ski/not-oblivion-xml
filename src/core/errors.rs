use core::fmt;
use std::error::Error;

use crate::core::Token;

/**
 * Early return from a function with a given error
 * + in dev mode, print line called from in dev mode.
 */
#[macro_export]
macro_rules! err {
    ($e:expr) => {{
        // compute value to avoid ownership issues with debug_assertions
        let err = $e;
        #[cfg(debug_assertions)]
        {
            debug!("Returning error: {err:?}")
        }
        return Err(err);
    }};
}

/**
 The ErrorEnum trait.

 Provides an interface for easily converting name-value enums
 into `std::error::Error` `impl`s.
*/

pub trait ErrorEnum: fmt::Debug {
    /// Name of an ErrorEnum variant
    fn name(&self) -> String;
    /// Message to display after variant name
    fn message(&self) -> Option<String>;
}

impl fmt::Display for dyn ErrorEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.message() {
            Some(v) => write!(f, "{}: {}", self.name(), v),
            None => write!(f, "{}", self.name()),
        }
    }
}

impl Error for dyn ErrorEnum {}

/*
    Error Types
*/

/// Failed conversion from Line<Token> into Line<Expr>.
#[derive(Debug)]
pub enum ExprConversionFailure {
    /// Expected another token, but didn't get one.
    /// String describes expected subsequent token.
    UnexpectedLastToken(Token, String),
    /// Expected a different kind of token.
    /// String describes an error message.
    InvalidToken(Token, String),
    /// Token not supported for this operation.
    NotSupported(Token),
    /// Token not yet supported.
    ToDo(Token),
}

impl ErrorEnum for ExprConversionFailure {
    fn name(&self) -> String {
        match self {
            Self::InvalidToken(..) => "InvalidToken",
            Self::UnexpectedLastToken(..) => "UnexpectedLastToken",
            Self::NotSupported(_) => "NotSupported",
            Self::ToDo(_) => "ToDo",
        }
        .to_string()
    }

    fn message(&self) -> Option<String> {
        Some(match self {
            Self::InvalidToken(t, m) => format!("{m}; got {t}"),
            Self::UnexpectedLastToken(t, m) => format!("expected {m} after token '{t}'",),
            Self::NotSupported(token) => format!("token '{token}' not supported"),
            Self::ToDo(token) => format!("token '{token}' not yet supported"),
        })
    }
}

/// Failed conversion from &str into Line<Token>.
#[derive(Debug)]
pub enum TokenConversionFailure {
    /// No values aside from whitespace/comments identified.
    NoTokensPresent,
    /// Inconsistent usage of tabs and spaces
    InconsistentWhitespace,
    /// Expected a value, but reached end of line instead.
    /// Name describes the expected character.
    UnexpectedEol(&'static str),
    /// Expected a value, but reached the last token instead.
    /// Message describes expected tokens.
    UnexpectedLastToken(&'static str),
    /// Argument(s) of invalid type.
    /// Message describes expected type.
    InvalidArgument(&'static str),
}

impl ErrorEnum for TokenConversionFailure {
    fn name(&self) -> String {
        match self {
            Self::NoTokensPresent => "NoTokensPresent",
            Self::InconsistentWhitespace => "InconsistentWhitespace",
            Self::UnexpectedEol(_) => "UnexpectedEol",
            Self::UnexpectedLastToken(_) => "UnexpectedLastToken",
            Self::InvalidArgument(_) => "InvalidArgument",
        }
        .to_string()
    }

    fn message(&self) -> Option<String> {
        match self {
            Self::NoTokensPresent => None,
            Self::InconsistentWhitespace => {
                Some("Inconsistent usage of tabs and spaces".to_string())
            }
            Self::UnexpectedEol(expected) => Some(format!("expected {}, got EOL", expected)),
            Self::UnexpectedLastToken(msg) | Self::InvalidArgument(msg) => Some(msg.to_string()),
        }
    }
}

/// Failed conversion from &str into Line<Token>.
#[derive(Debug)]
pub enum LineConversionFailure {
    ExprFailure(ExprConversionFailure),
    TokenFailure(TokenConversionFailure),
}

impl ErrorEnum for LineConversionFailure {
    fn name(&self) -> String {
        format!(
            "LineConversionFailure::{}",
            match self {
                Self::ExprFailure(e) => e.name(),
                Self::TokenFailure(e) => e.name(),
            }
        )
    }

    fn message(&self) -> Option<String> {
        match self {
            Self::ExprFailure(e) => e.message(),
            Self::TokenFailure(e) => e.message(),
        }
    }
}
