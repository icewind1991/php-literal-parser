use crate::error::{ExpectToken, ParseError, ResultExt, SpannedError};
use crate::lexer::{SpannedToken, Token, TokenStream};
use crate::num::parse_int;
use crate::string::{is_array_key_numeric, parse_string};
use crate::{Key, Value};
use logos::Logos;
use std::collections::HashMap;
use std::num::ParseFloatError;

/// Parse a php literal
///
/// ## Example
///
/// ```rust
/// use php_literal_parser::{parse, Value, Key};
/// # use std::fmt::Debug;
/// # use std::error::Error;
///
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let map = parse(r#"["foo" => true, "nested" => ['foo' => false]]"#)?;
///
/// assert_eq!(map["foo"], true);
/// assert_eq!(map["nested"]["foo"], false);
/// # Ok(())
/// # }
/// ```
///
pub fn parse(source: &str) -> Result<Value, SpannedError<ParseError>> {
    Parser::new(source).run()
}

pub struct Parser<'source> {
    source: &'source str,
    tokens: TokenStream<'source>,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source str) -> Self {
        Parser {
            source,
            tokens: TokenStream::new(Token::lexer(source)),
        }
    }

    pub fn run(&mut self) -> Result<Value, SpannedError<ParseError>> {
        let token = self.tokens.next().expect_token(&[
            Token::Bool,
            Token::Integer,
            Token::Float,
            Token::LiteralString,
            Token::Null,
            Token::Array,
            Token::SquareOpen,
        ])?;
        self.parse_any(token)
    }

    pub fn parse_any(&mut self, token: SpannedToken) -> Result<Value, SpannedError<ParseError>> {
        let value = match token.token {
            Token::Bool => Value::Bool(self.parse_bool(token)?),
            Token::Integer => Value::Int(self.parse_int(token)?),
            Token::Float => Value::Float(self.parse_float(token)?),
            Token::LiteralString => Value::String(self.parse_string(token)?),
            Token::Null => Value::Null,
            Token::Array => Value::Array(self.parse_array(ArraySyntax::Long)?),
            Token::SquareOpen => Value::Array(self.parse_array(ArraySyntax::Short)?),
            _ => unreachable!(),
        };

        Ok(value)
    }

    pub fn parse_bool(&self, token: SpannedToken) -> Result<bool, SpannedError<ParseError>> {
        token
            .slice()
            .to_ascii_lowercase()
            .parse()
            .with_span(token.span)
    }

    pub fn parse_int(&self, token: SpannedToken) -> Result<i64, SpannedError<ParseError>> {
        parse_int(token.slice()).with_span(token.span)
    }

    pub fn parse_float(&self, token: SpannedToken) -> Result<f64, SpannedError<ParseError>> {
        parse_float(token.slice()).with_span(token.span)
    }

    pub fn parse_string(&self, token: SpannedToken) -> Result<String, SpannedError<ParseError>> {
        parse_string(token.slice()).with_span(token.span)
    }

    pub fn parse_array(
        &mut self,
        syntax: ArraySyntax,
    ) -> Result<HashMap<Key, Value>, SpannedError<ParseError>> {
        let mut builder = ArrayBuilder::default();

        if syntax == ArraySyntax::Long {
            self.tokens.next().expect_token(&[Token::BracketOpen])?;
        }

        loop {
            let key_value_or_close_token = self.tokens.next().expect_token(&[
                Token::Bool,
                Token::Integer,
                Token::Float,
                Token::LiteralString,
                Token::Null,
                Token::Array,
                Token::SquareOpen,
                syntax.close_bracket(),
            ])?;

            // trailing comma or empty array
            if key_value_or_close_token.token == syntax.close_bracket() {
                break;
            }
            let next = self.tokens.next().expect_token(&[
                syntax.close_bracket(),
                Token::Comma,
                Token::Arrow,
            ])?;

            match next.token {
                Token::BracketClose => {
                    builder.push_value(self.parse_any(key_value_or_close_token)?);
                    break;
                }
                Token::SquareClose => {
                    builder.push_value(self.parse_any(key_value_or_close_token)?);
                    break;
                }
                Token::Comma => {
                    builder.push_value(self.parse_any(key_value_or_close_token)?);
                }
                Token::Arrow => {
                    let key = self.parse_array_key(key_value_or_close_token)?;
                    let value = self.run()?;
                    builder.push_key_value(key, value);

                    match self
                        .tokens
                        .next()
                        .expect_token(&[syntax.close_bracket(), Token::Comma])?
                        .token
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

    pub fn parse_array_key(
        &mut self,
        token: SpannedToken,
    ) -> Result<Key, SpannedError<ParseError>> {
        let token = token.expect_token(&[
            Token::Bool,
            Token::Integer,
            Token::Float,
            Token::LiteralString,
            Token::Null,
        ])?;
        Ok(match self.parse_any(token)? {
            Value::Int(int) => Key::Int(int),
            Value::Float(float) => Key::Int(float as i64),
            Value::String(str) if is_array_key_numeric(&str) => Key::Int(parse_int(&str).unwrap()),
            Value::String(str) => Key::String(str),
            Value::Bool(bool) => Key::Int(if bool { 1 } else { 0 }),
            Value::Null => Key::String(String::from("")),
            _ => unreachable!(),
        })
    }
}

fn parse_float(literal: &str) -> Result<f64, ParseFloatError> {
    let stripped = literal.replace('_', "");
    stripped.parse()
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
pub enum ArraySyntax {
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
                Key::String("foo".into()) => Value::Null,
            }),
        }),
        parse(r#"["foo" => true, "nested" => ['foo' => null]]"#).unwrap()
    );
    assert_eq!(Value::Int(-432), parse(r#"-432"#).unwrap());
    assert_eq!(Value::Int(282), parse(r#"0432"#).unwrap());
    assert_eq!(Value::Int(26), parse(r#"0x1A"#).unwrap());
    assert_eq!(Value::Int(3), parse(r#"0b11"#).unwrap());
    assert_eq!(Value::Int(12345), parse(r#"12_34_5"#).unwrap());

    assert_eq!(Value::Bool(true), parse(r#"True"#).unwrap());

    assert_eq!(Value::Float(-432.0), parse(r#"-432.0"#).unwrap());
    assert_eq!(Value::Float(0.12), parse(r#".12"#).unwrap());
    assert_eq!(Value::Float(1000.0), parse(r#"10e2"#).unwrap());
    assert_eq!(Value::Float(1.0), parse(r#"10e-1"#).unwrap());
    assert_eq!(Value::Float(1234.5), parse(r#"12_34.5"#).unwrap());

    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(2) => Value::Int(3),
            Key::String("foo".into()) => Value::Int(4),
            Key::String("".into()) => Value::Int(5),
            Key::Int(1) => Value::Int(6),
            Key::Int(0) => Value::Int(7),
        }),
        parse(r#"array("2"=>3,"foo" => 4, null => 5, true => 6, false => 7)"#).unwrap()
    );
}
