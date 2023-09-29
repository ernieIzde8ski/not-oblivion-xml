#[cfg(test)]
mod parsing {
    use crate::parsing::*;

    #[test]
    fn empty_line() {
        let line = "# This line should be empty.";
        let err = extract_line(line).expect_err("should be empty");
        assert!(matches!(err, crate::LineConversionError::NoTokensPresent))
    }

    #[test]
    fn attributes() {
        let line = "rect name=\"container\": // this line should have an Attribute operator";
        let line = extract_line(line).expect("should yield tokens");
        let expected = Expr::Attribute {
            key: "name".to_owned(),
            val: "container".to_owned(),
        };
        assert!(line.tokens.contains(&expected));
    }

    #[test]
    fn traits() {
        let line = "me().width - 0\\.0 # This should contain a trait and escaped syntax.";
        let line = extract_line(line).expect("should yield tokens");

        let expected = Line {
            leading_whitespace: 0,
            tokens: vec![
                Expr::Trait {
                    src: "me()".to_string(),
                    r#trait: "width".to_string(),
                },
                Expr::Arithmetic(ArithmeticToken::Sub),
                Expr::Raw("0.0".to_string()),
            ],
        };

        assert_eq!(line, expected);
    }

    #[test]
    fn syntax_error() {
        let line = "me(). // This line should have a syntax error.";
        extract_line(line).expect_err("should fail with a syntax error");
    }

    #[test]
    fn arithmetic_operators() {
        use ArithmeticToken::*;
        use Expr::*;

        let tokens = extract_line("[ / * - + % ]")
            .expect("should yield tokens")
            .tokens;
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
        use RelationalOperator::*;
        use Expr::*;
        let tokens = extract_line("1 == 2 > 3 >= 4 < 5 <= 6 != 7")
            .expect("should yield tokens")
            .tokens;
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
}
