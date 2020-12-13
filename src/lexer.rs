use logos::{Lexer, Logos, Span};
use serde::export::fmt::Debug;
use serde::export::Formatter;

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
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

#[derive(Clone)]
pub struct SpannedToken<'source> {
    pub token: Token,
    pub span: Span,
    pub source: &'source str,
}

impl<'source> SpannedToken<'source> {
    pub fn slice(&self) -> &'source str {
        &self.source[self.span.clone()]
    }
}

impl<'source> Debug for SpannedToken<'source> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SpannedToken {{ {:?}: \"{}\"}} ",
            self.token,
            self.slice()
        )
    }
}

pub struct TokenStream<'source> {
    lexer: Lexer<'source, Token>,
}

impl<'source> TokenStream<'source> {
    pub fn new(lexer: Lexer<'source, Token>) -> Self {
        TokenStream { lexer }
    }
}

impl<'source> Iterator for TokenStream<'source> {
    type Item = SpannedToken<'source>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.lexer.next()?;
        Some(SpannedToken {
            token,
            span: self.lexer.span(),
            source: self.lexer.source(),
        })
    }
}
