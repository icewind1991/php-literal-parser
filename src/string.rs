/// unescaping php string literals borrowed mostly from `escape8259`
use std::char::decode_utf16;
use std::iter::Peekable;

#[derive(Debug, Clone, Eq, PartialEq)]

/// An error occurred while
pub struct UnescapeError;

type UnescapeResult<T> = Result<T, UnescapeError>;

// Used to collect output characters and queue u16 values for translation.
struct UnescapeState {
    // The accumulated characters
    out: String,
    // Store a fragment of a large character for later decoding
    stash: u16,
}

impl UnescapeState {
    fn new() -> UnescapeState {
        UnescapeState {
            out: String::new(),
            stash: 0,
        }
    }

    // Collect a new character
    fn push_char(&mut self, c: char) -> UnescapeResult<()> {
        if self.stash != 0 {
            return Err(UnescapeError);
        }
        self.out.push(c);
        Ok(())
    }

    // Collect a new UTF16 word.  This can either be one whole character,
    // or part of a larger character.
    fn push_u16(&mut self, x: u16) -> UnescapeResult<()> {
        let surrogate = x >= 0xD800 && x <= 0xDFFF;
        match (self.stash, surrogate) {
            (0, false) => {
                // The std library only provides utf16 decode of an iterator,
                // so to decode a single character we wrap it in an array.
                // Hopefully the compiler will elide most of this extra work.
                let words = [x];
                match decode_utf16(words.iter().copied()).next() {
                    Some(Ok(c)) => {
                        self.out.push(c);
                    }
                    _ => return Err(UnescapeError),
                }
            }
            (0, true) => self.stash = x,
            (_, false) => {
                return Err(UnescapeError);
            }
            (w, true) => {
                let words = [w, x];
                match decode_utf16(words.iter().copied()).next() {
                    Some(Ok(c)) => {
                        self.out.push(c);
                        self.stash = 0;
                    }
                    _ => return Err(UnescapeError),
                }
            }
        }
        Ok(())
    }

    // If we queued up part of a UTF-16 encoded word but didn't
    // finish it, return an error.  Otherwise, consume self and
    // return the accumulated String.
    fn finalize(self) -> UnescapeResult<String> {
        if self.stash != 0 {
            return Err(UnescapeError);
        }
        Ok(self.out)
    }
}

fn parse_u16_hex<S>(s: &mut Peekable<S>, max: Option<u8>) -> UnescapeResult<u16>
where
    S: Iterator<Item = char>,
{
    let mut result = 0;
    let mut max = max.unwrap_or(u8::max_value());
    while s.peek().map(|c| c.is_ascii_hexdigit()).unwrap_or_default() {
        result *= 16;
        result += s.next().unwrap().to_digit(16).unwrap() as u16;
        max -= 1;
        if max == 0 {
            break;
        }
    }
    Ok(result)
}

fn parse_u16_oct<S>(s: &mut Peekable<S>, mut result: u16, max: Option<u8>) -> UnescapeResult<u16>
where
    S: Iterator<Item = char>,
{
    let mut max = max.unwrap_or(u8::max_value());
    while s.peek().map(|c| c >= &'1' && c <= &'7').unwrap_or_default() {
        let digit = s.next().unwrap();
        dbg!(digit);
        result *= 8;
        result += digit.to_digit(8).unwrap() as u16;
        max -= 1;
        if max == 0 {
            break;
        }
    }
    Ok(result)
}

/// Un-escape a string, following php single quote rules
pub fn unescape_single(s: &str) -> UnescapeResult<String> {
    let mut state = UnescapeState::new();
    let mut ins = s.chars();

    while let Some(c) = ins.next() {
        if c == '\\' {
            match ins.next() {
                None => {
                    return Err(UnescapeError);
                }
                Some(d) => match d {
                    '\\' | '\'' => state.push_char(d)?,
                    _ => {
                        state.push_char('\\')?;
                        state.push_char(d)?
                    }
                },
            }
        } else {
            state.push_char(c)?;
        }
    }

    state.finalize()
}

/// Un-escape a string, following php double quote rules
pub fn unescape_double(s: &str) -> UnescapeResult<String> {
    let mut state = UnescapeState::new();
    let mut ins = s.chars().peekable();

    while let Some(c) = ins.next() {
        if c == '\\' {
            match ins.next() {
                None => {
                    return Err(UnescapeError);
                }
                Some(d) => {
                    match d {
                        '$' | '"' | '\\' => state.push_char(d)?,
                        'n' => state.push_char('\n')?,   // linefeed
                        'r' => state.push_char('\r')?,   // carriage return
                        't' => state.push_char('\t')?,   // tab
                        'v' => state.push_char('\x0B')?, // vertical tab
                        'f' => state.push_char('\x0C')?, // form feed
                        'x' => {
                            let val = parse_u16_hex(&mut ins, Some(2))?;
                            state.push_u16(val)?;
                        }
                        'u' => match ins.next() {
                            Some('{') => {
                                let val = parse_u16_hex(&mut ins, None)?;
                                state.push_u16(val)?;
                                if !matches!(ins.next(), Some('}')) {
                                    return Err(UnescapeError);
                                }
                            }
                            Some(d) => {
                                state.push_char('\\')?;
                                state.push_char('u')?;
                                state.push_char(d)?;
                            }
                            None => {
                                state.push_char('\\')?;
                                state.push_char(d)?;
                            }
                        },
                        '0'..='7' => {
                            let val =
                                parse_u16_oct(&mut ins, d.to_digit(8).unwrap() as u16, Some(3))?;
                            state.push_u16(val)?;
                        }
                        _ => {
                            state.push_char('\\')?;
                            state.push_char(d)?
                        }
                    }
                }
            }
        } else {
            state.push_char(c)?;
        }
    }

    state.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unescape_single() {
        assert_eq!(unescape_single(&r#"abc"#), Ok("abc".into()));
        assert_eq!(unescape_single(&r#"ab\nc"#), Ok("ab\\nc".into()));
        assert_eq!(unescape_single(r#"ab\zc"#), Ok("ab\\zc".into()));
        assert_eq!(unescape_single(r#" \"abc\" "#), Ok(" \\\"abc\\\" ".into()));
        assert_eq!(unescape_single(r#"𝄞"#), Ok("𝄞".into()));
        assert_eq!(unescape_single(r#"\𝄞"#), Ok("\\𝄞".into()));
        assert_eq!(
            unescape_single(r#"\xD834\xDD1E"#),
            Ok("\\xD834\\xDD1E".into())
        );
        assert_eq!(unescape_single(r#"\xD834"#), Ok("\\xD834".into()));
        assert_eq!(unescape_single(r#"\xDD1E"#), Ok("\\xDD1E".into()));
        assert_eq!(unescape_single("\t"), Ok("\t".into()));
    }

    #[test]
    fn test_unescape_double() {
        assert_eq!(unescape_double(&r#"abc"#), Ok("abc".into()));
        assert_eq!(unescape_double(&r#"ab\nc"#), Ok("ab\nc".into()));
        assert_eq!(unescape_double(r#"ab\zc"#), Ok("ab\\zc".into()));
        assert_eq!(unescape_double(r#" \"abc\" "#), Ok(" \"abc\" ".into()));
        assert_eq!(unescape_double(r#"𝄞"#), Ok("𝄞".into()));
        assert_eq!(unescape_double(r#"\𝄞"#), Ok("\\𝄞".into()));
        assert_eq!(unescape_double(r#"\u{D834}\u{DD1E}"#), Ok("𝄞".into()));
        assert_eq!(unescape_double(r#"\xD834"#), Ok("\u{D8}34".into()));
        assert_eq!(unescape_double(r#"\xDD1E"#), Ok("\u{DD}1E".into()));
        assert_eq!(unescape_double(r#"\xD"#), Ok("\u{D}".into()));
        assert_eq!(unescape_double("\t"), Ok("\t".into()));
        assert_eq!(unescape_double(r#"\u{D834"#), Err(UnescapeError));
        assert_eq!(unescape_double(r#"\uD834"#), Ok("\\uD834".into()));
        assert_eq!(unescape_double(r#"\u"#), Ok("\\u".into()));
        assert_eq!(unescape_double(r#"\47foo"#), Ok("'foo".into()));
        assert_eq!(unescape_double(r#"\48foo"#), Ok("\u{4}8foo".into()));
        assert_eq!(unescape_double(r#"\87foo"#), Ok("\\87foo".into()));
    }
}
