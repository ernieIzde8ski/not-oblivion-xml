#[cfg(debug_assertions)]
use crate::debug;

use super::{errors::ExprConversionFailure, Expr, Expr::*, Line, Token};
use crate::err;

pub type ExprLine = Line<Expr>;

impl TryFrom<Line<Token>> for ExprLine {
    type Error = ExprConversionFailure;

    fn try_from(value: Line<Token>) -> Result<Self, Self::Error> {
        use ExprConversionFailure::*;

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

                Token::EqualTo => EqualTo,
                Token::GreaterThanEqual => GreaterThanEqual,
                Token::LessThanEqual => LessThanEqual,
                Token::NotEqual => NotEqual,
                Token::LeftAngle => LessThan,
                Token::RightAngle => GreaterThan,

                Token::OpenBracket => OpenBracket,
                Token::Div => Div,
                Token::Mult => Mult,
                Token::Sub => Sub,
                Token::Add => Add,
                Token::Mod => Mod,
                Token::CloseBracket => CloseBracket,

                Token::Literal(s) => match tokens.next() {
                    // handling for name='attr' expressions
                    Some(Token::Equals) => {
                        let val = match tokens.next() {
                            Some(Token::Literal(s)) => s,
                            Some(t) => {
                                err!(InvalidToken(t, "expected string after equals sign".into()))
                            }
                            None => err!(UnexpectedLastToken(Token::Literal(s), "string".into())),
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
                        Some(Token::Literal(s)) => s,
                        Some(t) => err!(InvalidToken(t, "expected a string".into())),
                        None => err!(UnexpectedLastToken(token, "string literal".into())),
                    };

                    // argument for the selector or None for the object
                    let arg: Option<String> = match tokens.next() {
                        Some(Token::LeftAngle) => match tokens.next() {
                            // case: $sel<...>.trait
                            Some(Token::Literal(s)) => match tokens.next() {
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
                        Some(Token::Literal(s)) => s,
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
