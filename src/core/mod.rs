/*
The `core` module does all the effort of defining
structs and parsing them into expressions.
*/

mod errors;
pub(crate) mod lexing;

pub use lexing::{Operator, Token, TokenError};
