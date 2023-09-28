use crate::errors::Maybe;
use std::{fmt::Display, vec::Vec};

/// A single unit from a line.
#[derive(Debug)]
enum RawToken {
    /// An equals sign.
    Equals,
    /// A period.
    Period,
    /// An uppercase semicolon.
    Colon,
    /// A string that couldn't be parsed as any other symbol.
    String(String),
}

/// A space/quote-separated member.
#[derive(Debug, PartialEq)]
pub(crate) enum Token {
    /// represents a `key="value"` phrase
    Attribute { key: String, val: String },
    /// represents a `src.trait` phrase
    Trait { src: String, r#trait: String },
    /// represents a basic number
    Int(u16),
    /// represents an uppercase semicolon
    Colon,
    /// Data that couldn't be parsed as any other type
    Raw(String),
}

/// Displays a token in the same format as it is read
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Token::Attribute { key, val } => write!(f, "{}=\"{}\"", key, val),
            Token::Trait { src, r#trait } => write!(f, "{}.{}", src, r#trait),
            Token::Int(i) => write!(f, "{}", i),
            Token::Colon => write!(f, ":"),
            Token::Raw(s) => write!(f, "{}", s),
        }
    }
}

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
                        _ => {
                            return Err(
                                "Both values of an attribute must be strings!".to_string()
                            )
                        }
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
                RT::Colon | RT::String(_) => {
                    let token = match current_token {
                        RT::String(s) => {
                            if let Ok(n) = s.trim().parse::<u16>() {
                                T::Int(n)
                            } else {
                                T::Raw(s.clone())
                            }
                        }
                        RT::Colon => T::Colon,
                        _ => panic!(),
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
                return Err("last token must be a primitive! (String, Int, or Colon)".to_string())
            }
        };
        resp.push(token);
    }

    Maybe::Ok(resp)
}

#[derive(Debug, PartialEq)]
pub struct Line {
    /// Total whitespace characters leading into a string
    pub(crate) leading_whitespace: u8,
    /// Recognized tokens in a string
    pub(crate) tokens: Vec<Token>,
}

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..(self.leading_whitespace) {
            write!(f, " ")?
        }
        let len = self.tokens.len();
        if len == 0 {
            return Ok(());
        };
        for i in 0..(len - 1) {
            write!(f, "{} ", self.tokens[i])?
        }
        write!(f, "{}", self.tokens[len - 1])
    }
}

const _FORWARD_SLASH: char = '/';
const _BACKSLASH: char = '\\';
const _SPACE: char = ' ';
const _COLON: char = ':';
const _TRAIT_SEP: char = '.';
const _EQUALS_SIGN: char = '=';
const _QUOTE: char = '"';

pub fn extract_tokens(line: &str) -> Maybe<Line> {
    use Maybe::{Err, Not, Ok};

    // strip ending whitespace
    let mut line = line.trim_end().chars();
    /// gets the next value of `ch` OR
    macro_rules! next_ch_or {
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
                    ch = next_ch_or!(return Err("expected a character; got EOL".to_string()));
                    write_buf!("{}", ch);
                }
                // Treat as comment if two are found
                _FORWARD_SLASH => {
                    // end abruptly on EOL
                    ch = next_ch_or!({
                        write_buf!("/");
                        break;
                    });
                    match ch == _FORWARD_SLASH {
                        // end abruptly on comment
                        true => break,
                        // write slash to buffer and retry matching the next character
                        false => {
                            write_buf!("/");
                            continue;
                        }
                    };
                }
                // Spaces are delimiters
                _SPACE => flush_buf!(),
                // Mark the end of a tag, and allow in-lining afterwards
                _COLON => flush_buf!(RawToken::Colon),
                // `me().attr` expressions
                _TRAIT_SEP => flush_buf!(RawToken::Period),
                // `key="value"` expressions
                _EQUALS_SIGN => flush_buf!(RawToken::Equals),
                // Pause delimiting inside quote blocks
                _QUOTE => loop {
                    ch = next_ch_or!(return Err("expected closing quote; got EOL".to_string()));
                    ch = match ch {
                        _QUOTE => break,
                        _BACKSLASH => next_ch_or!(
                            return Err(
                                "expected closing quote before EOL; got backslash".to_string()
                            )
                        ),
                        ch => ch,
                    };
                    write_buf!("{}", ch);
                },
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
