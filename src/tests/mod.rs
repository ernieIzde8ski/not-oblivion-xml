use rstest::rstest;

use crate::Operator::*;
use crate::Token::*;
use crate::*;

#[rstest]
#[case("#This line should be empty.", vec![NewLine("".into())])]
#[case("$me<child>.width-0.0", vec![
    NewLine("".into()),
    Op(Dollar),
    Identifier("me".into()),
    Op(LeftAngle),
    Identifier("child".into()),
    Op(RightAngle),
    Op(Period),
    Identifier("width".into()),
    Op(Minus),
    Number(0.0),
])]
#[case("( / * - + % )", vec![
    NewLine("".into()), Op(LeftBracket), Op(Slash), Op(Asterisk), Op(Minus), Op(Plus), Op(Mod), Op(RightBracket),
])]
#[case("0 = 1 == 2 > 3 >= 4 < 5 <= 6 ! 7 != 8.0", vec![
    NewLine("".into()),   Number(0.0),
    Op(EqualsSign),       Number(1.0),
    Op(EqualTo),          Number(2.0),
    Op(RightAngle),       Number(3.0),
    Op(GreaterThanEqual), Number(4.0),
    Op(LeftAngle),        Number(5.0),
    Op(LessThanEqual),    Number(6.0),
    Op(Bang),             Number(7.0),
    Op(NotEqual),         Number(8.0),
])]
fn general_lexing(#[case] line: &str, #[case] expected: Vec<Token>) {
    let value = parse_string(line).expect("should yield expressions");
    assert_eq!(value, expected);
}
