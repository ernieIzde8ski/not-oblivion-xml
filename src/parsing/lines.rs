use core::fmt;

use crate::errors::{ExprConversionFailure, LineConversionFailure, TokenConversionFailure};

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
        use super::ArithmeticToken as AT;
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
                        #[cfg(debug_assertions)] debug!("RT: push String: {:?}", buf);
                        raw_tokens.push(Token::String(buf));
                        #[allow(unused_assignments)] { buf = String::new() };
                    }
                    $(
                        #[cfg(debug_assertions)] debug!("RT: push: {:?}", $arg);
                        raw_tokens.push($arg);
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
                match ch {
                    // Escape next character
                    CH::BACKSLASH => {
                        ch = next_ch_or!(UnexpectedEol("char after backslash"));
                        write_buf!("{}", ch);
                    }
                    // Treat as comment
                    CH::COMMENT => break,
                    // Act as delimiter
                    CH::SPACE => flush_buf!(),
                    // Mark the end of a tag, and allow in-lining afterwards
                    CH::COLON => flush_buf!(Token::Colon),
                    // `me().attr` expressions
                    CH::PERIOD => flush_buf!(Token::Period),
                    // `key="value"` expressions
                    CH::RIGHT_SQUARE => flush_buf!(Token::Arithmetic(AT::CloseBracket)),
                    CH::LEFT_SQUARE => flush_buf!(Token::Arithmetic(AT::OpenBracket)),
                    CH::FORWARD_SLASH => flush_buf!(Token::Arithmetic(AT::Div)),
                    CH::ASTERISK => flush_buf!(Token::Arithmetic(AT::Mult)),
                    CH::MINUS => flush_buf!(Token::Arithmetic(AT::Sub)),
                    CH::PLUS => flush_buf!(Token::Arithmetic(AT::Add)),
                    CH::PERCENTAGE => flush_buf!(Token::Arithmetic(AT::Mod)),
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

        let mut resp = Self {
            total_whitespace: value.total_whitespace,
            members: vec![],
        };

        let arr = &mut resp.members;

        let len = value.members.len();
        if len == 0 {
            return Ok(resp);
        };

        // Current token, next token
        let mut token = &value.members[0];
        let mut next;
        let mut i: usize = 1;

        'shortcircuit_loop: {
            /// Increments the `i` and `next` variables safely.
            ///
            /// If `i` cannot be incremented, and a message is provided,
            /// error with that message. If a lifetime is provided instead,
            /// break that lifetime. If nothing is provided, default to the
            /// 'shortcircuit_loop lifetime. Finally, if another expression
            /// is provided, evaluate that one.
            macro_rules! checked_increment_assign {
                () => {checked_increment_assign!('shortcircuit_loop)};
                ($id:lifetime) => {checked_increment_assign!({break $id})};

                // ($token:expr, $m:literal) => {checked_increment_assign!(return Err(UnexpectedLastToken(token.clone(), $m.into())));};


                ($action:block) => {{
                    i += 1;
                    if i < len {
                        next = &value.members[i]
                    } else {
                        $action
                    };
                }};

                ($err:expr) => {
                    checked_increment_assign!( { return Err($err); })
                };
            }

            while i < len {
                next = &value.members[i];
                match next {
                    // since attributes and traits are dependent on "what comes next",
                    // they get special operations
                    Token::Equals => {
                        // only parsing attributes here!
                        // relational operators have been parsed already
                        checked_increment_assign!(UnexpectedLastToken(
                            token.clone(),
                            "token after attribute operator".into()
                        ));
                        match (token, next) {
                            (Token::String(a), Token::String(b)) => arr.push(Attribute {
                                key: a.to_string(),
                                val: b.to_string(),
                            }),
                            _ => {
                                return Err(InvalidToken(
                                    token.clone(),
                                    "Both values to an attribute operator must be strings!".into(),
                                ))
                            }
                        };
                        checked_increment_assign!();
                    }
                    Token::Period => {
                        checked_increment_assign!(UnexpectedLastToken(
                            token.clone(),
                            "token after period".into()
                        ));
                        match (token, next) {
                            (Token::String(a), Token::String(b)) => arr.push(Trait {
                                src: a.to_string(),
                                r#trait: b.to_string(),
                            }),
                            _ => {
                                return Err(InvalidToken(
                                    token.clone(),
                                    "Both values to an attribute operator must be strings!".into(),
                                ))
                            }
                        };
                        checked_increment_assign!();
                    }
                    // All these remaining types are matched as singletons.
                    // The pattern is not a `_`, so that the compiler generates
                    // errors when new variants are implemented on RawToken.
                    Token::Colon
                    | Token::String(_)
                    | Token::Arithmetic(_)
                    | Token::Bang
                    | Token::LeftAngle
                    | Token::RightAngle
                    | Token::Relational(_) => {
                        // TODO: change `match` to `.unwrap` after
                        // implementing NOT operator. Token::{Period, Equals}
                        // cases should be matched by matches on `next_token`
                        arr.push(Expr::try_from(token.clone())?);
                    }
                };

                token = next;
                i += 1;
            }

            // push last member
            let token = match Expr::try_from(token.clone()) {
                Ok(t) => t,
                Result::Err(e) => return Err(e),
            };
            arr.push(token);
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
