use maplit::hashmap;
use php_literal_parser::{from_str, Key, ParseError, Value};

fn parse(source: &str) -> Result<Value, ParseError> {
    match from_str(source) {
        Ok(res) => Ok(res),
        Err(err) => {
            eprintln!("{}", err);
            Err(err)
        }
    }
}

#[test]
fn test_parse_value() {
    assert_eq!(Value::Bool(true), parse("true").unwrap());
    assert_eq!(Value::Bool(false), parse("false").unwrap());
    assert_eq!(Value::Int(12), parse("12").unwrap());
    assert_eq!(Value::Int(-1), parse("-1").unwrap());
    assert_eq!(Value::Float(1.12), parse("1.12").unwrap());
    assert_eq!(
        Value::String("test".to_string()),
        parse(r#""test""#).unwrap()
    );
    assert_eq!(Value::Array(hashmap! {}), parse(r#"array()"#).unwrap());
    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(0) => Value::Int(3),
            Key::Int(1) => Value::Int(4),
            Key::Int(2) => Value::Int(5),
        }),
        parse(r#"array(3,4,5)"#).unwrap()
    );
    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(0) => Value::Int(3),
            Key::Int(1) => Value::Int(4),
            Key::Int(2) => Value::Int(5),
        }),
        parse(r#"array(3,4,5,)"#).unwrap()
    );
    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(1) => Value::Int(3),
            Key::Int(3) => Value::Int(4),
            Key::Int(5) => Value::Int(5),
        }),
        parse(r#"array(1=>3,3=>4,5=>5)"#).unwrap()
    );
    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(1) => Value::Int(3),
            Key::Int(2) => Value::Int(4),
            Key::Int(3) => Value::Int(5),
        }),
        parse(r#"array(1=>3,4,5)"#).unwrap()
    );
    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(1) => Value::Int(3),
            Key::Int(2) => Value::Int(4),
            Key::Int(3) => Value::Int(5),
        }),
        parse(r#"array("1"=>3,4,5)"#).unwrap()
    );
    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(1) => Value::Int(3),
            Key::Int(2) => Value::Int(4),
            Key::Int(3) => Value::Int(5),
        }),
        parse(r#"array(1.5=>3,4,5)"#).unwrap()
    );
    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(1) => Value::Int(3),
            Key::Int(2) => Value::Int(4),
            Key::Int(3) => Value::Int(5),
        }),
        parse(r#"array(true=>3,4,5)"#).unwrap()
    );
    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(1) => Value::Int(3),
            Key::String("foo".into()) => Value::Int(4),
            Key::Int(2) => Value::Int(5),
        }),
        parse(r#"array(1=>3,"foo" => 4,5)"#).unwrap()
    );
    assert_eq!(
        Value::Array(hashmap! {
            Key::String("foo".into()) => Value::Bool(true),
            Key::String("nested".into()) => Value::Array(hashmap! {
                Key::String("foo".into()) => Value::Bool(false),
            }),
        }),
        parse(r#"array("foo" => true, "nested" => array ('foo' => false))"#).unwrap()
    );
    assert_eq!(
        Value::Array(hashmap! {
            Key::String("foo".into()) => Value::Bool(true),
            Key::String("nested".into()) => Value::Array(hashmap! {
                Key::String("foo".into()) => Value::Null,
            }),
        }),
        parse(r#"["foo" => true, "nested" => ['foo' => null]]"#).unwrap()
    );
    assert_eq!(Value::Int(-432), parse(r#"-432"#).unwrap());
    assert_eq!(Value::Int(282), parse(r#"0432"#).unwrap());
    assert_eq!(Value::Int(26), parse(r#"0x1A"#).unwrap());
    assert_eq!(Value::Int(3), parse(r#"0b11"#).unwrap());
    assert_eq!(Value::Int(12345), parse(r#"12_34_5"#).unwrap());

    assert_eq!(Value::Bool(true), parse(r#"True"#).unwrap());

    assert_eq!(Value::Float(-432.0), parse(r#"-432.0"#).unwrap());
    assert_eq!(Value::Float(0.12), parse(r#".12"#).unwrap());
    assert_eq!(Value::Float(1000.0), parse(r#"10e2"#).unwrap());
    assert_eq!(Value::Float(1.0), parse(r#"10e-1"#).unwrap());
    assert_eq!(Value::Float(1234.5), parse(r#"12_34.5"#).unwrap());

    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(2) => Value::Int(3),
            Key::String("foo".into()) => Value::Int(4),
            Key::String("".into()) => Value::Int(5),
            Key::Int(1) => Value::Int(6),
            Key::Int(0) => Value::Int(7),
        }),
        parse(r#"array("2"=>3,"foo" => 4, null => 5, true => 6, false => 7)"#).unwrap()
    );

    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(0) => hashmap! {
                Key::String("a".into()) => Value::Int(2),
            }.into(),
            Key::Int(1) => hashmap! {
                Key::String("b".into()) => Value::Int(3),
            }.into()
        }),
        parse(r#"[["a" => 2], ["b" => 3]]"#).unwrap()
    );

    assert_eq!(
        Value::Array(hashmap! {
            Key::Int(0) => hashmap! {
                Key::String("a".into()) => Value::Int(2),
            }.into(),
            Key::Int(1) => hashmap! {
                Key::String("b".into()) => Value::Int(3),
            }.into()
        }),
        parse(r#"array(array("a" => 2), array("b" => 3))"#).unwrap()
    );
}

#[test]
fn test_trailing_semi() {
    assert_eq!(Value::Int(12), parse(r#"12;"#).unwrap());
}
