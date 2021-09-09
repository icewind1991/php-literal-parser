use crate::lexer::{SpannedToken, Token};
use crate::num::ParseIntError;
use crate::string::UnescapeError;
use logos::Span;
use source_span::{
    fmt::{Formatter, Style},
    DefaultMetrics, Position, SourceBuffer, Span as SourceSpan,
};
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::num::ParseFloatError;
use std::str::ParseBoolError;
use thiserror::Error;

/// Any error that occurred while trying to parse the php literal
#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error(transparent)]
    /// A token that wasn't expected was found while parsing
    UnexpectedToken(#[from] UnexpectedTokenError),
    #[error(transparent)]
    /// A malformed integer, float, boolean or string literal was found
    InvalidPrimitive(#[from] PrimitiveError),
    #[error(transparent)]
    /// An array key was found that is invalid for this position
    UnexpectedArrayKey(ArrayKeyError),
    #[error(transparent)]
    /// Trailing characters after parsing
    TrailingCharacters(#[from] TrailingError),
    #[error("{0}")]
    /// Error while populating serde type
    Serde(String),
}

impl serde::de::Error for ParseError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        ParseError::Serde(msg.to_string())
    }
}

/// A token that wasn't expected was found while parsing
#[derive(Debug, Clone)]
pub struct UnexpectedTokenError {
    src: String,
    snip: Span,
    err_span: Span,
    pub expected: TokenList,
    pub found: Option<Token>,
}

impl Display for UnexpectedTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err = match &self.found {
            Some(Token::Error) => {
                format!("No valid token found, expected one of {}", self.expected)
            }
            Some(token) => format!(
                "Unexpected token, found {} expected one of {}",
                token, self.expected
            ),
            None => format!(
                "Unexpected token, found None expected one of {}",
                self.expected
            ),
        };
        fmt_spanned(f, err, self.err_span.clone(), &self.src)
    }
}

impl UnexpectedTokenError {
    pub fn new(
        expected: &[Token],
        found: Option<Token>,
        src: String,
        snip: Span,
        err_span: Span,
    ) -> Self {
        UnexpectedTokenError {
            src,
            snip,
            err_span,
            expected: expected.into(),
            found,
        }
    }
}

/// List of expected tokens
#[derive(Clone)]
pub struct TokenList(Vec<Token>);

impl Debug for TokenList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&[Token]> for TokenList {
    fn from(list: &[Token]) -> Self {
        TokenList(list.into())
    }
}

impl Display for TokenList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.len() {
            0 => {}
            1 => write!(f, "{}", self.0[0])?,
            _ => {
                let mut tokens = self.0[0..self.0.len() - 1].iter();
                write!(f, "{}", tokens.next().unwrap())?;
                for token in tokens {
                    write!(f, ", {}", token)?;
                }
                if self.0.len() > 1 {
                    write!(f, " or {}", self.0.last().unwrap())?;
                }
            }
        }
        Ok(())
    }
}

impl Error for UnexpectedTokenError {}

/// A malformed integer, float, boolean or string literal was found
#[derive(Debug, Clone)]
pub struct PrimitiveError {
    src: String,
    snip: Span,
    err_span: Span,
    pub kind: PrimitiveErrorKind,
}

#[derive(Error, Debug, Clone)]
pub enum PrimitiveErrorKind {
    #[error("Invalid boolean literal: {0}")]
    InvalidBoolLiteral(#[from] ParseBoolError),
    #[error("Invalid integer literal: {0}")]
    InvalidIntLiteral(#[from] ParseIntError),
    #[error("Invalid float literal: {0}")]
    InvalidFloatLiteral(#[from] ParseFloatError),
    #[error("Invalid string literal")]
    InvalidStringLiteral,
}

impl Display for PrimitiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err = format!("{}", self.kind);
        fmt_spanned(f, err, self.err_span.clone(), &self.src)
    }
}

impl Error for PrimitiveError {}

impl PrimitiveErrorKind {
    pub fn desc(&self) -> &str {
        match self {
            PrimitiveErrorKind::InvalidBoolLiteral(_) => "Not a boolean",
            PrimitiveErrorKind::InvalidIntLiteral(err) => err.desc(),
            PrimitiveErrorKind::InvalidFloatLiteral(_) => "Not a valid float",
            PrimitiveErrorKind::InvalidStringLiteral => "Not a string literal",
        }
    }
}

impl From<UnescapeError> for PrimitiveErrorKind {
    fn from(_: UnescapeError) -> Self {
        PrimitiveErrorKind::InvalidStringLiteral
    }
}

#[derive(Debug, Clone)]
pub struct ArrayKeyError {
    src: String,
    snip: Span,
    err_span: Span,
    kind: ArrayKeyErrorKind,
}

#[derive(Debug, Clone)]
pub enum ArrayKeyErrorKind {
    IntegerExpected,
    NonConsecutive,
}

impl Display for ArrayKeyErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ArrayKeyErrorKind::IntegerExpected => "Expected integer key",
                ArrayKeyErrorKind::NonConsecutive => "Expected consecutive integer key",
            }
        )
    }
}

impl Display for ArrayKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err = format!("{}", self.kind);
        fmt_spanned(f, err, self.err_span.clone(), &self.src)
    }
}

impl Error for ArrayKeyError {}

impl ArrayKeyError {
    pub fn new(kind: ArrayKeyErrorKind, source: &str, err_span: Span) -> Self {
        ArrayKeyError {
            src: source.into(),
            snip: (0..source.len()),
            err_span,
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrailingError {
    src: String,
    snip: Span,
    err_span: Span,
}

impl TrailingError {
    pub fn new(source: &str, err_span: Span) -> Self {
        TrailingError {
            src: source.into(),
            snip: (0..source.len()),
            err_span,
        }
    }
}

impl Display for TrailingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_spanned(
            f,
            format!("end of parsed value"),
            self.err_span.clone(),
            &self.src,
        )
    }
}

impl Error for TrailingError {}

pub trait ExpectToken<'source> {
    fn expect_token(
        self,
        expected: &[Token],
        source: &str,
    ) -> Result<SpannedToken<'source>, ParseError>;
}

impl<'source> ExpectToken<'source> for Option<SpannedToken<'source>> {
    fn expect_token(
        self,
        expected: &[Token],
        source: &str,
    ) -> Result<SpannedToken<'source>, ParseError> {
        self.ok_or_else(|| {
            UnexpectedTokenError::new(
                expected,
                None,
                source.into(),
                0..source.len(),
                source.len()..source.len(),
            )
            .into()
        })
        .and_then(|token| token.expect_token(expected, source))
    }
}

impl<'a, 'source> ExpectToken<'source> for Option<&'a SpannedToken<'source>> {
    fn expect_token(
        self,
        expected: &[Token],
        source: &str,
    ) -> Result<SpannedToken<'source>, ParseError> {
        self.ok_or_else(|| {
            UnexpectedTokenError::new(
                expected,
                None,
                source.into(),
                0..source.len(),
                source.len()..source.len(),
            )
            .into()
        })
        .and_then(|token| token.clone().expect_token(expected, source))
    }
}

impl<'source> ExpectToken<'source> for SpannedToken<'source> {
    fn expect_token(
        self,
        expected: &[Token],
        source: &str,
    ) -> Result<SpannedToken<'source>, ParseError> {
        if expected.iter().any(|expect| self.token.eq(expect)) {
            Ok(self)
        } else {
            Err(UnexpectedTokenError::new(
                expected,
                Some(self.token),
                source.into(),
                0..source.len(),
                self.span,
            )
            .into())
        }
    }
}

pub trait ResultExt<T> {
    fn with_span(self, span: Span, source: &str) -> Result<T, ParseError>;
}

impl<T, E: Into<PrimitiveErrorKind>> ResultExt<T> for Result<T, E> {
    fn with_span(self, span: Span, source: &str) -> Result<T, ParseError> {
        self.map_err(|error| {
            PrimitiveError {
                src: source.into(),
                snip: (0..source.len()),
                err_span: span,
                kind: error.into(),
            }
            .into()
        })
    }
}

fn get_position(text: &str, index: usize) -> Position {
    let mut pos = Position::default();
    for char in text.chars().take(index) {
        pos = pos.next(char, &METRICS);
    }

    pos
}

const METRICS: DefaultMetrics = DefaultMetrics::with_tab_stop(4);

fn fmt_spanned(f: &mut fmt::Formatter<'_>, err: String, span: Span, source: &str) -> fmt::Result {
    let start = get_position(source, span.start);
    let end = get_position(source, span.end);
    let span = SourceSpan::new(start, end, end.next_line());

    let mut fmt = Formatter::new();
    let buffer = SourceBuffer::new(
        source.chars().map(|char| Result::<char, ()>::Ok(char)),
        Position::default(),
        METRICS,
    );
    fmt.add(span, Some(format!("{}", err)), Style::Error);
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
    write!(f, "{}", formatted)
}
