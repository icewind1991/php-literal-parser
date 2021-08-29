use crate::error::{ExpectToken, ParseError, ResultExt};
use crate::lexer::{SpannedToken, Token, TokenStream};
use crate::num::parse_int;
use crate::string::{is_array_key_numeric, parse_string};
use crate::{Key, Value};
use logos::Logos;
use std::iter::Peekable;
use std::num::ParseFloatError;

pub struct Parser<'source> {
    source: &'source str,
    tokens: Peekable<TokenStream<'source>>,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source str) -> Self {
        Parser {
            source,
            tokens: TokenStream::new(Token::lexer(source)).peekable(),
        }
    }

    pub fn next_token(&mut self) -> Option<SpannedToken<'source>> {
        self.tokens.next()
    }

    pub fn parse_literal(&self, token: SpannedToken) -> Result<Value, ParseError> {
        let value = match token.token {
            Token::Bool => Value::Bool(self.parse_bool_token(token)?),
            Token::Integer => Value::Int(self.parse_int_token(token)?),
            Token::Float => Value::Float(self.parse_float_token(token)?),
            Token::LiteralString => Value::String(self.parse_string_token(token)?),
            Token::Null => Value::Null,
            _ => unreachable!(),
        };

        Ok(value)
    }

    pub fn parse_bool_token(&self, token: SpannedToken) -> Result<bool, ParseError> {
        token
            .slice()
            .to_ascii_lowercase()
            .parse()
            .with_span(token.span, token.source)
    }

    pub fn parse_int_token(&self, token: SpannedToken) -> Result<i64, ParseError> {
        parse_int(token.slice()).with_span(token.span, token.source)
    }

    pub fn parse_float_token(&self, token: SpannedToken) -> Result<f64, ParseError> {
        parse_float(token.slice()).with_span(token.span, token.source)
    }

    pub fn parse_string_token(&self, token: SpannedToken) -> Result<String, ParseError> {
        parse_string(token.slice()).with_span(token.span, token.source)
    }

    pub fn parse_array_key(&self, token: SpannedToken) -> Result<Key, ParseError> {
        let token = token.expect_token(
            &[
                Token::Bool,
                Token::Integer,
                Token::Float,
                Token::LiteralString,
                Token::Null,
            ],
            self.source,
        )?;
        Ok(match self.parse_literal(token)? {
            Value::Int(int) => Key::Int(int),
            Value::Float(float) => Key::Int(float as i64),
            Value::String(str) if is_array_key_numeric(&str) => Key::Int(parse_int(&str).unwrap()),
            Value::String(str) => Key::String(str),
            Value::Bool(bool) => Key::Int(if bool { 1 } else { 0 }),
            Value::Null => Key::String(String::from("")),
            _ => unreachable!(),
        })
    }

    pub fn source(&self) -> &'source str {
        self.source
    }
}

fn parse_float(literal: &str) -> Result<f64, ParseFloatError> {
    let stripped = literal.replace('_', "");
    stripped.parse()
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ArraySyntax {
    Short,
    Long,
}

impl ArraySyntax {
    pub fn close_bracket(&self) -> Token {
        match self {
            ArraySyntax::Long => Token::BracketClose,
            ArraySyntax::Short => Token::SquareClose,
        }
    }
}
