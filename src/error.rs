use crate::lexer::{SpannedToken, Token};
use crate::num::ParseIntError;
use crate::string::UnescapeError;
use logos::Span;
use source_span::{
    fmt::{Color, Formatter, Style},
    DefaultMetrics, Position, SourceBuffer, Span as SourceSpan,
};
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::num::ParseFloatError;
use std::str::ParseBoolError;
use thiserror::Error;

/// An error and related source span
///
/// You can pretty-print the error with the offending source by using `with_source`
///
/// ## Example
///
/// ```text
/// . |
/// 2 |     [
/// 3 |         "broken"
/// 4 |         "array"                                                                                         
///   |         ^^^^^^^^ Unexpected token, found LiteralString expected one of [SquareClose, Comma, Arrow]
/// 5 |     ]
/// 6 |
/// ```
///
#[derive(Debug)]
pub struct ParseError {
    span: Option<Span>,
    error: RawParseError,
}

impl serde::de::Error for ParseError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        ParseError {
            span: None,
            error: RawParseError::custom(msg),
        }
    }
}

impl ParseError {
    pub fn new(error: RawParseError, span: Span) -> Self {
        ParseError {
            span: Some(span),
            error,
        }
    }

    pub fn error(&self) -> &RawParseError {
        &self.error
    }

    pub fn with_source(self, source: &str) -> SourceSpannedError {
        SourceSpannedError {
            span: self.span,
            error: self.error,
            source,
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <RawParseError as Display>::fmt(&self.error, f)
    }
}

impl From<RawParseError> for ParseError {
    fn from(err: RawParseError) -> Self {
        ParseError {
            span: None,
            error: err,
        }
    }
}

pub struct SourceSpannedError<'source> {
    span: Option<Span>,
    error: RawParseError,
    source: &'source str,
}

impl<'source> SourceSpannedError<'source> {
    pub fn into_inner(self) -> ParseError {
        ParseError {
            span: self.span,
            error: self.error,
        }
    }
}

const METRICS: DefaultMetrics = DefaultMetrics::with_tab_stop(4);

impl<'source> Display for SourceSpannedError<'source> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.span.as_ref() {
            Some(span) => {
                let start = get_position(self.source, span.start);
                let end = get_position(self.source, span.end);
                let span = SourceSpan::new(start, end, end.next_line());

                let mut fmt = Formatter::with_margin_color(Color::Blue);
                let buffer = SourceBuffer::new(
                    self.source.chars().map(|char| Result::<char, ()>::Ok(char)),
                    Position::default(),
                    METRICS,
                );
                fmt.add(span, Some(format!("{}", self.error)), Style::Error);
                let formatted = fmt
                    .render(
                        buffer.iter(),
                        SourceSpan::new(
                            Position::default(),
                            Position::new(usize::max_value() - 1, usize::max_value()),
                            Position::end(),
                        ),
                        &METRICS,
                    )
                    .unwrap();
                write!(f, "{}", formatted)?;
            }
            None => write!(f, "{}", self.error)?,
        }
        Ok(())
    }
}

fn get_position(text: &str, index: usize) -> Position {
    let mut pos = Position::default();
    for char in text.chars().take(index) {
        pos = pos.next(char, &METRICS);
    }

    pos
}

#[derive(Error, Debug)]
pub enum RawParseError {
    #[error("{0}")]
    UnexpectedToken(#[from] UnexpectedTokenError),
    #[error("Invalid boolean literal: {0}")]
    InvalidBoolLiteral(#[from] ParseBoolError),
    #[error("Invalid integer literal: {0}")]
    InvalidIntLiteral(#[from] ParseIntError),
    #[error("Invalid float literal: {0}")]
    InvalidFloatLiteral(#[from] ParseFloatError),
    #[error("Invalid string literal")]
    InvalidStringLiteral,
    #[error("Array key not valid for this position")]
    UnexpectedArrayKey,
    #[error("Trailing characters after parsing")]
    TrailingCharacters,
    #[error("{0}")]
    Custom(String),
}

impl serde::de::Error for RawParseError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        RawParseError::Custom(msg.to_string())
    }
}

impl From<UnescapeError> for RawParseError {
    fn from(_: UnescapeError) -> Self {
        RawParseError::InvalidStringLiteral
    }
}

#[derive(Debug)]
pub struct UnexpectedTokenError {
    pub expected: Vec<Token>,
    pub found: Option<Token>,
}

impl UnexpectedTokenError {
    pub fn new(expected: &[Token], found: Option<Token>) -> Self {
        UnexpectedTokenError {
            expected: expected.to_vec(),
            found,
        }
    }
}

impl Display for UnexpectedTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.found {
            Some(Token::Error) => write!(
                f,
                "No valid token found, expected one of {:?}",
                self.expected
            ),
            Some(token) => write!(
                f,
                "Unexpected token, found {:?} expected one of {:?}",
                token, self.expected
            ),
            None => write!(
                f,
                "Unexpected token, found None expected one of {:?}",
                self.expected
            ),
        }
    }
}

impl Error for UnexpectedTokenError {}

pub trait ExpectToken<'source> {
    fn expect_token(self, expected: &[Token]) -> Result<SpannedToken<'source>, ParseError>;
}

impl<'source> ExpectToken<'source> for Option<SpannedToken<'source>> {
    fn expect_token(self, expected: &[Token]) -> Result<SpannedToken<'source>, ParseError> {
        self.ok_or_else(|| UnexpectedTokenError {
            expected: expected.to_vec(),
            found: None,
        })
        .with_span(usize::max_value()..usize::max_value())
        .and_then(|token| token.expect_token(expected))
    }
}

impl<'a, 'source> ExpectToken<'source> for Option<&'a SpannedToken<'source>> {
    fn expect_token(self, expected: &[Token]) -> Result<SpannedToken<'source>, ParseError> {
        self.ok_or_else(|| UnexpectedTokenError {
            expected: expected.to_vec(),
            found: None,
        })
        .with_span(usize::max_value()..usize::max_value())
        .and_then(|token| token.clone().expect_token(expected))
    }
}

impl<'source> ExpectToken<'source> for SpannedToken<'source> {
    fn expect_token(self, expected: &[Token]) -> Result<SpannedToken<'source>, ParseError> {
        if expected.iter().any(|expect| self.token.eq(expect)) {
            Ok(self)
        } else {
            Err(UnexpectedTokenError {
                expected: expected.to_vec(),
                found: Some(self.token),
            })
            .with_span(self.span)
        }
    }
}

pub trait ResultExt<T> {
    fn with_span(self, span: Span) -> Result<T, ParseError>;
}

impl<T, E: Into<RawParseError>> ResultExt<T> for Result<T, E> {
    fn with_span(self, span: Span) -> Result<T, ParseError> {
        self.map_err(|error| ParseError {
            span: Some(span),
            error: error.into(),
        })
    }
}
