# php-literal-parser

parser for php literals.

## Usage

```rust
use php_literal_parser::parse;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let map = parse(r#"["foo" => true, "nested" => ['foo' => false]]"#)?;
    assert_eq!(map["foo"], true);
    assert_eq!(map["nested"]["foo"], false);
    Ok(())
}
```