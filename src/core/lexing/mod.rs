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
        '\n' => {
            // take all subsequent whitespace non-newline chars
            let buf =
                predicated_char_writes(String::new(), chars, |c| c.is_whitespace() && c != &'\n')?;
            Token::NewLine(buf)
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
        c => Identifier(predicated_char_writes(String::from(c), chars, |c| {
            c.is_alphanumeric()
        })?),
    };

    Ok(Some(token))
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

    loop {
        if let Some(token) = parse_char(ch, &mut chars)? {
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
