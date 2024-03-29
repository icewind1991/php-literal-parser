//! Parser for php literals.
//!
//! Allows parsing of php string, bool, number and array literals.
//!
//! ## Usage
//!
//! Parse into a generic representation
//!
//! ```rust
//! use php_literal_parser::{from_str, Value};
//! # use std::fmt::Debug;
//! # use std::error::Error;
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let map = from_str::<Value>(r#"["foo" => true, "nested" => ['foo' => false]]"#)?;
//!
//! assert_eq!(map["foo"], true);
//! assert_eq!(map["nested"]["foo"], false);
//! # Ok(())
//! # }
//! ```
//!
//! Or parse into a specific struct using serde
//!
//! ```rust
//! use php_literal_parser::from_str;
//! use serde::Deserialize;
//! # use std::fmt::Debug;
//! # use std::error::Error;
//!
//! #[derive(Debug, Deserialize, PartialEq)]
//! struct Target {
//!     foo: bool,
//!     bars: Vec<u8>
//! }
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let target = from_str(r#"["foo" => true, "bars" => [1, 2, 3, 4,]]"#)?;
//!
//! assert_eq!(Target {
//!     foo: true,
//!     bars: vec![1,2,3,4]
//! }, target);
//! # Ok(())
//! # }
//! ```
//!
#![forbid(unsafe_code)]
mod error;
mod lexer;
mod num;
mod parser;
mod serde_impl;
mod string;

use crate::string::is_array_key_numeric;
pub use error::ParseError;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
pub use serde_impl::from_str;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Index;

/// A php value, can be either a bool, int, float, string, an array or null
///
/// note that in php all arrays are associative and thus represented by a map in rust.
///
/// You can convert a `Value` into a regular rust type by pattern matching or using the `into_` functions.
///
/// ## Indexing
///
/// If the value is a php array, you can directly index into the `Value`, this will null if the `Value` is not an array
/// or the key is not found
///
/// ```rust
/// # use maplit::hashmap;
/// # use php_literal_parser::Value;
/// #
/// # fn main() {
/// let value = Value::Array(hashmap!{
///     "key".into() => "value".into(),
///     10.into() => false.into()
/// });
/// assert_eq!(value["key"], "value");
/// assert_eq!(value[10], false);
/// assert!(value["not"]["found"].is_null());
/// # }
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(HashMap<Key, Value>),
    Null,
}

impl Value {
    /// Check if the value is a bool
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    /// Check if the value is an integer
    pub fn is_int(&self) -> bool {
        matches!(self, Value::Int(_))
    }

    /// Check if the value is a float
    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    /// Check if the value is a string
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Check if the value is an array
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Convert the value into a bool if it is one
    pub fn into_bool(self) -> Option<bool> {
        match self {
            Value::Bool(bool) => Some(bool),
            _ => None,
        }
    }

    /// Convert the value into a int if it is one
    pub fn into_int(self) -> Option<i64> {
        match self {
            Value::Int(int) => Some(int),
            _ => None,
        }
    }

    /// Convert the value into a float if it is one
    pub fn into_float(self) -> Option<f64> {
        match self {
            Value::Float(float) => Some(float),
            _ => None,
        }
    }

    /// Convert the value into a string if it is one
    pub fn into_string(self) -> Option<String> {
        match self {
            Value::String(string) => Some(string),
            _ => None,
        }
    }

    /// Convert the value into a hashmap if it is one
    pub fn into_hashmap(self) -> Option<HashMap<Key, Value>> {
        match self {
            Value::Array(map) => Some(map),
            _ => None,
        }
    }

    /// Get the value as &str if it is a string
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(str) => Some(str.as_str()),
            _ => None,
        }
    }

    /// Get the value as i64 if it is an int
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(int) => Some(*int),
            _ => None,
        }
    }

    /// Get the value as f64 if it is a float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(float) => Some(*float),
            _ => None,
        }
    }

    /// Iterate over array key and value pairs if it is an array
    pub fn iter(&self) -> impl Iterator<Item = (&Key, &Value)> {
        let map = match self {
            Value::Array(map) => Some(map),
            _ => None,
        };
        map.into_iter().flat_map(|map| map.iter())
    }

    /// Iterate over array keys if it is an array
    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        self.iter().map(|(key, _value)| key)
    }

    /// Iterate over array values if it is an array
    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.iter().map(|(_key, value)| value)
    }
}

impl PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        match self {
            Value::Bool(bool) => bool == other,
            _ => false,
        }
    }
}

impl PartialEq<i64> for Value {
    fn eq(&self, other: &i64) -> bool {
        match self {
            Value::Int(int) => int == other,
            _ => false,
        }
    }
}

impl PartialEq<f64> for Value {
    fn eq(&self, other: &f64) -> bool {
        match self {
            Value::Float(float) => float == other,
            _ => false,
        }
    }
}

impl PartialEq<String> for Value {
    fn eq(&self, other: &String) -> bool {
        match self {
            Value::String(str) => str == other,
            _ => false,
        }
    }
}

impl PartialEq<&str> for Value {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Value::String(str) => str.as_str() == *other,
            _ => false,
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Int(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.into())
    }
}

impl From<HashMap<Key, Value>> for Value {
    fn from(value: HashMap<Key, Value>) -> Self {
        Value::Array(value)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(val) => write!(f, "{}", val),
            Value::Int(val) => write!(f, "{}", val),
            Value::Float(val) => write!(f, "{}", val),
            Value::String(val) => write!(f, "{}", val),
            Value::Array(val) => {
                writeln!(f, "[")?;
                for (key, value) in val.iter() {
                    write!(f, "\t{} => {},", key, value)?;
                }
                write!(f, "]")
            }
            Value::Null => write!(f, "null"),
        }
    }
}

/// A php array key, can be either an int or string
#[derive(Debug, Eq, Clone)]
pub enum Key {
    Int(i64),
    String(String),
}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Key::Int(int) => int.hash(state),
            Key::String(str) => str.hash(state),
        }
    }
}

impl From<i64> for Key {
    fn from(int: i64) -> Self {
        Key::Int(int)
    }
}

impl From<String> for Key {
    fn from(str: String) -> Self {
        Key::String(str)
    }
}

impl From<&str> for Key {
    fn from(str: &str) -> Self {
        Key::String(str.into())
    }
}

impl Key {
    /// Check if the key is an integer
    pub fn is_int(&self) -> bool {
        matches!(self, Key::Int(_))
    }

    /// Check if the key is a string
    pub fn is_string(&self) -> bool {
        matches!(self, Key::String(_))
    }

    /// Convert the key into a bool if it is one
    pub fn into_int(self) -> Option<i64> {
        match self {
            Key::Int(int) => Some(int),
            _ => None,
        }
    }

    /// Convert the key into a string if it is one
    pub fn into_string(self) -> Option<String> {
        match self {
            Key::String(string) => Some(string),
            _ => None,
        }
    }

    /// Get the key as &str if it is a string
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Key::String(str) => Some(str.as_str()),
            _ => None,
        }
    }

    /// Get the key as i64 if it is an int
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Key::Int(int) => Some(*int),
            _ => None,
        }
    }
}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Key::Int(self_int), Key::Int(other_int)) => self_int.cmp(other_int),
            (Key::String(self_string), Key::String(other_string)) => self_string.cmp(other_string),
            (Key::String(self_string), Key::Int(other_int)) => {
                self_string.cmp(&other_int.to_string())
            }
            (Key::Int(self_int), Key::String(other_string)) => {
                self_int.to_string().cmp(other_string)
            }
        }
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Key::Int(self_int), Key::Int(other_int)) => self_int.eq(other_int),
            (Key::String(self_string), Key::String(other_string)) => self_string.eq(other_string),
            (Key::String(self_string), Key::Int(other_int)) => {
                self_string.eq(&other_int.to_string())
            }
            (Key::Int(self_int), Key::String(other_string)) => {
                self_int.to_string().eq(other_string)
            }
        }
    }
}

impl Borrow<str> for Key {
    fn borrow(&self) -> &str {
        match self {
            Key::String(str) => str.as_str(),
            _ => panic!(),
        }
    }
}

impl<Q: ?Sized> Index<&Q> for Value
where
    Key: Borrow<Q>,
    Q: Eq + Hash,
{
    type Output = Value;

    fn index(&self, index: &Q) -> &Self::Output {
        match self {
            Value::Array(map) => map.get(index).unwrap_or(&Value::Null),
            _ => &Value::Null,
        }
    }
}

impl Index<Key> for Value {
    type Output = Value;

    fn index(&self, index: Key) -> &Self::Output {
        match self {
            Value::Array(map) => map.get(&index).unwrap_or(&Value::Null),
            _ => &Value::Null,
        }
    }
}

impl Index<i64> for Value {
    type Output = Value;

    fn index(&self, index: i64) -> &Self::Output {
        match self {
            Value::Array(map) => map.get(&Key::Int(index)).unwrap_or(&Value::Null),
            _ => &Value::Null,
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::Int(val) => write!(f, "{}", val),
            Key::String(val) => write!(f, "{}", val),
        }
    }
}

#[test]
fn test_index() {
    use maplit::hashmap;
    let map = Value::Array(hashmap! {
        Key::String("key".to_string()) => Value::String("value".to_string()),
        Key::Int(1) => Value::Bool(true),
    });
    assert_eq!(map["key"], "value");
    assert_eq!(map[1], true);
    assert_eq!(map[Key::Int(1)], true);
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("any php literal")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Bool(v))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Int(v.into()))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Int(v.into()))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Int(v.into()))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Int(v))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Int(v.into()))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Int(v.into()))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Int(v.into()))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Int(v.try_into().map_err(|_| {
            E::custom(format!("i64 out of range: {}", v))
        })?))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Float(v.into()))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Float(v))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(v.to_string()))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(v.into()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(v.into()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(v))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Null)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, <A as SeqAccess<'de>>::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut result = HashMap::new();
        let mut next_key = 0;
        while let Some(value) = seq.next_element::<Value>()? {
            let key = Key::Int(next_key);
            next_key += 1;
            result.insert(key, value);
        }
        Ok(Value::Array(result))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
    where
        A: MapAccess<'de>,
    {
        let mut result = HashMap::new();
        while let Some((key, value)) = map.next_entry()? {
            result.insert(key, value);
        }
        Ok(Value::Array(result))
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

struct KeyVisitor;

impl<'de> Visitor<'de> for KeyVisitor {
    type Value = Key;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string, number, bool or null")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Key::Int(if v { 1 } else { 0 }))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Key::Int(v.into()))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Key::Int(v.into()))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Key::Int(v.into()))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Key::Int(v))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Key::Int(v.into()))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Key::Int(v.into()))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Key::Int(v.into()))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Key::Int(v.try_into().map_err(|_| {
            E::custom(format!("i64 out of range: {}", v))
        })?))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Key::Int(v as i64))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Key::Int(v as i64))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Key::String(v.to_string()))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(v.into())
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(v.into())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if is_array_key_numeric(&v) {
            Ok(Key::Int(v.parse().unwrap()))
        } else {
            Ok(Key::String(v))
        }
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Key::String(String::from("")))
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Key, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(KeyVisitor)
    }
}
