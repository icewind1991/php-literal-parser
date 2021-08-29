use crate::lexer::{SpannedToken, Token};
use crate::num::ParseIntError;
use crate::string::UnescapeError;
use logos::Span;
use miette::{Diagnostic, SourceOffset, SourceSpan};
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::num::ParseFloatError;
use std::str::ParseBoolError;
use thiserror::Error;

/// Any error that occurred while trying to parse the php literal
#[derive(Error, Debug, Clone, Diagnostic)]
pub enum ParseError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    /// A token that wasn't expected was found while parsing
    UnexpectedToken(#[from] UnexpectedTokenError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    /// A malformed integer, float, boolean or string literal was found
    InvalidPrimitive(#[from] PrimitiveError),
    #[error("Array key not valid for this position")]
    #[diagnostic(transparent)]
    /// An array key was found that is invalid for this position
    UnexpectedArrayKey(ArrayKeyError),
    #[error("Trailing characters after parsing")]
    #[diagnostic(code(php_object_parser::trailing))]
    /// Trailing characters after parsing
    TrailingCharacters,
    #[error("{0}")]
    #[diagnostic(code(php_object_parser::serde))]
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
#[derive(Debug, Clone, Diagnostic)]
#[diagnostic(code(php_object_parser::unexpected_token))]
pub struct UnexpectedTokenError {
    src: String,
    #[snippet(src)]
    snip: SourceSpan,
    #[highlight(snip, label("Expected {}", self.expected))]
    err_span: SourceSpan,
    pub expected: TokenList,
    pub found: Option<Token>,
}

impl UnexpectedTokenError {
    pub fn new(
        expected: &[Token],
        found: Option<Token>,
        src: String,
        snip: SourceSpan,
        err_span: SourceSpan,
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&[Token]> for TokenList {
    fn from(list: &[Token]) -> Self {
        TokenList(list.into())
    }
}

impl Display for TokenList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

impl Display for UnexpectedTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.found {
            Some(Token::Error) => {
                write!(f, "No valid token found, expected one of {}", self.expected)
            }
            Some(token) => write!(
                f,
                "Unexpected token, found {} expected one of {}",
                token, self.expected
            ),
            None => write!(
                f,
                "Unexpected token, found None expected one of {}",
                self.expected
            ),
        }
    }
}

impl Error for UnexpectedTokenError {}

/// A malformed integer, float, boolean or string literal was found
#[derive(Debug, Clone, Error, Diagnostic)]
#[diagnostic(code(php_object_parser::invalid_primitive))]
#[error("{kind}")]
pub struct PrimitiveError {
    src: String,
    #[snippet(src)]
    snip: SourceSpan,
    #[highlight(snip, label("{}", self.kind.desc()))]
    err_span: SourceSpan,
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

#[derive(Debug, Clone, Error, Diagnostic)]
#[diagnostic(code(php_object_parser::invalid_array_key))]
#[error("Invalid array key")]
pub struct ArrayKeyError {
    src: String,
    #[snippet(src)]
    snip: SourceSpan,
    #[highlight(snip, label("{}", self.kind))]
    err_span: SourceSpan,
    kind: ArrayKeyErrorKind,
}

#[derive(Debug, Clone)]
pub enum ArrayKeyErrorKind {
    IntegerExpected,
    NonConsecutive,
}

impl Display for ArrayKeyErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

impl ArrayKeyError {
    pub fn new(kind: ArrayKeyErrorKind, source: &str, err_span: Span) -> Self {
        ArrayKeyError {
            src: source.into(),
            snip: map_span(&(0..source.len())),
            err_span: map_span(&err_span),
            kind,
        }
    }
}

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
                map_span(&(0..source.len())),
                map_span(&(source.len()..source.len())),
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
                map_span(&(0..source.len())),
                map_span(&(source.len()..source.len())),
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
                map_span(&(0..source.len())),
                map_span(&self.span),
            )
            .into())
        }
    }
}

fn map_span(span: &Span) -> SourceSpan {
    SourceSpan::new(
        SourceOffset::from(span.start),
        SourceOffset::from(span.end - span.start),
    )
}

pub trait ResultExt<T> {
    fn with_span(self, span: Span, source: &str) -> Result<T, ParseError>;
}

impl<T, E: Into<PrimitiveErrorKind>> ResultExt<T> for Result<T, E> {
    fn with_span(self, span: Span, source: &str) -> Result<T, ParseError> {
        self.map_err(|error| {
            PrimitiveError {
                src: source.into(),
                snip: map_span(&(0..source.len())),
                err_span: map_span(&span),
                kind: error.into(),
            }
            .into()
        })
    }
}
