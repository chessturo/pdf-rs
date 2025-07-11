//! Lexer for parsing PDF files

use std::fmt::Display;
use std::iter::{Enumerate, Peekable};
use std::slice::Iter;

#[derive(Copy, Clone, Debug)]
pub(crate) enum Tok<'input> {
    RawStrDelimOpen,
    RawStrDelimClose,
    RawStrContent(&'input [u8]),
    HexStrDelimOpen,
    HexStrDelimClose,
    HexStrContent(&'input [u8]),

    Name(&'input [u8]),

    True,
    False,

    Number(&'input [u8]),

    UnknownTok(&'input [u8]),
}

#[derive(Clone, Copy)]
enum PdfLexerMode {
    Base,
    RawString,
    HexString,
}

#[derive(Debug)]
pub enum PdfLexError<'input> {
    /// Represents a situation where a character is found (at position `usize`) that cannot be
    /// lexed into a token
    UnexpectedChar(usize),
    /// Represents a situation where we the file ends mid-token.
    UnexpectedEOF,
    /// Represents a situation where the token we're lexing is (much) longer than it should be
    TokenTooLong(&'input [u8]),
}

impl Display for PdfLexError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PdfLexError::UnexpectedChar(loc) => write!(f, "Unexpected character at byte {}", loc),
            PdfLexError::UnexpectedEOF => write!(f, "Unexpected end-of-file"),
            PdfLexError::TokenTooLong(tok) => {
                if let Ok(s) = str::from_utf8(tok) {
                    write!(f, "Token `{s}...` is longer than it should be")
                } else {
                    write!(f, "Token too long")
                }
            }
        }
    }
}

struct PdfLexerForwards<'input> {
    chars: Peekable<Enumerate<Iter<'input, u8>>>,
    input: &'input [u8],
    mode: PdfLexerMode,
}

impl<'input> PdfLexerForwards<'input> {
    pub fn new(input: &'input [u8]) -> Self {
        Self {
            chars: input.iter().enumerate().peekable(),
            input,
            mode: PdfLexerMode::Base,
        }
    }
}

// How many bytes to look for a keyword before giving up and issuing a lexer error
static KEYWORD_LOOKAHEAD: usize = 30;

pub(crate) type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;
impl<'input> Iterator for PdfLexerForwards<'input> {
    type Item = Spanned<Tok<'input>, usize, PdfLexError<'input>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.mode {
            PdfLexerMode::Base => {
                'base: loop {
                    match self.chars.next() {
                        // EOF
                        None => return None,

                        // We're starting a raw string, switch to its mode.
                        Some((i, b'(')) => {
                            self.mode = PdfLexerMode::RawString;
                            return Some(Ok((i, Tok::RawStrDelimOpen, i + 1)));
                        }
                        Some((i, b')')) => {
                            return Some(Ok((i, Tok::RawStrDelimClose, i + 1)));
                        }

                        // We're starting a hex string, switch to its mode
                        Some((i, b'<')) => {
                            self.mode = PdfLexerMode::HexString;
                            return Some(Ok((i, Tok::HexStrDelimOpen, i + 1)));
                        }
                        Some((i, b'>')) => {
                            return Some(Ok((i, Tok::HexStrDelimClose, i + 1)));
                        }

                        // Handle a name token
                        Some((i, b'/')) => return self.lex_name(i),

                        // Handle a number
                        Some((i, b'0'..=b'9'))
                        | Some((i, b'.'))
                        | Some((i, b'+'))
                        | Some((i, b'-')) => return self.lex_number(i),

                        // Comment, consume all characters until an EOL marker
                        Some((_, b'%')) => loop {
                            match self.chars.next() {
                                None => return None,
                                Some((_, b'\r')) => {
                                    if let Some((_, b'\n')) = self.chars.peek() {
                                        self.chars.next();
                                    }
                                    continue 'base;
                                }
                                Some((_, b'\n')) => {
                                    continue 'base;
                                }
                                Some(_) => {}
                            }
                        },

                        // Skip whitespace in base mode
                        Some((_, b'\x00'))
                        | Some((_, b'\t'))
                        | Some((_, b'\n'))
                        | Some((_, b'\x0C' /* FORM FEED */))
                        | Some((_, b'\r'))
                        | Some((_, b' ')) => continue,

                        // It's not some kind of delimiter, so look for a keyword
                        Some((i, _)) => return self.lex_keyword(i),
                    }
                }
            }

            PdfLexerMode::RawString => return self.lex_raw_string(),

            PdfLexerMode::HexString => return self.lex_hex_string(),
        }
    }
}

impl PdfLexerForwards<'_> {
    fn lex_raw_string(&mut self) -> Option<<Self as Iterator>::Item> {
        let mut depth = 1;
        // FIXME once
        // https://doc.rust-lang.org/std/iter/struct.Enumerate.html#method.next_index is
        // stabilized
        let head = self.chars.peek();
        if head.is_none() {
            return None;
        }
        let start = head.unwrap().0;
        loop {
            match self.chars.peek() {
                None => {
                    return Some(Ok((
                        start,
                        Tok::RawStrContent(&self.input[start..]),
                        self.input.len(),
                    )));
                }
                Some((i, c)) => {
                    match **c {
                        b')' => {
                            if depth == 1 {
                                self.mode = PdfLexerMode::Base;
                                return Some(Ok((
                                    start,
                                    Tok::RawStrContent(&self.input[start..*i]),
                                    *i,
                                )));
                            } else {
                                depth -= 1;
                                self.chars.next();
                            }
                        }
                        b'(' => {
                            depth += 1;
                            self.chars.next();
                        }
                        b'\\' => {
                            self.chars.next();
                            // Consume whatever was escaped to prevent depth changes if it
                            // was a `(` or `)`
                            self.chars.next();
                        }
                        // Just some character
                        _ => {
                            self.chars.next();
                        }
                    }
                }
            }
        }
    }

    fn lex_hex_string(&mut self) -> Option<<Self as Iterator>::Item> {
        // FIXME once
        // https://doc.rust-lang.org/std/iter/struct.Enumerate.html#method.next_index is
        // stabilized
        let head = self.chars.peek();
        if head.is_none() {
            return Some(Err(PdfLexError::UnexpectedEOF));
        }
        let start = head.unwrap().0;
        loop {
            match self.chars.peek() {
                None => return Some(Err(PdfLexError::UnexpectedEOF)),
                Some((i, b'>')) => {
                    self.mode = PdfLexerMode::Base;
                    return Some(Ok((start, Tok::HexStrContent(&self.input[start..*i]), *i)));
                }
                Some((_, b'0'..=b'9')) | Some((_, b'a'..=b'f')) | Some((_, b'A'..=b'F')) => {
                    self.chars.next();
                }
                Some((i, _)) => return Some(Err(PdfLexError::UnexpectedChar(*i))),
            }
        }
    }

    fn lex_name(&mut self, start: usize) -> Option<<Self as Iterator>::Item> {
        loop {
            match self.chars.peek() {
                // Names are ended by (non-NUL) whitespace
                Some((j, b'\t'))
                | Some((j, b'\n'))
                | Some((j, b'\x0C' /* FORM FEED */))
                | Some((j, b'\r'))
                | Some((j, b' ')) => {
                    return Some(Ok((start, Tok::Name(&self.input[start..*j]), *j)));
                }
                // I suppose a name could be ended by EOF as well...
                None => {
                    return Some(Ok((
                        start,
                        Tok::Name(&self.input[start..]),
                        self.input.len(),
                    )));
                }
                // The NUL character is disallowed in names
                Some((j, b'\x00')) => {
                    return Some(Err(PdfLexError::UnexpectedChar(*j)));
                }
                Some((_, _)) => {
                    self.chars.next();
                }
            }
        }
    }

    fn lex_keyword(&mut self, start: usize) -> Option<<Self as Iterator>::Item> {
        let mut keyword_len = 1;
        loop {
            match self.chars.peek() {
                // Whitespace or EOF separates tokens
                None
                | Some((_, b'\x00'))
                | Some((_, b'\t'))
                | Some((_, b'\n'))
                | Some((_, b'\x0C' /* FORM FEED */))
                | Some((_, b'\r'))
                | Some((_, b' ')) => break,

                Some((_, _)) => {
                    self.chars.next();
                    keyword_len += 1;
                    if keyword_len == KEYWORD_LOOKAHEAD {
                        return Some(Err(PdfLexError::TokenTooLong(
                            &self.input[start..(start + keyword_len)],
                        )));
                    }
                }
            }
        }

        let tok = match &self.input[start..(start + keyword_len)] {
            b"true" => Tok::True,
            b"false" => Tok::False,
            _ => Tok::UnknownTok(&self.input[start..(start + keyword_len)]),
        };

        return Some(Ok((start, tok, start + keyword_len)));
    }

    fn lex_number(&mut self, start: usize) -> Option<<Self as Iterator>::Item> {
        loop {
            match self.chars.peek() {
                Some((_, b'0'..=b'9')) | Some((_, b'.')) => {
                    self.chars.next();
                }
                // End of the number
                Some((j, b'\x00'))
                | Some((j, b'\t'))
                | Some((j, b'\n'))
                | Some((j, b'\x0C' /* FORM FEED */))
                | Some((j, b'\r'))
                | Some((j, b' ')) => {
                    return Some(Ok((start, Tok::Number(&self.input[start..*j]), *j)));
                }
                // An EOF could end the number as well
                None => {
                    return Some(Ok((
                        start,
                        Tok::Number(&self.input[start..]),
                        self.input.len(),
                    )));
                }
                // Some gunk in the middle of our number
                Some((j, _)) => {
                    return Some(Err(PdfLexError::UnexpectedChar(*j)));
                }
            }
        }
    }
}

pub(crate) struct PdfLexer<'input> {
    toks: Vec<(usize, Tok<'input>, usize)>,
}

impl<'input> PdfLexer<'input> {
    pub(crate) fn new(input: &'input [u8]) -> Result<PdfLexer<'input>, PdfLexError<'input>> {
        let lex = PdfLexerForwards::new(input);
        let toks = lex.collect::<Result<Vec<_>, PdfLexError>>()?;
        Ok(Self { toks })
    }
}

impl<'input> Iterator for PdfLexer<'input> {
    type Item = Spanned<Tok<'input>, usize, PdfLexError<'input>>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Ok(self.toks.pop()?))
    }
}
