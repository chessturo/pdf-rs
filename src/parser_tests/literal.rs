//! Test for boolean and numeric literals

mod boolean {
    use crate::lexer::PdfLexer;
    use crate::pdf::BooleanParser;

    #[test]
    fn true_val() {
        let inp = b"true";
        assert_eq!(
            BooleanParser::new()
                .parse(inp, PdfLexer::new(inp).unwrap())
                .unwrap(),
            true
        );
    }

    #[test]
    fn false_val() {
        let inp = b"false";
        assert_eq!(
            BooleanParser::new()
                .parse(inp, PdfLexer::new(inp).unwrap())
                .unwrap(),
            false
        );
    }
}

mod number {
    mod integer {
        use crate::ast::object::Number;
        use crate::lexer::PdfLexer;
        use crate::pdf::NumberParser;

        #[test]
        fn zero() {
            let inp = b"0";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Integer(0)
            );
        }

        #[test]
        fn leading_zero() {
            let inp = b"00";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Integer(0)
            );

            let inp = b"07";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Integer(7)
            );

            let inp = b"09";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Integer(9)
            );

            let inp = b"0900";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Integer(900)
            );
        }

        #[test]
        fn negative() {
            let inp = b"-0";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Integer(-0)
            );

            let inp = b"-1";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Integer(-1)
            );

            let inp = b"-100";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Integer(-100)
            );
        }
    }

    mod real {
        use crate::ast::object::Number;
        use crate::lexer::PdfLexer;
        use crate::pdf::NumberParser;

        #[test]
        fn zero() {
            let inp = b"0.";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(0.0)
            );

            let inp = b".0";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(0.0)
            );

            let inp = b"0.0";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(0.0)
            );

            let inp = b"0.00000";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(0.0)
            );

            let inp = b"-0.";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(-0.0)
            );

            let inp = b"-.0";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(-0.0)
            );

            let inp = b"-.0";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(-0.0)
            );

            let inp = b"-0.0";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(-0.0)
            );

            let inp = b"-0.000000";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(-0.0)
            );
        }

        #[test]
        fn positive() {
            let inp = b"12.";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(12 as f64)
            );

            let inp = b".12";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(0.12)
            );

            let inp = b"0.12";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(0.12)
            );

            let inp = b"12.34";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(12.34)
            );
        }

        #[test]
        fn negative() {
            let inp = b"-12.";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(-12 as f64)
            );

            let inp = b"-.12";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(-0.12)
            );

            let inp = b"-0.12";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(-0.12)
            );

            let inp = b"-12.34";
            assert_eq!(
                NumberParser::new()
                    .parse(inp, PdfLexer::new(inp).unwrap())
                    .unwrap(),
                Number::Real(-12.34)
            );
        }
    }
}
