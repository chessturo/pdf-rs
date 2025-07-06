//! Tests for PDF string parsing

/// Tests for `RawPdfStr`
mod raw {
    use crate::lexer::PdfLexer;
    use crate::pdf::RawPdfStrParser;

    #[test]
    fn smoke() {
        let inp = b"(test)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"test".to_vec()
        );

        let inp = b"(\n\n\n)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"\n\n\n".to_vec()
        );
    }

    #[test]
    fn empty() {
        let inp = b"()";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"".to_vec()
        );
    }

    #[test]
    fn failure() {
        let inp = br"()()";
        assert!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .is_err()
        );

        let inp = br"()\)";
        assert!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .is_err()
        );

        let inp = br"(\\\)";
        assert!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .is_err()
        );
    }

    #[test]
    fn nested_delims() {
        let inp = b"(())";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"()".to_vec()
        );

        let inp = b"(abc () xyz)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"abc () xyz".to_vec()
        );

        let inp = b"(abc (()) () xyz)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"abc (()) () xyz".to_vec()
        );
    }

    #[test]
    fn whitespace1() {
        let inp = b"(abc\nxyz)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"abc\nxyz".to_vec()
        );
    }

    #[test]
    fn escape_delim() {
        let inp = br"(\)\()";
        dbg!(PdfLexer::new(inp).collect::<Vec<_>>());
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            br")(".to_vec()
        );

        let inp = br"(\\\)\()";
        dbg!(PdfLexer::new(inp).collect::<Vec<_>>());
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            br"\)(".to_vec()
        );

        let inp = br"(( \) \( ))";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            br"( ) ( )".to_vec()
        );
    }

    #[test]
    fn line_continue() {
        let inp = b"(abc\\\nxyz)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"abcxyz".to_vec()
        );

        let inp = b"(abc\\\r\nxyz)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"abcxyz".to_vec()
        );

        let inp = b"(abc\\\n\nxyz)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"abc\nxyz".to_vec()
        );
    }

    #[test]
    fn escape_whitespace_newline_coalesce() {
        let inp = b"(\t\r\n\r\r\n)";
        // PDF Spec section 7.3.4.2:
        // An end-of-line marker appearing within a literal string without a preceding REVERSE
        // SOLIDUS shall be treated as a byte value of (0Ah), irrespective of whether the
        // end-of-line marker was a CARRIAGE RETURN (0Dh), a LINE FEED (0Ah), or both.
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"\t\n\n\n".to_vec()
        );
    }
    #[test]
    fn escape_whitespace_newline_replace() {
        let inp = b"(\n\n\n\r\r\r\r)";
        // PDF Spec section 7.3.4.2:
        // An end-of-line marker appearing within a literal string without a preceding REVERSE
        // SOLIDUS shall be treated as a byte value of (0Ah), irrespective of whether the
        // end-of-line marker was a CARRIAGE RETURN (0Dh), a LINE FEED (0Ah), or both.
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"\n\n\n\n\n\n\n".to_vec()
        );
    }
    #[test]
    fn escape_whitespace_all() {
        let inp = br"(\n\r\t\b\f)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"\n\r\t\x08\x0C".to_vec()
        );
    }

    #[test]
    fn escape_escape() {
        let inp = br"(\\\\\\n)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            br"\\\n".to_vec()
        );
    }

    #[test]
    fn escape_octal_low() {
        let inp = br"(\0)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"\x00".to_vec()
        );

        let inp = br"(\000)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"\x00".to_vec()
        );
    }
    #[test]
    fn escape_octal_middle() {
        let inp = br"(\1)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"\x01".to_vec()
        );
        let inp = br"(\100)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"\x40".to_vec()
        );
    }
    #[test]
    fn escape_octal_limit() {
        let inp = br"(\377)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"\xFF".to_vec()
        );
    }

    #[test]
    fn escape_invalid_letters() {
        let inp = br"(\x\y\z)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"xyz".to_vec()
        );
    }
    #[test]
    fn escape_invalid_octal() {
        let inp = br"(abc\400xyz)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"abcxyz".to_vec()
        );
        let inp = br"(abc\800xyz)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"abc800xyz".to_vec()
        );
        let inp = br"(abc\08xyz)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"abc\x008xyz".to_vec()
        );
        let inp = br"(abc\008xyz)";
        assert_eq!(
            RawPdfStrParser::new()
                .parse(inp, PdfLexer::new(inp))
                .unwrap(),
            b"abc\x008xyz".to_vec()
        );
    }

    /// Tests that the examples given in section 7.3.4.2 of the PDF Spec are handled as described
    /// there.
    mod sec7342 {
        use crate::lexer::PdfLexer;
        use crate::pdf::RawPdfStrParser;

        #[test]
        fn example1() {
            let inp = b"(This is a string)";
            assert_eq!(
                RawPdfStrParser::new()
                    .parse(inp, PdfLexer::new(inp))
                    .unwrap(),
                b"This is a string".to_vec()
            );

            let inp = br"(Strings can contain
 newlines and such.)";
            assert_eq!(
                RawPdfStrParser::new()
                    .parse(inp, PdfLexer::new(inp))
                    .unwrap(),
                b"Strings can contain\n newlines and such.".to_vec()
            );

            let inp = br"(Strings can contain balanced parentheses ()
 and special characters ( * ! & } ^ %and so on) .)";
            assert_eq!(
                RawPdfStrParser::new()
                    .parse(inp, PdfLexer::new(inp))
                    .unwrap(),
                b"Strings can contain balanced parentheses ()\n and special characters ( * ! & } ^ %and so on) .".to_vec()
            );

            let inp = br"(The following is an empty string .)";
            assert_eq!(
                RawPdfStrParser::new()
                    .parse(inp, PdfLexer::new(inp))
                    .unwrap(),
                b"The following is an empty string .".to_vec()
            );

            let inp = br"()";
            assert_eq!(
                RawPdfStrParser::new()
                    .parse(inp, PdfLexer::new(inp))
                    .unwrap(),
                b"".to_vec()
            );

            let inp = br"(It has zero (0) length.)";
            assert_eq!(
                RawPdfStrParser::new()
                    .parse(inp, PdfLexer::new(inp))
                    .unwrap(),
                b"It has zero (0) length.".to_vec()
            );
        }

        #[test]
        fn example2() {
            let inp1 = br"(These \
two strings \
are the same.)";
            let out1 = RawPdfStrParser::new()
                .parse(inp1, PdfLexer::new(inp1))
                .unwrap();

            let inp2 = br"(These two strings are the same.)";
            let out2 = RawPdfStrParser::new()
                .parse(inp2, PdfLexer::new(inp2))
                .unwrap();

            assert_eq!(out1, out2);
            assert_eq!(out1, b"These two strings are the same.".to_vec());
        }

        #[test]
        fn example3() {
            let inp = br"(This string has an end-of-line at the end of it.
)";
            assert_eq!(
                RawPdfStrParser::new()
                    .parse(inp, PdfLexer::new(inp))
                    .unwrap(),
                b"This string has an end-of-line at the end of it.\n".to_vec()
            );

            let inp = br"(So does this one.\n)";
            assert_eq!(
                RawPdfStrParser::new()
                    .parse(inp, PdfLexer::new(inp))
                    .unwrap(),
                b"So does this one.\n".to_vec()
            );
        }

        #[test]
        fn example4() {
            let inp = br"(This string contains \245two octal characters\307.)";
            assert_eq!(
                RawPdfStrParser::new()
                    .parse(inp, PdfLexer::new(inp))
                    .unwrap(),
                b"This string contains \xA5two octal characters\xC7.".to_vec()
            );
        }

        #[test]
        fn example5() {
            let inp = br"(\0053)";
            assert_eq!(
                RawPdfStrParser::new()
                    .parse(inp, PdfLexer::new(inp))
                    .unwrap(),
                b"\x053".to_vec()
            );

            let inp = br"(\053)";
            assert_eq!(
                RawPdfStrParser::new()
                    .parse(inp, PdfLexer::new(inp))
                    .unwrap(),
                b"\x2B".to_vec()
            );

            let inp = br"(\53)";
            assert_eq!(
                RawPdfStrParser::new()
                    .parse(inp, PdfLexer::new(inp))
                    .unwrap(),
                b"\x2B".to_vec()
            );
        }
    }
}
