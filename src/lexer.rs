use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[token("array")]
    Array,
    #[regex("(?i:true|false)")]
    Bool,
    #[regex("null")]
    Null,
    #[token("=>")]
    Arrow,
    #[token("(")]
    BracketOpen,
    #[token(")")]
    BracketClose,
    #[token("[")]
    SquareOpen,
    #[token("]")]
    SquareClose,
    #[token(",")]
    Comma,
    #[regex("(\"([^\"\\\\]|\\\\.)*\")|(\'([^\'\\\\]|\\\\.)*\')")]
    LiteralString,
    #[regex("-?((([0-9]+(_[0-9]+)*|([0-9]*(_[0-9]+)*[\\.][0-9]+(_[0-9]+)*)|([0-9]+(_[0-9]+)*[\\.][0-9]*(_[0-9]+)*)))[eE][+-]?[0-9]+(_[0-9]+)*|([0-9]*(_[0-9]+)*[\\.][0-9]+(_[0-9]+)*)|([0-9]+(_[0-9]+)*[\\.][0-9]*(_[0-9]+)*))")]
    Float,
    #[regex("-?(0|[1-9][0-9]*(_[0-9]+)*|0[xX][0-9a-fA-F]+(_[0-9a-fA-F]+)*|0[0-7]+(_[0-7]+)*|0[bB][01]+(_[01]+)*)")]
    Integer,
    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[test]
fn test_lex() {
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
    let mut lex = Token::lexer(source);

    assert_eq!(lex.next(), Some(Token::Array));
    assert_eq!(lex.next(), Some(Token::BracketOpen));

    assert_eq!(lex.next(), Some(Token::LiteralString));
    assert_eq!(lex.next(), Some(Token::Arrow));
    assert_eq!(lex.next(), Some(Token::LiteralString));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::LiteralString));
    assert_eq!(lex.next(), Some(Token::Arrow));
    assert_eq!(lex.next(), Some(Token::LiteralString));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::LiteralString));
    assert_eq!(lex.next(), Some(Token::Arrow));
    assert_eq!(lex.next(), Some(Token::LiteralString));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::Integer));
    assert_eq!(lex.next(), Some(Token::Arrow));
    assert_eq!(lex.next(), Some(Token::Integer));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::LiteralString));
    assert_eq!(lex.next(), Some(Token::Arrow));
    assert_eq!(lex.next(), Some(Token::SquareOpen));

    assert_eq!(lex.next(), Some(Token::LiteralString));
    assert_eq!(lex.next(), Some(Token::Arrow));
    assert_eq!(lex.next(), Some(Token::LiteralString));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::SquareClose));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::LiteralString));
    assert_eq!(lex.next(), Some(Token::Arrow));
    assert_eq!(lex.next(), Some(Token::SquareOpen));
    assert_eq!(lex.next(), Some(Token::Integer));
    assert_eq!(lex.next(), Some(Token::Comma));
    assert_eq!(lex.next(), Some(Token::Integer));
    assert_eq!(lex.next(), Some(Token::Comma));
    assert_eq!(lex.next(), Some(Token::Integer));
    assert_eq!(lex.next(), Some(Token::Comma));
    assert_eq!(lex.next(), Some(Token::Integer));
    assert_eq!(lex.next(), Some(Token::SquareClose));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::LiteralString));
    assert_eq!(lex.next(), Some(Token::Arrow));
    assert_eq!(lex.next(), Some(Token::Bool));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::LiteralString));
    assert_eq!(lex.next(), Some(Token::Arrow));
    assert_eq!(lex.next(), Some(Token::Integer));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::LiteralString));
    assert_eq!(lex.next(), Some(Token::Arrow));
    assert_eq!(lex.next(), Some(Token::Null));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::BracketClose));

    assert_eq!(lex.next(), None);
}

#[test]
fn test_lex_int() {
    let source = r###"0,123,0x123,0123,0b111,12_34_56"###;
    let mut lex = Token::lexer(source);

    assert_eq!(lex.next(), Some(Token::Integer));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::Integer));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::Integer));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::Integer));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::Integer));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::Integer));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_lex_float() {
    let source = r###".1,123.0,123e1,123e+1,123e-1,1_23.456"###;
    let mut lex = Token::lexer(source);

    assert_eq!(lex.next(), Some(Token::Float));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::Float));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::Float));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::Float));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::Float));
    assert_eq!(lex.next(), Some(Token::Comma));

    assert_eq!(lex.next(), Some(Token::Float));
    assert_eq!(lex.next(), None);
}
