use crate::lexer::Token;
use crate::string::UnescapeError;
use crate::Value;
use logos::Span;
use source_span::{
    fmt::{Color, Formatter, Style},
    DefaultMetrics, Position, SourceBuffer, Span as SourceSpan,
};
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;
use thiserror::Error;

/// An error and related source span, will print out the problematic code fragment and error on `Display`
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
pub struct SpannedError<'a, T: Error + Debug> {
    span: Span,
    source: &'a str,
    error: T,
}

impl<'a, T: Error + Debug> SpannedError<'a, T> {
    pub fn new(error: T, span: Span, source: &'a str) -> Self {
        SpannedError {
            span,
            source,
            error,
        }
    }

    pub fn error(&self) -> &T {
        &self.error
    }
}

impl<'a, T: Error + Debug + 'static> Error for SpannedError<'a, T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

const METRICS: DefaultMetrics = DefaultMetrics::with_tab_stop(4);

impl<'a, T: Error + Debug> fmt::Display for SpannedError<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = get_position(self.source, self.span.start);
        let end = get_position(self.source, self.span.end);
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
pub enum ParseError {
    #[error("{0}")]
    UnexpectedToken(#[from] UnexpectedTokenError),
    #[error("{0}")]
    InvalidArrayKey(#[from] InvalidArrayKeyError),
    #[error("Invalid boolean literal: {0}")]
    InvalidBoolLiteral(#[from] ParseBoolError),
    #[error("Invalid integer literal: {0}")]
    InvalidIntLiteral(#[from] ParseIntError),
    #[error("Invalid float literal: {0}")]
    InvalidFloatLiteral(#[from] ParseFloatError),
    #[error("Invalid string literal")]
    InvalidStringLiteral,
}

impl From<UnescapeError> for ParseError {
    fn from(_: UnescapeError) -> Self {
        ParseError::InvalidStringLiteral
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

#[derive(Error, Debug)]
#[error("Invalid array key {0:?} expected number or string")]
pub struct InvalidArrayKeyError(pub Value);

pub trait ExpectToken {
    fn expect_token(self, expected: &[Token]) -> Result<Token, UnexpectedTokenError>;
}

impl ExpectToken for Option<Token> {
    fn expect_token(self, expected: &[Token]) -> Result<Token, UnexpectedTokenError> {
        self.ok_or_else(|| UnexpectedTokenError {
            expected: expected.to_vec(),
            found: None,
        })
        .and_then(|token| token.expect_token(expected))
    }
}

impl ExpectToken for Token {
    fn expect_token(self, expected: &[Token]) -> Result<Token, UnexpectedTokenError> {
        if expected.iter().any(|expect| self.eq(expect)) {
            Ok(self)
        } else {
            Err(UnexpectedTokenError {
                expected: expected.to_vec(),
                found: Some(self),
            })
        }
    }
}

pub trait ResultExt<'a, T, E: Error + Debug> {
    fn with_span(self, span: Span, source: &'a str) -> Result<T, SpannedError<'a, E>>;
}

impl<'a, T, E: Into<ParseError>> ResultExt<'a, T, ParseError> for Result<T, E> {
    fn with_span(self, span: Span, source: &'a str) -> Result<T, SpannedError<'a, ParseError>> {
        self.map_err(|error| SpannedError {
            span,
            source,
            error: error.into(),
        })
    }
}
