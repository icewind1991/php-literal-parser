use php_literal_parser::parse;

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
    )
    "###;

    match parse(source) {
        Ok(result) => print!("{:#?}", result),
        Err(err) => eprint!("{}", err),
    }
}
