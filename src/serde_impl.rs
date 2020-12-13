use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::Deserialize;

use crate::error::{ExpectToken, ResultExt};
use crate::lexer::{SpannedToken, Token};
use crate::num::ParseIntError;
use crate::parser::{ArraySyntax, Parser};
use crate::{Key, ParseError, RawParseError};
use serde::export::TryFrom;
use std::collections::VecDeque;

type Result<T> = std::result::Result<T, ParseError>;

pub struct Deserializer<'de> {
    parser: Parser<'de>,
    peeked: VecDeque<SpannedToken<'de>>,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer {
            parser: Parser::new(input),
            peeked: Default::default(),
        }
    }
}

/// Parse a php literal
///
/// ## Example
///
/// ```rust
/// use php_literal_parser::{from_str, Value, Key};
/// # use std::fmt::Debug;
/// # use std::error::Error;
///
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let map = from_str::<Value>(r#"["foo" => true, "nested" => ['foo' => false]]"#)?;
///
/// assert_eq!(map["foo"], true);
/// assert_eq!(map["nested"]["foo"], false);
/// # Ok(())
/// # }
/// ```
///
pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.next_token().is_none() {
        Ok(t)
    } else {
        Err(RawParseError::TrailingCharacters.into())
    }
}

impl<'de> Deserializer<'de> {
    fn next_token(&mut self) -> Option<SpannedToken<'de>> {
        self.peeked.pop_front().or_else(|| self.parser.next_token())
    }

    fn peek_token(&mut self) -> Option<&SpannedToken<'de>> {
        if self.peeked.is_empty() {
            let next = self.next_token()?;
            self.peeked.push_back(next)
        }
        self.peeked.front()
    }

    fn eat_token(&mut self) {
        let _ = self.next_token();
    }

    fn parse_bool(&mut self) -> Result<bool> {
        let token = self.next_token().expect_token(&[Token::Bool])?;
        Ok(self.parser.parse_bool_token(token)?)
    }

    fn push_peeked(&mut self, peeked: SpannedToken<'de>) {
        self.peeked.push_back(peeked)
    }

    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: TryFrom<i64>,
    {
        let token = self.next_token().expect_token(&[Token::Integer])?;
        let span = token.span.clone();
        let int = self.parser.parse_int_token(token)?;
        if int < 0 {
            Err(ParseError::new(
                RawParseError::InvalidIntLiteral(ParseIntError::UnexpectedNegative),
                span,
            )
            .into())
        } else {
            Ok(T::try_from(int).map_err(|_| {
                ParseError::new(
                    RawParseError::InvalidIntLiteral(ParseIntError::Overflow),
                    span,
                )
            })?)
        }
    }

    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: TryFrom<i64>,
    {
        let token = self.next_token().expect_token(&[Token::Integer])?;
        let span = token.span.clone();
        Ok(
            T::try_from(self.parser.parse_int_token(token)?).map_err(|_| {
                ParseError::new(
                    RawParseError::InvalidIntLiteral(ParseIntError::Overflow),
                    span,
                )
            })?,
        )
    }

    fn parse_float(&mut self) -> Result<f64> {
        let token = self.next_token().expect_token(&[Token::Float])?;
        Ok(self.parser.parse_float_token(token)?)
    }

    fn parse_string(&mut self) -> Result<String> {
        let token = self.next_token().expect_token(&[Token::LiteralString])?;
        Ok(self.parser.parse_string_token(token)?)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = ParseError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let peek = self.peek_token().expect_token(&[
            Token::Null,
            Token::Bool,
            Token::LiteralString,
            Token::Integer,
            Token::Float,
            Token::Array,
            Token::SquareOpen,
        ])?;
        match peek.token {
            Token::Null => self.deserialize_unit(visitor),
            Token::Bool => self.deserialize_bool(visitor),
            Token::LiteralString => self.deserialize_string(visitor),
            Token::Integer => self.deserialize_i64(visitor),
            Token::Float => self.deserialize_f64(visitor),
            Token::Array | Token::SquareOpen => self.deserialize_map(visitor),
            _ => unreachable!(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    // The `parse_signed` function is generic over the integer type `T` so here
    // it is invoked with `T=i8`. The next 8 methods are similar.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_signed()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_signed()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_signed()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_signed()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_unsigned()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_unsigned()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_unsigned()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_unsigned()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse_float()? as f32)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_float()?)
    }

    // The `Serializer` implementation on the previous page serialized chars as
    // single-character strings so handle that representation here.
    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Parse a string, check that it is one character, call `visit_char`.
        unimplemented!()
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let str = self.parse_string()?;
        visitor.visit_str(str.as_str())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.parse_string()?)
    }

    // The `Serializer` implementation on the previous page serialized byte
    // arrays as JSON arrays of bytes. Handle that representation here.
    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
        // visitor.visit_string(self.parse_string()?.to_vec())
    }

    // An absent optional is represented as the JSON `null` and a present
    // optional is represented as just the contained value.
    //
    // As commented in `Serializer` implementation, this is a lossy
    // representation. For example the values `Some(())` and `None` both
    // serialize as just `null`. Unfortunately this is typically what people
    // expect when working with JSON. Other formats are encouraged to behave
    // more intelligently if possible.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let token = self.peek_token().expect_token(&[
            Token::Null,
            Token::Bool,
            Token::LiteralString,
            Token::Integer,
            Token::Float,
            Token::Array,
            Token::SquareOpen,
        ])?;
        if token.token == Token::Null {
            let _ = self.next_token();
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.next_token().expect_token(&[Token::Null])?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let token = self
            .next_token()
            .expect_token(&[Token::Array, Token::SquareOpen])?;
        let syntax = match token.token {
            Token::Array => {
                self.next_token().expect_token(&[Token::BracketOpen])?;
                ArraySyntax::Long
            }
            Token::SquareOpen => ArraySyntax::Short,
            _ => unreachable!(),
        };

        let value = visitor.visit_seq(ArrayWalker::new(&mut self, syntax))?;
        Ok(value)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let token = self
            .next_token()
            .expect_token(&[Token::Array, Token::SquareOpen])?;
        let syntax = match token.token {
            Token::Array => {
                self.next_token().expect_token(&[Token::BracketOpen])?;
                ArraySyntax::Long
            }
            Token::SquareOpen => ArraySyntax::Short,
            _ => unreachable!(),
        };

        let value = visitor.visit_map(ArrayWalker::new(&mut self, syntax))?;
        Ok(value)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // panic!("a");
        let token = self.peek_token().expect_token(&[
            Token::LiteralString,
            Token::Array,
            Token::SquareOpen,
        ])?;
        match token.token {
            Token::LiteralString => visitor.visit_enum(self.parse_string()?.into_deserializer()),
            Token::Array | Token::SquareOpen => {
                self.eat_token();
                let syntax = if token.token == Token::Array {
                    self.next_token().expect_token(&[Token::BracketOpen])?;
                    ArraySyntax::Long
                } else {
                    ArraySyntax::Short
                };

                let value = visitor.visit_enum(Enum::new(self))?;
                self.next_token().expect_token(&[syntax.close_bracket()])?;
                Ok(value)
            }
            _ => unreachable!(),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct ArrayWalker<'source, 'a> {
    de: &'a mut Deserializer<'source>,
    next_int_key: i64,
    syntax: ArraySyntax,
    done: bool,
}

impl<'source, 'a> ArrayWalker<'source, 'a> {
    pub fn new(de: &'a mut Deserializer<'source>, syntax: ArraySyntax) -> Self {
        ArrayWalker {
            de,
            next_int_key: 0,
            syntax,
            done: false,
        }
    }
}

impl<'de, 'a> SeqAccess<'de> for ArrayWalker<'de, 'a> {
    type Error = ParseError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.done {
            return Ok(None);
        }

        let token = self.de.next_token().expect_token(&[
            Token::Bool,
            Token::Integer,
            Token::Float,
            Token::LiteralString,
            Token::Null,
            Token::Array,
            Token::SquareOpen,
            self.syntax.close_bracket(),
        ])?;

        if token.token == self.syntax.close_bracket() {
            self.done = true;
            return Ok(None);
        }

        let next = self.de.next_token().expect_token(&[
            self.syntax.close_bracket(),
            Token::Comma,
            Token::Arrow,
        ])?;

        let value_token = match next.token.clone() {
            Token::Comma => token,
            Token::Arrow => {
                let span = token.span.clone();
                let key = self.de.parser.parse_array_key(token)?;
                match key {
                    Key::Int(key) if key == self.next_int_key => Ok(()),
                    _ => Err(RawParseError::UnexpectedArrayKey).with_span(span),
                }?;
                self.next_int_key += 1;
                let value = self.de.next_token().expect_token(&[
                    Token::Bool,
                    Token::Integer,
                    Token::Float,
                    Token::LiteralString,
                    Token::Null,
                    Token::Array,
                    Token::SquareOpen,
                ])?;
                let next = self
                    .de
                    .next_token()
                    .expect_token(&[Token::Comma, self.syntax.close_bracket()])?;
                if next.token == self.syntax.close_bracket() {
                    self.done = true;
                }
                value
            }
            peeked_token if peeked_token == self.syntax.close_bracket() => {
                self.done = true;
                token
            }
            _ => unreachable!(),
        };

        // Deserialize an array element.
        self.de.push_peeked(value_token);
        seed.deserialize(&mut *self.de).map(Some)
    }
}

impl<'de, 'a> MapAccess<'de> for ArrayWalker<'de, 'a> {
    type Error = ParseError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.done {
            return Ok(None);
        }

        let token = self.de.next_token().expect_token(&[
            Token::Bool,
            Token::Integer,
            Token::Float,
            Token::LiteralString,
            Token::Null,
            self.syntax.close_bracket(),
        ])?;

        if token.token == self.syntax.close_bracket() {
            self.done = true;
            return Ok(None);
        }

        let next = self.de.next_token().expect_token(&[
            Token::Arrow,
            Token::Comma,
            self.syntax.close_bracket(),
        ])?;

        match next.token {
            Token::Arrow => {
                // Deserialize a map key.
                if let Key::Int(int_key) = self.de.parser.parse_array_key(token.clone())? {
                    self.next_int_key = int_key + 1;
                }
                self.de.push_peeked(token);
                seed.deserialize(&mut *self.de).map(Some)
            }
            _ => {
                // implicit key
                let key = self.next_int_key;
                self.next_int_key += 1;
                self.de.push_peeked(token);
                self.de.push_peeked(next);
                seed.deserialize(format!("{}", key).into_deserializer())
                    .map(Some)
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        self.de.peek_token().expect_token(&[
            Token::Bool,
            Token::Integer,
            Token::Float,
            Token::LiteralString,
            Token::Null,
            Token::Array,
            Token::SquareOpen,
        ])?;

        // Deserialize a map key.
        let value = seed.deserialize(&mut *self.de)?;

        let next = self
            .de
            .next_token()
            .expect_token(&[Token::Comma, self.syntax.close_bracket()])?;

        if next.token == self.syntax.close_bracket() {
            self.done = true;
        }
        Ok(value)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = ParseError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        self.de.next_token().expect_token(&[Token::Arrow])?;
        Ok((val, self))
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = ParseError;

    fn unit_variant(self) -> Result<()> {
        self.de.next_token().expect_token(&[Token::LiteralString])?;
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use serde_derive::Deserialize;

    fn from_str<'a, T>(source: &'a str) -> super::Result<T>
    where
        T: serde::Deserialize<'a>,
    {
        match super::from_str(source) {
            Ok(res) => Ok(res),
            Err(err) => {
                let sourced = err.with_source(source);
                eprintln!("{}", sourced);
                Err(sourced.into_inner())
            }
        }
    }

    #[test]
    fn test_vec() {
        let j = r#"["a","b"]"#;
        let expected: Vec<String> = vec!["a".to_string(), "b".to_string()];
        assert_eq!(expected, from_str::<Vec<String>>(j).unwrap());
    }

    #[test]
    fn test_vec_explicit_keys() {
        let j = r#"[0=>"a", 1=>"b"]"#;
        let expected: Vec<String> = vec!["a".to_string(), "b".to_string()];
        assert_eq!(expected, from_str::<Vec<String>>(j).unwrap());
    }

    #[test]
    fn test_struct() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            seq: Vec<String>,
        }

        let j = r#"["int"=>1,"seq"=>["a","b"]]"#;
        let expected = Test {
            int: 1,
            seq: vec!["a".to_owned(), "b".to_owned()],
        };
        assert_eq!(expected, from_str(j).unwrap());
    }

    #[test]
    fn test_struct_nested() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Inner {
            a: f32,
            b: bool,
        }

        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            nested: Inner,
        }

        let j = r#"["int"=>1,"nested"=>["a" => 1.0, "b" => false]]"#;
        let expected = Test {
            int: 1,
            nested: Inner { a: 1.0, b: false },
        };
        assert_eq!(expected, from_str(j).unwrap());
    }

    #[test]
    fn test_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        enum E {
            Unit,
            Newtype1(u32),
            Newtype2(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let j = r#""Unit""#;
        let expected = E::Unit;
        assert_eq!(expected, from_str(j).unwrap());

        let j = r#"["Newtype1"=>1]"#;
        let expected = E::Newtype1(1);
        assert_eq!(expected, from_str(j).unwrap());

        let j = r#"["Newtype2"=>1]"#;
        let expected = E::Newtype2(1);
        assert_eq!(expected, from_str(j).unwrap());

        let j = r#"["Tuple"=>[1,2]]"#;
        let expected = E::Tuple(1, 2);
        assert_eq!(expected, from_str(j).unwrap());

        let j = r#"["Struct"=>["a"=>1]]"#;
        let expected = E::Struct { a: 1 };
        assert_eq!(expected, from_str(j).unwrap());
    }
}
