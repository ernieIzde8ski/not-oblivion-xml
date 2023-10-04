mod token;
use std::iter::Peekable;

#[cfg(debug_assertions)]
use crate::debug;
use crate::err;
use core::fmt::Write as _;
pub use token::{Operator, Token, TokenError};

/// Repeatedly writes to a buffer using new characters from `chars`
/// so long as said chars satisfy a given predicate function.
fn predicated_char_writes(
    mut buf: String,
    chars: &mut Peekable<impl Iterator<Item = char>>,
    pred: impl Fn(&char) -> bool,
) -> Result<String, TokenError> {
    while let Some(mut ch) = chars.next_if(&pred) {
        if ch == '\\' {
            match chars.next() {
                Some(c) => ch = c,
                None => err!(TokenError::UnterminatedStringLiteral(buf)),
            }
        };

        // should be OK?
        write!(buf, "{ch}").expect("writing to string buffer");
    }
    Ok(buf)
}

fn parse_char(
    ch: char,
    chars: &mut Peekable<impl Iterator<Item = char>>,
) -> Result<Option<Token>, TokenError> {
    use Operator::*;
    use Token::*;
    let token = match ch {
        '#' => {
            // consume every subsequent char until hitting next line
            while let Some(_) = chars.next_if(|c| c != &'\n') {}
            return Ok(None);
        }
        '$' => Op(Dollar),
        ':' => Op(Colon),
        '.' => Op(Period),
        '(' => Op(LeftBracket),
        ')' => Op(RightBracket),
        '/' => Op(Slash),
        '*' => Op(Asterisk),
        '-' => Op(Minus),
        '+' => Op(Plus),
        '%' => Op(Mod),
        '!' => match chars.next_if_eq(&'=') {
            Some(_) => Op(NotEqual),
            None => Op(Bang),
        },
        '=' => match chars.next_if_eq(&'=') {
            Some(_) => Op(EqualTo),
            None => Op(EqualsSign),
        },
        '<' => match chars.next_if_eq(&'=') {
            Some(_) => Op(LessThanEqual),
            None => Op(LeftAngle),
        },
        '>' => match chars.next_if_eq(&'=') {
            Some(_) => Op(GreaterThanEqual),
            None => Op(RightAngle),
        },
        '\'' | '"' => {
            let buf = predicated_char_writes(String::new(), chars, |c| c != &ch)?;
            match chars.next() {
                // asserting that the next char is in fact the correct one
                Some(c) if c == ch => Token::StringLiteral(buf),
                _ => err!(TokenError::UnterminatedStringLiteral(buf)),
            }
        }
        c if c.is_whitespace() => return Ok(None),
        // parsing Token::Num
        n if n.is_numeric() => {
            // take all subsequent numeric chars
            let mut buf = predicated_char_writes(String::from(n), chars, |c| c.is_numeric())?;
            if let Some(_) = chars.next_if_eq(&'.') {
                // if we found a decimal point, take all subsequent numerics, again
                write!(buf, ".").expect("writing to string buffer");
                buf = predicated_char_writes(buf, chars, |c| c.is_numeric())?;
            }
            // this *shouldn't* panic, because the rust library is capable of parsing
            // anything along the lines of `\d+(\.\d*)?` (regex format), which is
            // what the buf should match at this point
            Token::Number(buf.parse().expect("should parse to f32"))
        }
        // parsing identifiers as a buffer of any alphanumeric string following an alpha
        c if c == '_' || c.is_alphabetic() => {
            Identifier(predicated_char_writes(String::from(c), chars, |c| {
                c == &'_' || c.is_alphanumeric()
            })?)
        }
        c => err!(TokenError::InvalidChar(c)),
    };

    Ok(Some(token))
}

/// Special parsing case for `\n`. TODO: revise entirely
fn parse_newline(
    indent: &mut (usize, char, usize),
    chars: &mut Peekable<impl Iterator<Item = char>>,
    tokens: &mut Vec<Token>,
) -> Result<(), TokenError> {
    if tokens.len() != 0 {
        let token = Token::Op(Operator::NewLine);
        #[cfg(debug_assertions)]
        debug!("Pushing token: {token:?}");
        tokens.push(token)
    };

    let indent_chars =
        predicated_char_writes(String::new(), chars, |c| c.is_whitespace() && c != &'\n')?;

    'c: {
        let indent_len = indent_chars.len();
        if indent_len == 0 {
            break 'c;
        };
        let indent_chars = Vec::from_iter(indent_chars.chars());

        match chars.peek() {
            Some(c) if c == &'#' || c == &'\n' => break 'c,
            None => break 'c,
            _ => (),
        };

        if indent.1 == '_' {
            indent.1 = indent_chars[0].clone();
            indent.2 = indent_len;
        };

        if indent_chars.iter().any(|c| c != &indent.1) {
            err!(TokenError::InconsistentLeadingWhitespaceChars);
        };

        if indent_len == 0 {
            break 'c;
        } else if indent_len > indent.0 {
            let token = Token::Indent;
            #[cfg(debug_assertions)]
            debug!("Pushing token: {token:?}");
            tokens.push(token);
            indent.0 = indent_len
        } else if indent_len < indent.0 {
            if indent_len % indent.2 != 0 {
                err!(TokenError::InconsistentLeadingWhitespaceChars)
            }

            while indent.0 != indent_len {
                let token = Token::Dedent;
                #[cfg(debug_assertions)]
                debug!("Pushing token: {token:?}");
                tokens.push(token);

                indent.0 -= indent.2;
            }
        };
    }
    Ok(())
}

pub(crate) fn parse_chars(
    mut chars: Peekable<impl Iterator<Item = char>>,
) -> Result<Vec<Token>, TokenError> {
    // Since we check for indent levels after finding a newline,
    // but also since we want to check for indent levels on the
    // first iteration, it becomes necessary to pretend that the
    // first given character is a newline
    let mut ch = '\n';
    let mut resp = Vec::new();

    // information about current indent level
    // indent.0: last indent level
    // indent.1: indent char
    // indent.2: expected diff in indent
    let mut indent = (0, '_', 0);

    loop {
        if ch == '\n' {
            parse_newline(&mut indent, &mut chars, &mut resp)?;
        } else if let Some(token) = parse_char(ch, &mut chars)? {
            #[cfg(debug_assertions)]
            debug!("Pushing token: {token:?}");
            resp.push(token);
        }

        match chars.next() {
            Some(c) => ch = c,
            None => break,
        }
    }

    Ok(resp)
}
