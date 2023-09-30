#[cfg(test)]
mod parsing {
    use crate::errors;
    use crate::parsing::*;

    #[test]
    fn empty_line() {
        let line = "# This line should be empty.";
        let err = ExprLine::try_from(line).expect_err("should be empty");
        assert!(match err {
            LineConversionFailure::TokenFailure(err) =>
                matches!(err, errors::TokenConversionFailure::NoTokensPresent),
            _ => false,
        })
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

    #[test]
    fn traits() {
        let line = "me().width - 0\\.0 # This should contain a trait and escaped syntax.";
        let line = ExprLine::try_from(line).expect("should yield expressions");

        let expected = Line {
            total_whitespace: 0,
            members: vec![
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
        ExprLine::try_from(line).expect_err("should fail with a syntax error");
    }

    #[test]
    fn arithmetic_operators() {
        use ArithmeticToken::*;
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
}
