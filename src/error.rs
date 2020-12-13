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
/// You can pretty-print the error with the offending source by using `display_with_source`
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
pub struct SpannedError<T: Error + Debug> {
    span: Span,
    error: T,
}

impl<T: Error + Debug> SpannedError<T> {
    pub fn new(error: T, span: Span) -> Self {
        SpannedError { span, error }
    }

    pub fn error(&self) -> &T {
        &self.error
    }
}

impl<T: Error + Debug + 'static> Error for SpannedError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

impl<T: Error + Debug> Display for SpannedError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <T as Display>::fmt(&self.error, f)
    }
}

const METRICS: DefaultMetrics = DefaultMetrics::with_tab_stop(4);

impl<T: Error + Debug> SpannedError<T> {
    pub fn display_with_source(&self, source: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = get_position(source, self.span.start);
        let end = get_position(source, self.span.end);
        let span = SourceSpan::new(start, end, end.next_line());

        let mut fmt = Formatter::with_margin_color(Color::Blue);
        let buffer = SourceBuffer::new(
            source.chars().map(|char| Result::<char, ()>::Ok(char)),
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

pub trait ExpectToken<'source> {
    fn expect_token(
        self,
        expected: &[Token],
    ) -> Result<SpannedToken<'source>, SpannedError<ParseError>>;
}

impl<'source> ExpectToken<'source> for Option<SpannedToken<'source>> {
    fn expect_token(
        self,
        expected: &[Token],
    ) -> Result<SpannedToken<'source>, SpannedError<ParseError>> {
        self.ok_or_else(|| UnexpectedTokenError {
            expected: expected.to_vec(),
            found: None,
        })
        .with_span(usize::max_value()..usize::max_value())
        .and_then(|token| token.expect_token(expected))
    }
}

impl<'source> ExpectToken<'source> for SpannedToken<'source> {
    fn expect_token(
        self,
        expected: &[Token],
    ) -> Result<SpannedToken<'source>, SpannedError<ParseError>> {
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

pub trait ResultExt<T, E: Error + Debug> {
    fn with_span(self, span: Span) -> Result<T, SpannedError<E>>;
}

impl<T, E: Into<ParseError>> ResultExt<T, ParseError> for Result<T, E> {
    fn with_span(self, span: Span) -> Result<T, SpannedError<ParseError>> {
        self.map_err(|error| SpannedError {
            span,
            error: error.into(),
        })
    }
}
