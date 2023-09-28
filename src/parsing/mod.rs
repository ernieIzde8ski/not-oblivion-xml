mod structs;

use crate::errors::Maybe;
use std::vec::Vec;

pub(crate) use structs::{ArithmeticToken, Line, RawToken, Token};

/// Converts a vector of RawToken to a vector of Token.
fn to_token_vec(arr: Vec<RawToken>) -> Maybe<Vec<Token>> {
    use Maybe::{Err, Not};
    use RawToken as RT;
    use Token as T;

    let len = arr.len();
    if len == 0 {
        return Not;
    };

    let mut resp: Vec<Token> = vec![];

    let mut current_token = &arr[0];
    match current_token {
        RT::String(_) => (),
        _ => return Err("first value must be a raw string!".to_string()),
    };

    let mut i: usize = 1;
    let mut next_token;

    'shortcircuit_loop: {
        /// Increments the `i` variable safely.
        /// If `i` cannot be incremented, and a message is provided,
        /// error with that message. If no message is provided, break
        /// away from the entire loop.
        macro_rules! checked_increment_assign {
            ($($emsg:expr)?) => {{
                i += 1;
                if i < len {
                    next_token = &arr[i]
                } else {
                    $(return Err($emsg.to_string());)?
                    #[allow(unreachable_code)] {break 'shortcircuit_loop;}
                };
            }};
        }

        while i < len {
            next_token = &arr[i];
            match next_token {
                // since attributes and traits are dependent on "what comes next",
                // they get special operations
                RT::Equals => {
                    checked_increment_assign!("expected token after attribute operator");
                    match (current_token, next_token) {
                        (RT::String(a), RT::String(b)) => resp.push(T::Attribute {
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
                        (RT::String(a), RT::String(b)) => resp.push(T::Trait {
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
                // for colons and strings in the next pattern, they are irrelevant
                RT::Colon | RT::String(_) | RT::Arithmetic(_) => {
                    let token = match current_token {
                        RT::String(s) => {
                            if let Ok(n) = s.trim().parse::<u16>() {
                                T::Int(n)
                            } else {
                                T::Raw(s.clone())
                            }
                        }
                        RT::Arithmetic(op) => T::Arithmetic(op.clone()),
                        RT::Colon => T::Colon,
                        RT::Period | RT::Equals => panic!(),
                    };
                    resp.push(token);
                }
            };

            current_token = next_token;
            i += 1;
        }

        // push last member
        let token = match current_token {
            RT::String(s) => {
                if let Ok(n) = s.trim().parse::<u16>() {
                    T::Int(n)
                } else {
                    T::Raw(s.clone())
                }
            }
            RT::Colon => T::Colon,
            RT::Equals | RT::Period => {
                return Err(
                    "last token must be a primitive! (String, Int, Arithmetic, or Colon)"
                        .to_string(),
                )
            }
            RT::Arithmetic(_) => todo!(),
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
_CLOSE_BRACKET, '[',
_OPEN_BRACKET, ']',
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
            ($($extra:expr)*) => {
                if buf.len() > 0 {
                    raw_tokens.push(RawToken::String(buf));
                    $(raw_tokens.push($extra);)*
                    #[allow(unused_assignments)] { buf = String::new() };
                } else {
                    $(raw_tokens.push($extra);)*
                }
            };
        }

        loop {
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
                _EQUALS_SIGN => flush_buf!(RawToken::Equals),
                _CLOSE_BRACKET => flush_buf!(RawToken::Arithmetic(AT::CloseBracket)),
                _OPEN_BRACKET => flush_buf!(RawToken::Arithmetic(AT::OpenBracket)),
                _DIV => flush_buf!(RawToken::Arithmetic(AT::Div)),
                _MULT => flush_buf!(RawToken::Arithmetic(AT::Mult)),
                _SUB => flush_buf!(RawToken::Arithmetic(AT::Sub)),
                _ADD => flush_buf!(RawToken::Arithmetic(AT::Add)),
                _MOD => flush_buf!(RawToken::Arithmetic(AT::Mod)),
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
