//! Test for boolean and numeric literals

mod boolean {
    use crate::lexer::PdfLexer;
    use crate::pdf::BooleanParser;

    #[test]
    fn true_val() {
        let inp = b"true";
        assert_eq!(
            BooleanParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
            true
        );
    }

    #[test]
    fn false_val() {
        let inp = b"false";
        assert_eq!(
            BooleanParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
            false
        );
    }
}
