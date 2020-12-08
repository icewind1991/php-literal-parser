#[derive(Debug, Clone, Eq, PartialEq)]

/// An error occurred while
pub struct UnescapeError;

type UnescapeResult<T> = Result<T, UnescapeError>;

// Used to collect output characters and queue u16 values for translation.
struct UnescapeState {
    // The accumulated characters
    out: Vec<u8>,
}

impl UnescapeState {
    fn new() -> UnescapeState {
        UnescapeState { out: Vec::new() }
    }

    fn with_capacity(capacity: usize) -> UnescapeState {
        UnescapeState {
            out: Vec::with_capacity(capacity),
        }
    }

    // Collect a new character
    fn push_char(&mut self, c: char) {
        let mut buff = [0; 8];
        self.out
            .extend_from_slice(c.encode_utf8(&mut buff).as_bytes());
    }

    fn push_u8(&mut self, c: u8) {
        self.out.push(c);
    }

    fn push_raw(&mut self, c: u32) -> UnescapeResult<()> {
        match std::char::from_u32(c) {
            Some(c) => Ok(self.push_char(c)),
            None => Err(UnescapeError),
        }
    }

    fn push_slice(&mut self, slice: &[u8]) {
        self.out.extend_from_slice(slice);
    }

    fn finalize(self) -> String {
        // this is safe because we only push bytes into the buffer that either
        //   - come from the source &str, and are delimited a \
        //   - are validated unicode points, utf8 encoded
        unsafe { String::from_utf8_unchecked(self.out) }
    }
}

fn parse_u32(
    s: &mut PeekableBytes,
    radix: u32,
    mut result: u32,
    max: Option<u8>,
) -> UnescapeResult<u32> {
    let mut max = max.unwrap_or(u8::max_value());
    while let Some(digit) = s.peek().and_then(|digit| (digit as char).to_digit(radix)) {
        let _ = s.next(); // consume the digit we peeked
        result = result.checked_mul(radix).ok_or(UnescapeError)?;
        result = result.checked_add(digit).ok_or(UnescapeError)?;
        max -= 1;
        if max == 0 {
            break;
        }
    }
    Ok(result)
}

fn handle_single_escape<'a>(
    bytes: &'a [u8],
    state: &mut UnescapeState,
) -> UnescapeResult<&'a [u8]> {
    let mut ins = PeekableBytes::new(bytes);
    debug_assert_eq!(ins.next(), Some(b'\\'));
    match ins.next() {
        None => {
            return Err(UnescapeError);
        }
        Some(d) => match d {
            b'\\' | b'\'' => state.push_u8(d),
            _ => {
                state.push_u8(b'\\');
                state.push_u8(d)
            }
        },
    }
    Ok(ins.as_slice())
}

/// Un-escape a string, following php single quote rules
pub fn unescape_single(s: &str) -> UnescapeResult<String> {
    let mut state = UnescapeState::with_capacity(s.len());
    let mut bytes = s.as_bytes();
    while let Some(escape_index) = memchr::memchr(b'\\', bytes) {
        state.push_slice(&bytes[0..escape_index]);
        bytes = &bytes[escape_index..];
        bytes = handle_single_escape(bytes, &mut state)?;
    }

    state.push_slice(&bytes[0..]);

    Ok(state.finalize())
}

fn handle_double_escape<'a>(
    bytes: &'a [u8],
    state: &mut UnescapeState,
) -> UnescapeResult<&'a [u8]> {
    let mut ins = PeekableBytes::new(bytes);
    debug_assert_eq!(ins.next(), Some(b'\\'));
    match ins.next() {
        None => {
            return Err(UnescapeError);
        }
        Some(d) => {
            match d {
                b'$' | b'"' | b'\\' => state.push_u8(d),
                b'n' => state.push_u8(b'\n'),   // linefeed
                b'r' => state.push_u8(b'\r'),   // carriage return
                b't' => state.push_u8(b'\t'),   // tab
                b'v' => state.push_u8(b'\x0B'), // vertical tab
                b'f' => state.push_u8(b'\x0C'), // form feed
                b'x' => {
                    let val = parse_u32(&mut ins, 16, 0, Some(2))?;
                    state.push_raw(val)?;
                }
                b'u' => match ins.next() {
                    Some(b'{') => {
                        let val = parse_u32(&mut ins, 16, 0, None)?;
                        state.push_raw(val)?;
                        if !matches!(ins.next(), Some(b'}')) {
                            return Err(UnescapeError);
                        }
                    }
                    Some(d) => {
                        state.push_u8(b'\\');
                        state.push_u8(b'u');
                        state.push_u8(d);
                    }
                    None => {
                        state.push_u8(b'\\');
                        state.push_u8(d);
                    }
                },
                b'0'..=b'7' => {
                    let val = parse_u32(&mut ins, 8, (d as char).to_digit(8).unwrap(), Some(3))?;
                    state.push_raw(val)?;
                }
                _ => {
                    state.push_u8(b'\\');
                    state.push_u8(d)
                }
            }
        }
    }
    Ok(ins.as_slice())
}

/// Un-escape a string, following php double quote rules
pub fn unescape_double(s: &str) -> UnescapeResult<String> {
    let mut state = UnescapeState::with_capacity(s.len());
    let mut bytes = s.as_bytes();
    while let Some(escape_index) = memchr::memchr(b'\\', bytes) {
        state.push_slice(&bytes[0..escape_index]);
        bytes = &bytes[escape_index..];
        bytes = handle_double_escape(bytes, &mut state)?;
    }

    state.push_slice(&bytes[0..]);

    Ok(state.finalize())
}

struct PeekableBytes<'a> {
    slice: &'a [u8],
    pos: usize,
}

impl<'a> Iterator for PeekableBytes<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let byte = self.slice.get(self.pos)?;
        self.pos += 1;
        Some(*byte)
    }
}

impl<'a> PeekableBytes<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        PeekableBytes { slice, pos: 0 }
    }

    pub fn peek(&self) -> Option<u8> {
        self.slice.get(self.pos).copied()
    }

    pub fn as_slice(self) -> &'a [u8] {
        &self.slice[self.pos..]
    }
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
        assert_eq!(unescape_double(r#"\u{1D11E}"#), Ok("𝄞".into()));
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

        assert_eq!(unescape_double(r#"\u{999999}"#), Err(UnescapeError));
        assert_eq!(
            unescape_double(r#"\u{999999999999999999}"#),
            Err(UnescapeError)
        );
    }
}
