#[cfg(test)]
mod parsing {
    use crate::parsing::*;

    #[test]
    fn empty_line() {
        let line = "// This line should be empty.";
        extract_tokens(line).expect_not("should be empty")
    }

    #[test]
    fn equality_line() {
        let line = "rect name=\"container\": // this line should have an Equality operator";
        let line = extract_tokens(line).expect("should have tokens");
        let expected = Token::Equality {
            key: "name".to_owned(),
            val: "container".to_owned(),
        };
        assert!(line.tokens.contains(&expected));
    }

    #[test]
    fn traits() {
        let line = "me().width - 0\\.0 // This should contain a trait and escaped syntax.";
        let line = extract_tokens(line).expect("should have tokens");

        let expected = Line {
            leading_whitespace: 0,
            tokens: vec![
                Token::Trait {
                    src: "me()".to_string(),
                    r#trait: "width".to_string(),
                },
                Token::Raw("-".to_string()),
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
}
