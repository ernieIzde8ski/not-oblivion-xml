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
    Expr::Arithmetic(ArithmeticOperator::Sub),
    Expr::Raw("0.0".into()),
])]
#[case("$me<>.width - 0\\.0", vec![
    Expr::Trait {src: "me".into(), arg: Some("".into()), r#trait: "width".into()},
    Expr::Arithmetic(ArithmeticOperator::Sub),
    Expr::Raw("0.0".into()),
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
    use ArithmeticOperator::*;
    use Expr::*;

    let tokens = ExprLine::try_from("[ / * - + % ]")
        .expect("should yield expressions")
        .members;
    let expected = vec![
        Arithmetic(OpenBracket),
        Arithmetic(Div),
        Arithmetic(Mult),
        Arithmetic(Sub),
        Arithmetic(Add),
        Arithmetic(Mod),
        Arithmetic(CloseBracket),
    ];
    assert_eq!(tokens, expected)
}

#[test]
fn relational_operators() {
    use Expr::*;
    use RelationalOperator::*;
    let tokens = ExprLine::try_from("1 == 2 > 3 >= 4 < 5 <= 6 != 7")
        .expect("should yield expressions")
        .members;
    let expected = vec![
        Int(1),
        Relational(EqualTo),
        Int(2),
        Relational(GreaterThan),
        Int(3),
        Relational(GreaterThanEqual),
        Int(4),
        Relational(LessThan),
        Int(5),
        Relational(LessThanEqual),
        Int(6),
        Relational(NotEqual),
        Int(7),
    ];

    assert_eq!(tokens, expected);
}
