use main_error::MainError;
use php_literal_parser::from_str;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Target {
    foo: bool,
    bars: Vec<u8>,
}

fn main() -> Result<(), MainError> {
    let target = from_str(r#"["foo" => true, "bars" => [1, 2, 3, 4]]"#)?;

    assert_eq!(
        Target {
            foo: true,
            bars: vec![1, 2, 3, 4]
        },
        target
    );
    Ok(())
}
