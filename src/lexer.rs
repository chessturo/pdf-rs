//! Lexer for parsing PDF files

use std::cell::RefCell;
use std::fmt::Display;
use std::iter::{Enumerate, Peekable};
use std::rc::Rc;
use std::slice::Iter;

#[derive(Copy, Clone, Debug)]
pub(crate) enum Tok<'input> {
    RawStrDelimOpen,
    RawStrDelimClose,
    RawStrContent(&'input [u8]),
}

pub(crate) enum PdfLexerMode {
    /// The base mode; in the top-level structure of the PDF.
    Base,
    RawString,
}

#[derive(Debug)]
pub enum PdfLexError {
    /// Represents a situation where a character is found (at position `usize`) that cannot be
    /// lexed into a token
    UnexpectedChar(usize),
    /// Represents a situation where we the file ends mid-token.
    UnexpectedEOF,
}

impl Display for PdfLexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PdfLexError::UnexpectedChar(loc) => write!(f, "Unexpected character at byte {}", loc),
            PdfLexError::UnexpectedEOF => write!(f, "Unexpected end-of-file"),
        }
    }
}

pub(crate) struct PdfLexer<'input> {
    chars: Peekable<Enumerate<Iter<'input, u8>>>,
    input: &'input [u8],
    /// The current lexer mode. `pub(crate)` so the parser can reach in and mess with it if needed.
    pub(crate) mode: Rc<RefCell<PdfLexerMode>>,
}

impl<'input> PdfLexer<'input> {
    pub fn new(input: &'input [u8]) -> Self {
        Self {
            chars: input.iter().enumerate().peekable(),
            input,
            mode: Rc::new(RefCell::new(PdfLexerMode::Base)),
        }
    }
}

pub(crate) type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;
impl<'input> Iterator for PdfLexer<'input> {
    type Item = Spanned<Tok<'input>, usize, PdfLexError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut mode = self.mode.borrow_mut();
        match *mode {
            PdfLexerMode::Base => {
                match self.chars.next() {
                    // EOF
                    None => return None,

                    // We're starting a string, switch to string mode.
                    Some((i, b'(')) => {
                        *mode = PdfLexerMode::RawString;
                        return Some(Ok((i, Tok::RawStrDelimOpen, i + 1)));
                    }
                    Some((i, b')')) => {
                        return Some(Ok((i, Tok::RawStrDelimClose, i + 1)));
                    }

                    // Catch-all error case
                    Some((i, _)) => return Some(Err(PdfLexError::UnexpectedChar(i))),
                }
            }

            PdfLexerMode::RawString => {
                let mut depth = 1;
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
                        Some((i, c)) => {
                            match **c {
                                b')' => {
                                    if depth == 1 {
                                        *mode = PdfLexerMode::Base;
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
        }
    }
}
