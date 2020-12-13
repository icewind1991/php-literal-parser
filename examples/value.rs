use php_literal_parser::{from_str, Value};

fn main() {
    let source = r###"
    array (
        "double" => "quote",
        'single' => 'quote',
        "escaped" => "\"quote\"",
        1 => 2,
        "nested" => [
            "sub" => "key",
        ],
        "array" => [1,2,3,4],
        "bool" => false,
        "negative" => -1,
        "null" => null,
    )
    "###;

    match from_str::<Value>(source) {
        Ok(result) => print!("{:#?}", result),
        Err(err) => eprint!("{}", err.with_source(source)),
    }
}
