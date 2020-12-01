mod ast;
mod error;
mod lexer;
mod string;

pub use ast::parse;
pub use error::{ParseError, SpannedError};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Index;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(HashMap<Key, Value>),
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

impl PartialEq<str> for Value {
    fn eq(&self, other: &str) -> bool {
        match self {
            Value::String(str) => str.as_str() == other,
            _ => false,
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
            Value::Array(map) => map.index(index),
            _ => panic!("index into non array value"),
        }
    }
}

impl Index<Key> for Value {
    type Output = Value;

    fn index(&self, index: Key) -> &Self::Output {
        match self {
            Value::Array(map) => map.index(&index),
            _ => panic!("index into non array value"),
        }
    }
}

impl Index<i64> for Value {
    type Output = Value;

    fn index(&self, index: i64) -> &Self::Output {
        match self {
            Value::Array(map) => map.index(&Key::Int(index)),
            _ => panic!("index into non array value"),
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
    assert_eq!(map["key"], Value::String("value".to_string()));
    assert_eq!(map[1], Value::Bool(true));
    assert_eq!(map[Key::Int(1)], Value::Bool(true));
}
