use core::fmt;
use std::error::Error;

use crate::parsing::Token;

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

/// The error result of a failed conversion from RawToken into Token.
#[derive(Debug)]
pub enum TokenUnitConversionError {
    /// Token not supported for this operation.
    NotSupported(Token),
    /// Token not yet supported.
    ToDo(Token),
}

impl ErrorEnum for TokenUnitConversionError {
    fn name(&self) -> String {
        match self {
            Self::NotSupported(_) => "NotSupported",
            Self::ToDo(_) => "ToDo",
        }
        .to_string()
    }

    fn message(&self) -> Option<String> {
        match self {
            Self::NotSupported(token) => Some(format!("token '{}' not supported", token)),
            Self::ToDo(token) => Some(format!("token '{}' not yet supported", token)),
        }
    }
}

/// The error result of a failed conversion from &str into Line.
#[derive(Debug)]
pub enum LineConversionError {
    /// No values aside from spaces/comments in a string.
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
    /// Failed attempt at converting a standalone `RawToken`.
    /// Inherited from `Token::TryFrom<RawToken>`.
    BadTokenUnit(TokenUnitConversionError),
}

impl ErrorEnum for LineConversionError {
    fn name(&self) -> String {
        match self {
            Self::NoTokensPresent => "NoTokensPresent",
            Self::InconsistentWhitespace => "InconsistentWhitespace",
            Self::UnexpectedEol(_) => "UnexpectedEol",
            Self::UnexpectedLastToken(_) => "UnexpectedLastToken",
            Self::InvalidArgument(_) => "InvalidArgument",
            Self::BadTokenUnit(t) => return format!("BadTokenUnit::{}", t.name()),
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
            Self::BadTokenUnit(t) => t.message(),
        }
    }
}
