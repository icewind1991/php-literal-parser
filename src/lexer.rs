use logos::Logos;
use std::fmt::{Display, Formatter};

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[token("array")]
    Array,
    #[regex("true|false")]
    Bool,
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

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Array => write!(f, "array"),
            Token::Bool => write!(f, "bool"),
            Token::Arrow => write!(f, "arrow"),
            Token::BracketOpen => write!(f, "bracket open"),
            Token::BracketClose => write!(f, "bracket close"),
            Token::SquareOpen => write!(f, "square bracket open"),
            Token::SquareClose => write!(f, "square bracket close"),
            Token::Comma => write!(f, "comma"),
            Token::LiteralString => write!(f, "string literal"),
            Token::Float => write!(f, "float literal"),
            Token::Integer => write!(f, "integer literal"),
            Token::Error => write!(f, "invalid token"),
        }
    }
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

    assert_eq!(lex.next(), Some(Token::BracketClose));

    assert_eq!(lex.next(), None);
}
