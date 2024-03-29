use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ParseIntError {
    #[error("cannot parse integer from empty string")]
    Empty,
    #[error("invalid digit found in string")]
    InvalidDigit,
    #[error("number too large or small to fit in target type")]
    Overflow,
    #[error("unexpected negative number")]
    UnexpectedNegative,
}
impl ParseIntError {
    pub fn desc(&self) -> &str {
        match self {
            ParseIntError::Empty => "Number expected",
            ParseIntError::InvalidDigit => "Not a valid digit",
            ParseIntError::Overflow => "Number to large",
            ParseIntError::UnexpectedNegative => "Expected a positive number",
        }
    }
}

/// Mostly copied from std, adjusted for php specifics
pub fn parse_int(src: &str) -> Result<i64, ParseIntError> {
    if src.is_empty() {
        return Err(ParseIntError::Empty);
    }

    // all valid digits are ascii, so we will just iterate over the utf8 bytes
    // and cast them to chars. .to_digit() will safely return None for anything
    // other than a valid ascii digit for the given radix, including the first-byte
    // of multi-byte sequences
    let src = src.as_bytes();

    let (sign, digits) = match src[0] {
        b'+' => (1, &src[1..]),
        b'-' => (-1, &src[1..]),
        _ => (1, src),
    };

    let (radix, digits) = match digits {
        [b'0', b'x', tail @ ..] => (16, tail),
        [b'0', b'b', tail @ ..] => (2, tail),
        [b'0', tail @ ..] if !tail.is_empty() => (8, tail),
        tail => (10, tail),
    };

    if digits.is_empty() {
        return Err(ParseIntError::Empty);
    }

    let mut result: i64 = 0;

    // The number is positive
    for &c in digits {
        if c != b'_' {
            let x = match (c as char).to_digit(radix) {
                Some(x) => x,
                None => return Err(ParseIntError::InvalidDigit),
            };
            result = match result.checked_mul(radix as i64) {
                Some(result) => result,
                None => return Err(ParseIntError::Overflow),
            };
            result = match result.checked_add(x as i64) {
                Some(result) => result,
                None => return Err(ParseIntError::Overflow),
            };
        }
    }
    Ok(result * sign)
}
