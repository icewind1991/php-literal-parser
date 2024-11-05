# php-literal-parser

parser for php literals.

## Usage

Parse into a generic representation

```rust
use php_literal_parser::{from_str, Value, ParseError};

fn main() -> Result<(), ParseError> {
    let map = from_str::<Value>(r#"["foo" => true, "nested" => ['foo' => false]]"#)?;

    assert_eq!(map["foo"], true);
    assert_eq!(map["nested"]["foo"], false);

    Ok(())
}
```

Or parse into a specific struct using serde

```rust
use php_literal_parser::{from_str, ParseError};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Target {
    foo: bool,
    bars: Vec<u8>
}

fn main() -> Result<(), ParseError> {
    let target = from_str(r#"["foo" => true, "bars" => [1, 2, 3, 4,]]"#)?;

    assert_eq!(Target {
        foo: true,
        bars: vec![1, 2, 3, 4]
    }, target);
    Ok(())
}
```
