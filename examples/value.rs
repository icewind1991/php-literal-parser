use miette::DiagnosticResult;
use php_literal_parser::{from_str, Value};

fn main() -> DiagnosticResult<()> {
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

    println!("{:#?}", from_str::<Value>(source)?);
    Ok(())
}
