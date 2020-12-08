//! Parser for php literals.
//!
//! Allows parsing of php string, bool, number and array literals.
//!
//! ## Example
//!
//! ```rust
//! use php_literal_parser::{parse, Value, Key};
//! # use std::fmt::Debug;
//! # use std::error::Error;
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let map = parse(r#"["foo" => true, "nested" => ['foo' => false]]"#)?;
//!
//! assert_eq!(map["foo"], true);
//! assert_eq!(map["nested"]["foo"], false);
//! # Ok(())
//! # }
//! ```
//!
mod error;
mod lexer;
mod num;
mod parser;
mod string;

pub use error::{ParseError, SpannedError};
pub use parser::parse;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Index;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(HashMap<Key, Value>),
    Null,
}

/// A php value, can be either a bool, int, float, string or array
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

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(val) => write!(f, "{}", val),
            Value::Int(val) => write!(f, "{}", val),
            Value::Float(val) => write!(f, "{}", val),
            Value::String(val) => write!(f, "{}", val),
            Value::Array(val) => {
                write!(f, "[\n")?;
                for (key, value) in val.iter() {
                    write!(f, "\t{} => {},", key, value)?;
                }
                write!(f, "]")
            }
            Value::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
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
            Value::Array(map) => map.index(&Key::Int(index)),
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
