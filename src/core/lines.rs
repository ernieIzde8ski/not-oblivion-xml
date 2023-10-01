use core::fmt;

use super::errors::LineConversionFailure;

use super::{ExprLine, Token};

/// A single line.
/// Usually should be either Line<Token> or Line<Expr>
#[derive(Debug, PartialEq)]
pub struct Line<T> {
    /// Total whitespace characters leading into a string
    pub(crate) total_whitespace: u8,
    /// Recognized tokens in a string
    pub(crate) members: Vec<T>,
}

impl TryFrom<&str> for ExprLine {
    type Error = LineConversionFailure;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use LineConversionFailure as Error;

        let line = match Line::<Token>::try_from(value) {
            Ok(l) => l,
            Err(e) => return Err(Error::TokenFailure(e)),
        };

        match ExprLine::try_from(line) {
            Ok(l) => Ok(l),
            Err(e) => Err(Error::ExprFailure(e)),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Line<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..(self.total_whitespace) {
            write!(f, " ")?
        }
        let len = self.members.len();
        if len == 0 {
            return Ok(());
        };
        for i in 0..(len - 1) {
            write!(f, "{} ", self.members[i])?
        }
        write!(f, "{}", self.members[len - 1])
    }
}
