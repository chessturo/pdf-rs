//! Tests for parsing PDF names

use crate::lexer::PdfLexer;
use crate::pdf::NameParser;

#[test]
fn empty() {
    let inp = b"/";
    assert_eq!(
        NameParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
        b"".to_vec()
    );
}

#[test]
fn smoke() {
    let inp = b"/Name";
    assert_eq!(
        NameParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
        b"Name".to_vec()
    );
}

#[test]
fn escapes() {
    let inp = b"/#2F#2F";
    assert_eq!(
        NameParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
        br"//".to_vec()
    );

    let inp = b"/#20#20";
    assert_eq!(
        NameParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
        b"  ".to_vec()
    );
}

/// Tests to ensure examples provided in PDF Spec section 7.3.5 are handled as described
mod sec735 {
    use crate::lexer::PdfLexer;
    use crate::pdf::NameParser;

    #[test]
    fn table4() {
        let inp = b"/Name1";
        assert_eq!(
            NameParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
            b"Name1".to_vec()
        );

        let inp = b"/ASomewhatLongerName";
        assert_eq!(
            NameParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
            b"ASomewhatLongerName".to_vec()
        );

        let inp = b"/A;Name_With-Various***Characters?";
        assert_eq!(
            NameParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
            b"A;Name_With-Various***Characters?".to_vec()
        );

        let inp = b"/1.2";
        assert_eq!(
            NameParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
            b"1.2".to_vec()
        );

        let inp = b"/$$";
        assert_eq!(
            NameParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
            b"$$".to_vec()
        );

        let inp = b"/@pattern";
        assert_eq!(
            NameParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
            b"@pattern".to_vec()
        );

        let inp = b"/.notdef";
        assert_eq!(
            NameParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
            b".notdef".to_vec()
        );

        let inp = b"/Lime#20Green";
        assert_eq!(
            NameParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
            b"Lime Green".to_vec()
        );

        let inp = b"/paired#28#29parentheses";
        assert_eq!(
            NameParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
            b"paired()parentheses".to_vec()
        );

        let inp = b"/The_Key_of_F#23_Minor";
        assert_eq!(
            NameParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
            b"The_Key_of_F#_Minor".to_vec()
        );

        let inp = b"/A#42";
        assert_eq!(
            NameParser::new().parse(inp, PdfLexer::new(inp)).unwrap(),
            b"AB".to_vec()
        );
    }
}
