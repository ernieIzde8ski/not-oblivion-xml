use core::fmt;

use crate::{
    errors::{ExprConversionFailure, LineConversionFailure, TokenConversionFailure},
    parsing::RelationalOperator,
};

use super::{Expr, Token};
#[cfg(debug_assertions)]
use crate::debug;

/// A single line.
/// Usually should be either Line<Token> or Line<Expr>
#[derive(Debug, PartialEq)]
pub struct Line<T> {
    /// Total whitespace characters leading into a string
    pub(crate) total_whitespace: u8,
    /// Recognized tokens in a string
    pub(crate) members: Vec<T>,
}

pub(crate) type TokenLine = Line<Token>;
pub type ExprLine = Line<Expr>;

impl TryFrom<&str> for TokenLine {
    type Error = TokenConversionFailure;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use super::char_literals as CH;
        use super::ArithmeticOperator as AT;
        use TokenConversionFailure::*;

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
                    return Err($err);
                })
            };
        }

        let mut ch = next_ch_or!(NoTokensPresent);

        // loop over the first couple characters and check for whitespace total/consistency
        let whitespace_char = ch;
        let mut leading_whitespace: u8 = 0;
        while ch.is_whitespace() {
            if ch != whitespace_char {
                return Err(InconsistentWhitespace);
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

impl TryFrom<TokenLine> for ExprLine {
    type Error = ExprConversionFailure;

    fn try_from(value: TokenLine) -> Result<Self, Self::Error> {
        // No star-import over Token to avoid clashes with Expr::*,
        // and to avoid accidental globs in the future when pattern
        // matching over it
        use Expr::*;
        use ExprConversionFailure::*;
        use RelationalOperator::*;

        let mut resp = Self {
            total_whitespace: value.total_whitespace,
            members: vec![],
        };

        let mut tokens = value.members.into_iter();
        let mut token = match tokens.next() {
            Some(t) => t,
            // If there are no tokens in the vec, we can just return
            // an empty response
            None => return Ok(resp),
        };

        'expr_loop: loop {
            let resp = &mut resp.members;

            macro_rules! err {
                ($e:expr) => {{
                    let err = $e;
                    #[cfg(debug_assertions)]
                    {
                        debug!("Returning error: {err:?}")
                    }
                    return Err(err);
                }};
            }

            macro_rules! push {
                () => {
                    push!(token);
                };
                ($expr:expr) => {
                    let push_res = $expr;
                    #[cfg(debug_assertions)]
                    {
                        debug!("Pushing expression: {:?}", push_res);
                    }
                    resp.push(push_res);
                };
            }

            let expr = match token {
                Token::Equals | Token::Period => {
                    err!(InvalidToken(
                        token.to_owned(),
                        "Incorrect token to start expression".into(),
                    ))
                }
                Token::Bang => err!(ToDo(token.to_owned())),
                Token::Colon => Colon,
                Token::Relational(op) => Relational((&op).into()),
                Token::Arithmetic(t) => Arithmetic(t),
                Token::LeftAngle => Relational(LessThan),
                Token::RightAngle => Relational(GreaterThan),
                Token::String(s) => match tokens.next() {
                    // handling for name='attr' expressions
                    Some(Token::Equals) => {
                        let val = match tokens.next() {
                            Some(Token::String(s)) => s,
                            Some(t) => {
                                err!(InvalidToken(t, "expected string after equals sign".into()))
                            }
                            None => err!(UnexpectedLastToken(Token::String(s), "string".into())),
                        };
                        Attribute { key: s, val }
                    }
                    // with no subsequent token
                    None => match s.parse() {
                        Ok(n) => Int(n),
                        Err(_) => Raw(s),
                    },
                    // in the case that the next token is just some irrelevant
                    // token, we will handle the first token before trying again
                    // with parsing the second
                    Some(t) => {
                        push!(match s.parse() {
                            Ok(n) => Int(n),
                            Err(_) => Raw(s),
                        });
                        token = t;
                        continue 'expr_loop;
                    }
                },

                Token::Dollar => {
                    // the object or selector
                    let src: String = match tokens.next() {
                        Some(Token::String(s)) => s,
                        Some(t) => err!(InvalidToken(t, "expected a string".into())),
                        None => err!(UnexpectedLastToken(token, "string literal".into())),
                    };

                    // argument for the selector or None for the object
                    let arg: Option<String> = match tokens.next() {
                        Some(Token::LeftAngle) => match tokens.next() {
                            // case: $sel<...>.trait
                            Some(Token::String(s)) => match tokens.next() {
                                Some(Token::RightAngle) => match tokens.next() {
                                    Some(Token::Period) => Some(s),
                                    Some(t) => {
                                        err!(InvalidToken(t, "expected a period".into()))
                                    }
                                    None => err!(InvalidToken(token, "period".into())),
                                },
                                Some(t) => {
                                    err!(InvalidToken(t, "expected a right angle bracket".into()))
                                }
                                None => {
                                    err!(UnexpectedLastToken(token, "right angle bracket".into()))
                                }
                            },
                            // case: $sel<>.trait
                            Some(Token::RightAngle) => match tokens.next() {
                                Some(Token::Period) => Some(String::new()),
                                Some(t) => err!(InvalidToken(t, "expected a period".into())),
                                None => err!(UnexpectedLastToken(token, "period".into())),
                            },
                            Some(t) => {
                                err!(InvalidToken(
                                    t,
                                    "expected a string or right angle bracket".into(),
                                ))
                            }
                            None => {
                                err!(UnexpectedLastToken(
                                    token,
                                    "string or right angle bracket".into(),
                                ))
                            }
                        },
                        Some(Token::Period) => None,
                        Some(t) => err!(InvalidToken(t, "expected a period".into())),
                        None => err!(UnexpectedLastToken(token, "period".into())),
                    };

                    // name of trait for the trait-tag
                    let r#trait: String = match tokens.next() {
                        Some(Token::String(s)) => s,
                        Some(t) => err!(InvalidToken(t, "expected a string".into())),
                        None => err!(UnexpectedLastToken(token, "string".into())),
                    };

                    Expr::Trait { src, arg, r#trait }
                }
            };

            push!(expr);
            match tokens.next() {
                Some(t) => token = t,
                None => break 'expr_loop,
            };
        }

        Ok(resp)
    }
}

impl TryFrom<&str> for ExprLine {
    type Error = LineConversionFailure;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use LineConversionFailure as Error;

        let line = match TokenLine::try_from(value) {
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
