mod structs;

use crate::errors::Maybe;
use std::vec::Vec;

#[cfg(debug_assertions)]
use crate::debug;
pub(crate) use structs::{ArithmeticToken, Line, RawToken, RelationalOperator, Token};

/// Converts a vector of RawToken to a vector of Token.
fn to_token_vec(arr: Vec<RawToken>) -> Maybe<Vec<Token>> {
    use Maybe::{Err, Not};
    // No star-import over RawToken to avoid clashes with Token::*
    // and to avoid accidental globs in the future when pattern
    // matching over it, but still alias for convenience
    use RawToken as RT;
    use Token::*;

    let len = arr.len();
    if len == 0 {
        return Not;
    };

    let mut resp: Vec<Token> = vec![];

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
            ($m:literal) => {checked_increment_assign!(return Err($m.to_string()));};
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
                RT::Equals => {
                    // only parsing attributes here!
                    // relational operators have been parsed already
                    checked_increment_assign!("expected token after attribute operator");
                    match (current_token, next_token) {
                        (RT::String(a), RT::String(b)) => resp.push(Attribute {
                            key: a.to_string(),
                            val: b.to_string(),
                        }),
                        _ => return Err("Both values of an attribute must be strings!".to_string()),
                    };
                    checked_increment_assign!();
                }
                RT::Period => {
                    checked_increment_assign!("expected token after period");
                    match (current_token, next_token) {
                        (RT::String(a), RT::String(b)) => resp.push(Trait {
                            src: a.to_string(),
                            r#trait: b.to_string(),
                        }),
                        _ => {
                            return Err(
                                "Both values to an attribute operator must be strings!".to_string()
                            )
                        }
                    };
                    checked_increment_assign!();
                }
                // All these remaining types are matched as singletons.
                // The pattern is not a `_`, so that the compiler generates
                // errors when new variants are implemented on RawToken.
                RT::Colon
                | RT::String(_)
                | RT::Arithmetic(_)
                | RT::Bang
                | RT::LeftAngle
                | RT::RightAngle
                | RT::Relational(_) => {
                    // TODO: change `match` to `.unwrap` after
                    // implementing NOT operator. RT::{Period, Equals}
                    // cases should be matched by matches on `next_token`
                    let token = match Token::try_from(current_token.clone()) {
                        Ok(t) => t,
                        Result::Err(e) => return Err(e.to_string()),
                    };
                    resp.push(token);
                }
            };

            current_token = next_token;
            i += 1;
        }

        // push last member
        let token = match Token::try_from(current_token.clone()) {
            Ok(t) => t,
            Result::Err(e) => return Err(e.to_string()),
        };
        resp.push(token);
    }

    Maybe::Ok(resp)
}

macro_rules! __define_char_constants {
    ($($name:ident, $val:expr),*) => {
        $(const $name: char = $val;)*
    };
}

__define_char_constants! {
_POUND, '#',
_BACKSLASH, '\\',
_SPACE, ' ',
_COLON, ':',
_TRAIT_SEP,'.',
_EQUALS_SIGN, '=',
_LEFT_ANGLE, '<',
_RIGHT_ANGLE, '>',
_BANG, '!',
_OPEN_BRACKET, '[',
_CLOSE_BRACKET, ']',
_DIV, '/',
_MULT, '*',
_SUB, '-',
_ADD, '+',
_MOD, '%',
_SINGLE_QUOTE, '\'',
_DOUBLE_QUOTE, '"'
}

pub fn extract_tokens(line: &str) -> Maybe<Line> {
    use ArithmeticToken as AT;
    use Maybe::{Err, Not, Ok};

    // strip ending whitespace
    let mut line = line.trim_end().chars();
    /// gets the next value of `ch` OR
    macro_rules! next_ch_or {
        // return Err with the message when given a string literal
        ($m:literal) => {
            next_ch_or!(return Err($m.to_string()))
        };
        ($s:expr) => {
            match line.next() {
                Some(ch) => ch,
                None => $s,
            }
        };
    }
    let mut ch = next_ch_or!(return Not);

    // loop over the first couple characters and check for whitespace total/consistency
    let whitespace_char = ch;
    let mut leading_whitespace: u8 = 0;
    while ch.is_whitespace() {
        if ch != whitespace_char {
            return Err("Inconsistent usage of tabs & spaces".to_string());
        }
        leading_whitespace += 1;
        ch = next_ch_or!(return Not);
    }

    // do work now that the first non-whitespace character is known
    let tokens = {
        use std::fmt::Write;
        let mut raw_tokens: Vec<RawToken> = vec![];
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
                    raw_tokens.push(RawToken::String(buf));
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
                        let next_ch = next_ch_or!(break 'token $default);
                        match next_ch {
                            // here arise composite tokens
                            $($key => $type,)+
                            // parse backslashes as escape chars
                            _BACKSLASH => {
                                flush_buf!($default);
                                if next_ch == _BACKSLASH {
                                    write_buf!("\\");
                                    ch = next_ch_or!(break 'outer);
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
                    ch = next_ch_or!("expected a character; got EOL");
                    write_buf!("{}", ch);
                }
                // Treat as comment
                _POUND => break,
                // Act as delimiter
                _SPACE => flush_buf!(),
                // Mark the end of a tag, and allow in-lining afterwards
                _COLON => flush_buf!(RawToken::Colon),
                // `me().attr` expressions
                _TRAIT_SEP => flush_buf!(RawToken::Period),
                // `key="value"` expressions
                _CLOSE_BRACKET => flush_buf!(RawToken::Arithmetic(AT::CloseBracket)),
                _OPEN_BRACKET => flush_buf!(RawToken::Arithmetic(AT::OpenBracket)),
                _DIV => flush_buf!(RawToken::Arithmetic(AT::Div)),
                _MULT => flush_buf!(RawToken::Arithmetic(AT::Mult)),
                _SUB => flush_buf!(RawToken::Arithmetic(AT::Sub)),
                _ADD => flush_buf!(RawToken::Arithmetic(AT::Add)),
                _MOD => flush_buf!(RawToken::Arithmetic(AT::Mod)),
                _EQUALS_SIGN => composite_token!(
                    RawToken::Equals,
                    _EQUALS_SIGN,
                    RawToken::Relational(Relational::EqualTo)
                ),
                _LEFT_ANGLE => composite_token!(
                    RawToken::LeftAngle,
                    _EQUALS_SIGN,
                    RawToken::Relational(Relational::LessThanEqual)
                ),
                _RIGHT_ANGLE => composite_token!(
                    RawToken::RightAngle,
                    _EQUALS_SIGN,
                    RawToken::Relational(Relational::GreaterThanEqual)
                ),
                _BANG => composite_token!(
                    RawToken::Bang,
                    _EQUALS_SIGN,
                    RawToken::Relational(Relational::NotEqual)
                ),
                // Pause delimiting inside quote blocks
                _SINGLE_QUOTE | _DOUBLE_QUOTE => {
                    let quote = ch;
                    loop {
                        ch = next_ch_or!("expected closing quote; got EOL");
                        if ch == quote {
                            break;
                        } else if ch == _BACKSLASH {
                            ch = next_ch_or!("expected closing quote before EOL; got backslash");
                        };
                        write_buf!("{}", ch);
                    }
                }
                // Add unrecognized chars to buffer
                other => write_buf!("{}", other),
            };

            ch = next_ch_or!(break)
        }
        flush_buf!();
        to_token_vec(raw_tokens)
    };

    let tokens = match tokens {
        Ok(t) => t,
        Err(m) => return Err(m),
        Not => return Not,
    };

    Ok(Line {
        leading_whitespace,
        tokens,
    })
}
