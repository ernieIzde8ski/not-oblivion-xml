mod structs;

use crate::errors::LineConversionError;
use std::vec::Vec;

#[cfg(debug_assertions)]
use crate::debug;
// RelationalOperator is not used by this module, but it *is* used by tests.
#[cfg(debug_assertions)]
#[allow(unused_imports)]
pub(crate) use structs::RelationalOperator;
pub(crate) use structs::{ArithmeticToken, Expr, Line, Token};

/// Converts a vector of RawToken to a vector of Token.
fn to_expr_vec(arr: Vec<Token>) -> Result<Vec<Expr>, LineConversionError> {
    // No star-import over RawToken to avoid clashes with Token::*
    // and to avoid accidental globs in the future when pattern
    // matching over it, but still alias for convenience
    use Expr::*;
    use LineConversionError::*;

    let len = arr.len();
    if len == 0 {
        return Err(LineConversionError::NoTokensPresent);
    };

    let mut resp: Vec<Expr> = vec![];

    let mut current_token = &arr[0];

    let mut i: usize = 1;
    let mut next_token;

    'shortcircuit_loop: {
        /// Increments the `i` variable safely.
        ///
        /// If `i` cannot be incremented, and a message is provided,
        /// error with that message. If a lifetime is provided instead,
        /// break that lifetime. If nothing is provided, default to the
        /// 'shortcircuit_loop lifetime. Finally, if another expression
        /// is provided, evaluate that one.
        macro_rules! checked_increment_assign {
            () => {checked_increment_assign!('shortcircuit_loop)};
            ($m:literal) => {checked_increment_assign!(return Err(UnexpectedLastToken($m)));};
            ($id:lifetime) => {checked_increment_assign!(break $id)};
            ($action:expr) => {{
                i += 1;
                if i < len {
                    next_token = &arr[i]
                } else {
                    $action
                };
            }};
        }

        while i < len {
            next_token = &arr[i];
            match next_token {
                // since attributes and traits are dependent on "what comes next",
                // they get special operations
                Token::Equals => {
                    // only parsing attributes here!
                    // relational operators have been parsed already
                    checked_increment_assign!("token after attribute operator");
                    match (current_token, next_token) {
                        (Token::String(a), Token::String(b)) => resp.push(Attribute {
                            key: a.to_string(),
                            val: b.to_string(),
                        }),
                        _ => {
                            return Err(InvalidArgument(
                                "Both values to an attribute operator must be strings!",
                            ))
                        }
                    };
                    checked_increment_assign!();
                }
                Token::Period => {
                    checked_increment_assign!("token after period");
                    match (current_token, next_token) {
                        (Token::String(a), Token::String(b)) => resp.push(Trait {
                            src: a.to_string(),
                            r#trait: b.to_string(),
                        }),
                        _ => {
                            return Err(InvalidArgument(
                                "Both values to an attribute operator must be strings!",
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
                    let token = match Expr::try_from(current_token.clone()) {
                        Ok(t) => t,
                        Err(e) => return Err(BadTokenUnit(e)),
                    };
                    resp.push(token);
                }
            };

            current_token = next_token;
            i += 1;
        }

        // push last member
        let token = match Expr::try_from(current_token.clone()) {
            Ok(t) => t,
            Result::Err(e) => return Err(BadTokenUnit(e)),
        };
        resp.push(token);
    }

    Ok(resp)
}

macro_rules! __define_char_constants {
    ($($name:ident, $val:expr),*) => {
        $(const $name: char = $val;)*
    };
}

__define_char_constants! {
_COMMENT, '#',
_BACKSLASH, '\\',
_SPACE, ' ',
_COLON, ':',
_PERIOD,'.',
_EQUALS_SIGN, '=',
_LEFT_ANGLE, '<',
_RIGHT_ANGLE, '>',
_LEFT_SQUARE, '[',
_RIGHT_SQUARE, ']',
_BANG, '!',
_FORWARD_SLASH, '/',
_ASTERISK, '*',
_MINUS, '-',
_PLUS, '+',
_PERCENTAGE, '%',
_SINGLE_QUOTE, '\'',
_DOUBLE_QUOTE, '"'
}

pub fn extract_line(line: &str) -> Result<Line, LineConversionError> {
    use ArithmeticToken as AT;
    use LineConversionError::*;

    // strip ending whitespace
    let mut line = line.trim_end().chars();

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
            use structs::CompositeRelationalOperator as Relational;

            /// Defines a token that may take up two characters.
            macro_rules! composite_token {
                ($default:expr, $($key:ident, $type:expr)+) => {{
                    let token = 'token: {
                        let next_ch = next_ch_or!({break 'token $default});
                        match next_ch {
                            // here arise composite tokens
                            $($key => $type,)+
                            // parse backslashes as escape chars
                            _BACKSLASH => {
                                flush_buf!($default);
                                if next_ch == _BACKSLASH {
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
                _BACKSLASH => {
                    ch = next_ch_or!(UnexpectedEol("char after backslash"));
                    write_buf!("{}", ch);
                }
                // Treat as comment
                _COMMENT => break,
                // Act as delimiter
                _SPACE => flush_buf!(),
                // Mark the end of a tag, and allow in-lining afterwards
                _COLON => flush_buf!(Token::Colon),
                // `me().attr` expressions
                _PERIOD => flush_buf!(Token::Period),
                // `key="value"` expressions
                _RIGHT_SQUARE => flush_buf!(Token::Arithmetic(AT::CloseBracket)),
                _LEFT_SQUARE => flush_buf!(Token::Arithmetic(AT::OpenBracket)),
                _FORWARD_SLASH => flush_buf!(Token::Arithmetic(AT::Div)),
                _ASTERISK => flush_buf!(Token::Arithmetic(AT::Mult)),
                _MINUS => flush_buf!(Token::Arithmetic(AT::Sub)),
                _PLUS => flush_buf!(Token::Arithmetic(AT::Add)),
                _PERCENTAGE => flush_buf!(Token::Arithmetic(AT::Mod)),
                _EQUALS_SIGN => composite_token!(
                    Token::Equals,
                    _EQUALS_SIGN,
                    Token::Relational(Relational::EqualTo)
                ),
                _LEFT_ANGLE => composite_token!(
                    Token::LeftAngle,
                    _EQUALS_SIGN,
                    Token::Relational(Relational::LessThanEqual)
                ),
                _RIGHT_ANGLE => composite_token!(
                    Token::RightAngle,
                    _EQUALS_SIGN,
                    Token::Relational(Relational::GreaterThanEqual)
                ),
                _BANG => composite_token!(
                    Token::Bang,
                    _EQUALS_SIGN,
                    Token::Relational(Relational::NotEqual)
                ),
                // Pause delimiting inside quote blocks
                _SINGLE_QUOTE | _DOUBLE_QUOTE => {
                    let quote = ch;
                    loop {
                        ch = next_ch_or!(UnexpectedEol("closing quote"));
                        if ch == quote {
                            break;
                        } else if ch == _BACKSLASH {
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
        to_expr_vec(raw_tokens)
    };

    Ok(Line {
        leading_whitespace,
        tokens: tokens?,
    })
}
