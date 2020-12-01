use crate::error::UnexpectedTokenError;
use crate::error::{ExpectToken, InvalidArrayKeyError, ParseError, ResultExt, SpannedError};
use crate::lexer::Token;
use crate::string::{unescape_double, unescape_single, UnescapeError};
use logos::{Lexer, Logos};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(HashMap<Key, Value>),
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Key {
    Int(i64),
    String(String),
}

pub fn parse(source: &str) -> Result<Value, SpannedError<ParseError>> {
    let mut lexer: Lexer<Token> = Token::lexer(source);
    parse_lexer(source, &mut lexer)
}

pub fn parse_lexer<'source>(
    source: &'source str,
    lexer: &mut Lexer<Token>,
) -> Result<Value, SpannedError<'source, ParseError>> {
    let token = lexer
        .next()
        .expect_token(&[
            Token::Bool,
            Token::Integer,
            Token::Float,
            Token::LiteralString,
            Token::Array,
            Token::SquareOpen,
        ])
        .with_span(lexer.span(), source)?;
    let value = match token {
        Token::Bool => parse_literal(token, lexer.slice()).with_span(lexer.span(), source)?,
        Token::Integer => parse_literal(token, lexer.slice()).with_span(lexer.span(), source)?,
        Token::Float => parse_literal(token, lexer.slice()).with_span(lexer.span(), source)?,
        Token::LiteralString => {
            parse_literal(token, lexer.slice()).with_span(lexer.span(), source)?
        }
        Token::Array => Value::Array(parse_array(source, lexer, ArraySyntax::Long)?),
        Token::SquareOpen => Value::Array(parse_array(source, lexer, ArraySyntax::Short)?),
        _ => unreachable!(),
    };

    Ok(value)
}

fn parse_literal(token: Token, slice: &str) -> Result<Value, ParseError> {
    let token = token.expect_token(&[
        Token::Bool,
        Token::Integer,
        Token::Float,
        Token::LiteralString,
    ])?;
    match token {
        Token::Bool => Ok(Value::Bool(slice.parse()?)),
        Token::Integer => Ok(Value::Int(slice.parse()?)),
        Token::Float => Ok(Value::Float(slice.parse()?)),
        Token::LiteralString => Ok(Value::String(parse_string(slice)?)),
        _ => unreachable!(),
    }
}

fn parse_string(literal: &str) -> Result<String, UnescapeError> {
    let single_quote = literal.bytes().next().unwrap() == b'\'';
    let inner = &literal[1..(literal.len()) - 1];

    if single_quote {
        unescape_single(inner)
    } else {
        unescape_double(inner)
    }
}

#[derive(Default)]
struct ArrayBuilder {
    next_int_key: i64,
    data: HashMap<Key, Value>,
}

impl ArrayBuilder {
    fn push_value(&mut self, value: Value) {
        let key = Key::Int(self.next_int_key);
        self.next_int_key += 1;
        self.data.insert(key, value);
    }

    fn push_key_value(&mut self, key: Key, value: Value) {
        if let Key::Int(int) = &key {
            self.next_int_key = int + 1;
        }
        self.data.insert(key, value);
    }
}

#[derive(Eq, PartialEq)]
enum ArraySyntax {
    Short,
    Long,
}

impl ArraySyntax {
    fn close_bracket(&self) -> Token {
        match self {
            ArraySyntax::Long => Token::BracketClose,
            ArraySyntax::Short => Token::SquareClose,
        }
    }
}

fn parse_array<'source>(
    source: &'source str,
    lexer: &mut Lexer<Token>,
    syntax: ArraySyntax,
) -> Result<HashMap<Key, Value>, SpannedError<'source, ParseError>> {
    let mut builder = ArrayBuilder::default();

    if syntax == ArraySyntax::Long {
        lexer
            .next()
            .expect_token(&[Token::BracketOpen])
            .with_span(lexer.span(), source)?;
    }

    loop {
        let key_or_value = match parse_lexer(source, lexer) {
            Ok(value) => value,
            Err(err) => {
                // trailing comma or empty array
                match err.error() {
                    ParseError::UnexpectedToken(UnexpectedTokenError {
                        found: Some(token),
                        ..
                    }) if token == &syntax.close_bracket() => break,
                    _ => return Err(err),
                }
            }
        };
        let key_or_value_span = lexer.span();
        let next = lexer
            .next()
            .expect_token(&[syntax.close_bracket(), Token::Comma, Token::Arrow])
            .with_span(lexer.span(), source)?;

        match next {
            Token::BracketClose => {
                builder.push_value(key_or_value);
                break;
            }
            Token::SquareClose => {
                builder.push_value(key_or_value);
                break;
            }
            Token::Comma => {
                builder.push_value(key_or_value);
            }
            Token::Arrow => {
                let value = parse_lexer(source, lexer)?;
                let key = match key_or_value {
                    Value::Int(int) => Key::Int(int),
                    Value::Float(float) => Key::Int(float as i64),
                    Value::String(str) => Key::String(str),
                    value => {
                        let err = ParseError::InvalidArrayKey(InvalidArrayKeyError(value));
                        let span_err = SpannedError::new(err, key_or_value_span, source);
                        return Err(span_err);
                    }
                };
                builder.push_key_value(key, value);

                match lexer
                    .next()
                    .expect_token(&[syntax.close_bracket(), Token::Comma])
                    .with_span(lexer.span(), source)?
                {
                    Token::BracketClose => {
                        break;
                    }
                    Token::SquareClose => {
                        break;
                    }
                    Token::Comma => {}
                    _ => unreachable!(),
                }
            }
            _ => {
                unreachable!();
            }
        }
    }

    Ok(builder.data)
}

#[test]
fn test_parse() {
    use maplit::hashmap;

    assert_eq!(Value::Bool(true), parse("true").unwrap());
    assert_eq!(Value::Bool(false), parse("false").unwrap());
    assert_eq!(Value::Int(12), parse("12").unwrap());
    assert_eq!(Value::Int(-1), parse("-1").unwrap());
    assert_eq!(Value::Float(1.12), parse("1.12").unwrap());
    assert_eq!(
        Value::String("test".to_string()),
        parse(r#""test""#).unwrap()
    );
    assert_eq!(Value::Array(hashmap! {}), parse(r#"array()"#).unwrap());
    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(0) => Value::Int(3),
            Key::Int(1) => Value::Int(4),
            Key::Int(2) => Value::Int(5),
        }),
        parse(r#"array(3,4,5)"#).unwrap()
    );
    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(0) => Value::Int(3),
            Key::Int(1) => Value::Int(4),
            Key::Int(2) => Value::Int(5),
        }),
        parse(r#"array(3,4,5,)"#).unwrap()
    );
    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(1) => Value::Int(3),
            Key::Int(3) => Value::Int(4),
            Key::Int(5) => Value::Int(5),
        }),
        parse(r#"array(1=>3,3=>4,5=>5)"#).unwrap()
    );
    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(1) => Value::Int(3),
            Key::Int(2) => Value::Int(4),
            Key::Int(3) => Value::Int(5),
        }),
        parse(r#"array(1=>3,4,5)"#).unwrap()
    );
    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(1) => Value::Int(3),
            Key::String("foo".into()) => Value::Int(4),
            Key::Int(2) => Value::Int(5),
        }),
        parse(r#"array(1=>3,"foo" => 4,5)"#).unwrap()
    );
    assert_eq!(
        Value::Array(hashmap! {
            Key::String("foo".into()) => Value::Bool(true),
            Key::String("nested".into()) => Value::Array(hashmap! {
                Key::String("foo".into()) => Value::Bool(false),
            }),
        }),
        parse(r#"array("foo" => true, "nested" => array ('foo' => false))"#).unwrap()
    );
    assert_eq!(
        Value::Array(hashmap! {
            Key::String("foo".into()) => Value::Bool(true),
            Key::String("nested".into()) => Value::Array(hashmap! {
                Key::String("foo".into()) => Value::Bool(false),
            }),
        }),
        parse(r#"["foo" => true, "nested" => ['foo' => false]]"#).unwrap()
    );
}