use crate::error::{
    ExpectToken, InvalidArrayKeyError, ParseError, ResultExt, SpannedError, UnexpectedTokenError,
};
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
        .expect_token("bool, int, float, string, array start")
        .with_span(lexer.span(), source)?;
    parse_token(token, source, lexer)
}

pub fn parse_token<'source>(
    token: Token,
    source: &'source str,
    lexer: &mut Lexer<Token>,
) -> Result<Value, SpannedError<'source, ParseError>> {
    let value = match token {
        Token::Bool => parse_literal(token, lexer.slice()).with_span(lexer.span(), source)?,
        Token::Integer => parse_literal(token, lexer.slice()).with_span(lexer.span(), source)?,
        Token::Float => parse_literal(token, lexer.slice()).with_span(lexer.span(), source)?,
        Token::LiteralString => {
            parse_literal(token, lexer.slice()).with_span(lexer.span(), source)?
        }
        Token::Array => Value::Array(parse_array(source, lexer, ArraySyntax::Long)?),
        Token::SquareOpen => Value::Array(parse_array(source, lexer, ArraySyntax::Short)?),
        _ => todo!(),
    };

    Ok(value)
}

fn parse_literal(token: Token, slice: &str) -> Result<Value, ParseError> {
    match token {
        Token::Bool => Ok(Value::Bool(slice.parse()?)),
        Token::Integer => Ok(Value::Int(slice.parse()?)),
        Token::Float => Ok(Value::Float(slice.parse()?)),
        Token::LiteralString => Ok(Value::String(parse_string(slice)?)),
        token => Err(ParseError::UnexpectedToken(UnexpectedTokenError::new(
            "bool, int, float, string, array start",
            Some(token),
        ))),
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

fn parse_array<'source>(
    source: &'source str,
    lexer: &mut Lexer<Token>,
    syntax: ArraySyntax,
) -> Result<HashMap<Key, Value>, SpannedError<'source, ParseError>> {
    let mut builder = ArrayBuilder::default();

    if syntax == ArraySyntax::Long {
        let open = lexer
            .next()
            .expect_token("open bracket")
            .with_span(lexer.span(), source)?;
        if !matches!(open, Token::BracketOpen) {
            return Err(ParseError::UnexpectedToken(UnexpectedTokenError::new(
                "open bracket",
                Some(open),
            )))
            .with_span(lexer.span(), source);
        }
    }

    loop {
        let key_or_value = parse_lexer(source, lexer)?;
        let key_or_value_span = lexer.span();
        let next = lexer
            .next()
            .expect_token("close bracket, comma, arrow")
            .with_span(lexer.span(), source)?;

        match next {
            Token::BracketClose if syntax == ArraySyntax::Long => {
                builder.push_value(key_or_value);
                break;
            }
            Token::SquareClose if syntax == ArraySyntax::Short => {
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
                    .expect_token("close bracket, comma, arrow")
                    .with_span(lexer.span(), source)?
                {
                    Token::BracketClose if syntax == ArraySyntax::Long => {
                        break;
                    }
                    Token::SquareClose if syntax == ArraySyntax::Short => {
                        break;
                    }
                    Token::Comma => {}
                    token => {
                        return Err(ParseError::UnexpectedToken(UnexpectedTokenError::new(
                            "close bracket, comma, arrow",
                            Some(token),
                        )))
                        .with_span(lexer.span(), source)
                    }
                }
            }
            token => {
                return Err(ParseError::UnexpectedToken(UnexpectedTokenError::new(
                    "close bracket, comma, arrow",
                    Some(token),
                )))
                .with_span(lexer.span(), source)
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
