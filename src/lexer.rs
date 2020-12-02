use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[token("array")]
    Array,
    #[regex("true|false")]
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
    #[regex("-?[0-9]*\\.[0-9]+")]
    Float,
    #[regex("-?[0-9]+")]
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
