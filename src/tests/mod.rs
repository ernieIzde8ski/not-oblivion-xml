#[cfg(test)]
mod parsing {
    use crate::parsing::*;

    #[test]
    fn empty_line() {
        let line = "# This line should be empty.";
        extract_tokens(line).expect_not("should be empty")
    }

    #[test]
    fn attributes() {
        let line = "rect name=\"container\": // this line should have an Attribute operator";
        let line = extract_tokens(line).expect("should yield tokens");
        let expected = Token::Attribute {
            key: "name".to_owned(),
            val: "container".to_owned(),
        };
        assert!(line.tokens.contains(&expected));
    }

    #[test]
    fn traits() {
        let line = "me().width - 0\\.0 # This should contain a trait and escaped syntax.";
        let line = extract_tokens(line).expect("should yield tokens");

        let expected = Line {
            leading_whitespace: 0,
            tokens: vec![
                Token::Trait {
                    src: "me()".to_string(),
                    r#trait: "width".to_string(),
                },
                Token::Arithmetic(ArithmeticToken::Sub),
                Token::Raw("0.0".to_string()),
            ],
        };

        assert_eq!(line, expected);
    }

    #[test]
    fn syntax_error() {
        let line = "me(). // This line should have a syntax error.";
        extract_tokens(line).expect_err("should fail with a syntax error");
    }

    #[test]
    fn arithmetic_operators() {
        use ArithmeticToken::*;
        use Token::*;

        let tokens = extract_tokens("[ / * - + % ]")
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
        use Token::*;
        let tokens = extract_tokens("1 == 2 > 3 >= 4 < 5 <= 6 != 7")
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
