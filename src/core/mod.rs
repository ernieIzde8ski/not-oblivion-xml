/*
The `core` module does all the effort of defining
structs and parsing them into expressions.
*/

mod errors;
mod lexing;
mod lines;
mod parsing;
mod structs;

// publicly expose types for conversion: error types, Line types
// privately expose types for doing conversion work: tokens, expressions
pub use errors::{ErrorEnum, LineConversionFailure, TokenConversionFailure};
pub use lines::Line;
pub use parsing::ExprLine;
pub(crate) use structs::{ArithmeticOperator, Expr, RelationalOperator, Token};
