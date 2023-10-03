mod core;
#[cfg(debug_assertions)]
#[macro_use]
pub(crate) mod debug;

#[cfg(test)]
mod tests;

pub(crate) use crate::core::lexing;
pub use lexing::{Operator, Token, TokenError};

pub fn parse_string(s: &str) -> Result<Vec<Token>, TokenError> {
    let chars = s.trim_end().chars().peekable();
    lexing::parse_chars(chars)
}
