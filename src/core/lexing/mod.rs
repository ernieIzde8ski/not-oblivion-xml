mod char_literals;

use super::ArithmeticOperator as AT;
use super::{errors::TokenConversionFailure, Line, Token};
#[cfg(debug_assertions)]
use crate::debug;
use crate::err;
use char_literals as CH;
use TokenConversionFailure::*;

impl TryFrom<&str> for Line<Token> {
    type Error = TokenConversionFailure;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // strip ending whitespace
        let mut line = value.trim_end().chars();

        /// gets the next value of `ch` OR executes the block statement.
        ///
        /// If given any expression other than a block, it is assumed to
        /// be a LineConversionError variant, and the function exits early.
        macro_rules! next_ch_or {
            ($s:block) => {
                match line.next() {
                    Some(ch) => ch,
                    None => $s,
                }
            };
            ($err:expr) => {
                next_ch_or!({
                    err!($err);
                })
            };
        }

        let mut ch = next_ch_or!(NoTokensPresent);

        // loop over the first couple characters and check for whitespace total/consistency
        let whitespace_char = ch;
        let mut leading_whitespace: u8 = 0;
        while ch.is_whitespace() {
            if ch != whitespace_char {
                err!(InconsistentWhitespace);
            }
            leading_whitespace += 1;
            ch = next_ch_or!(NoTokensPresent);
        }

        // do work now that the first non-whitespace character is known
        let tokens = {
            use std::fmt::Write;
            let mut raw_tokens: Vec<Token> = vec![];
            let mut buf: String = String::new();

            macro_rules! write_buf {
                ($($arg:tt)*) => {
                    // pretty sure this shouldn´t panic but we´ll see
                    write!(buf, $($arg)*).expect("writing to buffer")
                };
            }
            macro_rules! flush_buf {
                ($($arg:expr)*) => {{
                    if buf.len() > 0 {
                        #[cfg(debug_assertions)] debug!("Pushing token: String({:?})", buf);
                        raw_tokens.push(Token::String(buf));
                        #[allow(unused_assignments)] { buf = String::new() };
                    }
                    $(
                        // avoids ownership & cloning issues by computing the value
                        // before subsequent usage
                        let arg = $arg;
                        #[cfg(debug_assertions)] debug!("Pushing token: {:?}", arg);
                        raw_tokens.push(arg);
                    )*

                }};
            }

            'outer: loop {
                use super::structs::CompositeRelationalOperator as Relational;

                /// Defines a token that may take up two characters.
                macro_rules! composite_token {
                    ($default:expr, $($key:pat, $type:expr)+) => {{
                        let token = 'token: {
                            let next_ch = next_ch_or!({break 'token $default});
                            match next_ch {
                                // here arise composite tokens
                                $($key => $type,)+
                                // parse backslashes as escape chars
                                CH::BACKSLASH => {
                                    flush_buf!($default);
                                    if next_ch == CH::BACKSLASH {
                                        write_buf!("\\");
                                        ch = next_ch_or!({break 'outer});
                                    } else {
                                        ch = next_ch;
                                    }
                                    continue 'outer;
                                }
                                // parse other kinds of characters as if they were normal
                                _ => {
                                    flush_buf!($default);
                                    ch = next_ch;
                                    continue 'outer;
                                }
                            }
                        };
                        flush_buf!(token);
                    }};
                }

                // Delimit at whitespace
                if ch.is_whitespace() {
                    flush_buf!();
                    ch = next_ch_or!({ break })
                };

                match ch {
                    // Escape next character
                    CH::BACKSLASH => {
                        ch = next_ch_or!(UnexpectedEol("char after backslash"));
                        write_buf!("{}", ch);
                    }
                    // Treat as comment
                    CH::COMMENT => break,
                    // Mark the end of a tag, and allow in-lining afterwards
                    CH::COLON => flush_buf!(Token::Colon),
                    // `me().attr` trait-tags
                    CH::PERIOD => flush_buf!(Token::Period),
                    CH::RIGHT_SQUARE => flush_buf!(Token::Arithmetic(AT::CloseBracket)),
                    CH::LEFT_SQUARE => flush_buf!(Token::Arithmetic(AT::OpenBracket)),
                    CH::FORWARD_SLASH => flush_buf!(Token::Arithmetic(AT::Div)),
                    CH::ASTERISK => flush_buf!(Token::Arithmetic(AT::Mult)),
                    CH::MINUS => flush_buf!(Token::Arithmetic(AT::Sub)),
                    CH::PLUS => flush_buf!(Token::Arithmetic(AT::Add)),
                    CH::PERCENTAGE => flush_buf!(Token::Arithmetic(AT::Mod)),
                    CH::DOLLAR => flush_buf!(Token::Dollar),
                    // `key="value"` attribute tags
                    CH::EQUALS_SIGN => composite_token!(
                        Token::Equals,
                        CH::EQUALS_SIGN,
                        Token::Relational(Relational::EqualTo)
                    ),
                    CH::LEFT_ANGLE => composite_token!(
                        Token::LeftAngle,
                        CH::EQUALS_SIGN,
                        Token::Relational(Relational::LessThanEqual)
                    ),
                    CH::RIGHT_ANGLE => composite_token!(
                        Token::RightAngle,
                        CH::EQUALS_SIGN,
                        Token::Relational(Relational::GreaterThanEqual)
                    ),
                    CH::BANG => composite_token!(
                        Token::Bang,
                        CH::EQUALS_SIGN,
                        Token::Relational(Relational::NotEqual)
                    ),
                    // Pause delimiting inside quote blocks
                    CH::SINGLE_QUOTE | CH::DOUBLE_QUOTE => {
                        let quote = ch;
                        loop {
                            ch = next_ch_or!(UnexpectedEol("closing quote"));
                            if ch == quote {
                                break;
                            } else if ch == CH::BACKSLASH {
                                ch = next_ch_or!(UnexpectedEol("char after backslash"));
                            };
                            write_buf!("{}", ch);
                        }
                    }
                    // Add unrecognized chars to buffer
                    other => write_buf!("{}", other),
                };

                ch = next_ch_or!({ break })
            }
            flush_buf!();

            raw_tokens
        };

        match tokens.len() {
            0 => Err(NoTokensPresent),
            _ => Ok(Line {
                total_whitespace: leading_whitespace,
                members: tokens,
            }),
        }
    }
}
