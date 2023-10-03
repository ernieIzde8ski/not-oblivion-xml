use rstest::rstest;

use crate::core::*;

#[test]
fn empty_line() {
    let line = "# This line should be empty.";
    let err = ExprLine::try_from(line).expect_err("should be empty");

    assert!(matches!(
        err,
        LineConversionFailure::TokenFailure(TokenConversionFailure::NoTokensPresent)
    ))
}

#[test]
fn attributes() {
    let line = "rect name=\"container\": // this line should have an Attribute operator";
    let line = ExprLine::try_from(line).expect("should yield expressions");
    let expected = Expr::Attribute {
        key: "name".to_owned(),
        val: "container".to_owned(),
    };
    assert!(line.members.contains(&expected));
}

#[rstest]
#[case("$me.width-0\\.0", vec![
    Expr::Trait { src: "me".into(), arg: None, r#trait: "width".into() },
    Expr::Sub,
    Expr::Num(0.0),
])]
#[case("$me<>.width - 0\\.0", vec![
    Expr::Trait {src: "me".into(), arg: Some("".into()), r#trait: "width".into()},
    Expr::Sub,
    Expr::Num(0.0),
])]
#[case("$me<0\\.0>.width", vec![
    Expr::Trait { src: "me".into(), arg: Some("0.0".into()), r#trait: "width".into() }
])]
fn trait_tags(#[case] line: &str, #[case] expected: Vec<Expr>) {
    let value = ExprLine::try_from(line)
        .expect("should yield expressions")
        .members;
    assert_eq!(value, expected);
}

#[test]
fn syntax_error() {
    let line = "me(). // This line should have a syntax error.";
    ExprLine::try_from(line).expect_err("should fail with a syntax error");
}

#[test]
fn arithmetic_operators() {
    use Expr::*;

    let tokens = ExprLine::try_from("[ / * - + % ]")
        .expect("should yield expressions")
        .members;
    let expected = vec![OpenBracket, Div, Mult, Sub, Add, Mod, CloseBracket];
    assert_eq!(tokens, expected)
}

#[test]
fn relational_operators() {
    use Expr::*;
    let tokens = ExprLine::try_from("1 == 2 > 3 >= 4 < 5 <= 6 != 7")
        .expect("should yield expressions")
        .members;
    let expected = vec![
        Num(1.0),
        EqualTo,
        Num(2.0),
        GreaterThan,
        Num(3.0),
        GreaterThanEqual,
        Num(4.0),
        LessThan,
        Num(5.0),
        LessThanEqual,
        Num(6.0),
        NotEqual,
        Num(7.0),
    ];

    assert_eq!(tokens, expected);
}
