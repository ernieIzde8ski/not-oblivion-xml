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

pub fn render_tokens(
    tokens: Vec<Token>,
    buf: &mut impl std::io::Write,
) -> Result<(), std::io::Error> {
    // use std::iter::Peekable as _;
    let mut indent_level = 0;
    let tokens = &mut tokens.into_iter().peekable();
    while let Some(token) = tokens.next() {
        use Operator::*;
        use Token::*;
        match token {
            Op(NewLine) => {
                while let Some(_) = tokens.next_if(|t| matches!(t, Indent)) {
                    indent_level += 1;
                }
                while let Some(_) = tokens.next_if(|t| matches!(t, Dedent)) {
                    indent_level -= 1;
                }
                writeln!(buf)?;
                for _ in 0..indent_level {
                    write!(buf, "\t")?;
                }
            }
            t => print!("{t} "),
        }
    }

    writeln!(buf)
}
