//! Helper functions called from the parser

pub(crate) fn handle_raw_str_escapes(val: &[u8]) -> Vec<u8> {
    let mut it = val.iter().peekable();
    let mut out: Vec<u8> = Vec::with_capacity(val.len());
    loop {
        match it.next() {
            None => return out,

            // An end-of-line marker appearing within a literal string without a preceding
            // REVERSE SOLIDUS shall be treated as a byte value of (0Ah), irrespective of
            // whether the end-of-line marker was a CARRIAGE RETURN (0Dh), a LINE FEED
            // (0Ah), or both.
            Some(b'\r') => {
                if let Some(b'\n') = it.peek() {
                    it.next();
                }
                out.push(b'\n');
            }

            Some(b'\\') => {
                // We unwrap because a string cannot end with an unescaped `\`, since then it would
                // be escaping the end delimiter
                match **it.peek().unwrap() {
                    b'n' => {
                        out.push(b'\n');
                        it.next();
                    }
                    b'r' => {
                        out.push(b'\r');
                        it.next();
                    }
                    b't' => {
                        out.push(b'\t');
                        it.next();
                    }
                    b'b' => {
                        out.push(b'\x08' /* BACKSPACE (BS) */);
                        it.next();
                    }
                    b'f' => {
                        out.push(b'\x0C' /* FORM FEED (FF) */);
                        it.next();
                    }
                    b'(' => {
                        out.push(b'(');
                        it.next();
                    }
                    b')' => {
                        out.push(b')');
                        it.next();
                    }
                    b'\\' => {
                        out.push(b'\\');
                        it.next();
                    }

                    // PDF Spec section 7.3.4.2:
                    // The REVERSE SOLIDUS (5Ch) (backslash character) at the end of a line shall
                    // be used to indicate that the string continues on the following line. A PDF
                    // processor shall disregard the REVERSE SOLIDUS and the end-of-line marker
                    // following it when reading the string; the resulting string value shall be
                    // identical to that which would be read if the string were not split.
                    b'\r' => {
                        it.next();
                        if let Some(b'\n') = it.peek() {
                            it.next();
                        }
                    }
                    b'\n' => {
                        it.next();
                    }

                    mut digit1 @ b'0'..=b'7' => {
                        it.next();
                        let mut digit2;
                        let mut digit3;
                        match it.peek() {
                            Some(d @ b'0'..=b'7') => {
                                digit2 = **d;
                                it.next();
                            }
                            None | Some(_) => {
                                digit2 = digit1;
                                digit1 = b'0';
                            }
                        }
                        match it.peek() {
                            Some(d @ b'0'..=b'7') => {
                                digit3 = **d;
                                it.next();
                            }
                            None | Some(_) => {
                                digit3 = digit2;
                                digit2 = digit1;
                                digit1 = b'0';
                            }
                        }

                        digit1 -= b'0';
                        digit2 -= b'0';
                        digit3 -= b'0';

                        let res = digit1 as u32 * u32::pow(8, 2)
                            + digit2 as u32 * u32::pow(8, 1)
                            + digit3 as u32 * u32::pow(8, 0);
                        if res <= u8::MAX as u32 {
                            out.push(res as u8)
                        }
                    }

                    // PDF Spec section 7.3.4.2:
                    // If the character following the REVERSE SOLIDUS is not one of those shown in
                    // "Table 3 — Escape sequences in literal strings", the REVERSE SOLIDUS shall
                    // be ignored.
                    c => {
                        out.push(c);
                        it.next();
                    }
                }
            }

            Some(c) => out.push(*c),
        }
    }
}

/// Decodes the hex representation of the string content into the actual bytes.
///
/// Returns `None` if any of the characters are outside the range `b'0'..=b'9' | b'a'..=b'f' |
/// b'A'..=b'F'`
pub(crate) fn handle_hex_str(val: &[u8]) -> Option<Vec<u8>> {
    let mut it = val.iter();
    let mut out = Vec::with_capacity(val.len() / 2 + 1);
    loop {
        let Some(digit1) = it.next() else {
            return Some(out);
        };
        let Some(digit2) = it.next() else {
            out.push(char_to_val(*digit1)? * 16);
            return Some(out);
        };
        out.push(char_to_val(*digit1)? * 16 + char_to_val(*digit2)?);
    }
}

fn char_to_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

pub(crate) fn handle_name_escapes(val: &[u8]) -> Option<Vec<u8>> {
    let mut it = val.iter();
    // Consume the leading b'/'
    assert!(it.next() == Some(&b'/'));

    let mut out = Vec::with_capacity(val.len());

    loop {
        match it.next() {
            Some(b'#') => {
                let d1 = char_to_val(*it.next()?)?;
                let d2 = char_to_val(*it.next()?)?;
                out.push(d1 * 16 + d2);
            }
            Some(c) => out.push(*c),
            None => return Some(out),
        }
    }
}
