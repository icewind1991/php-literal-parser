mod ast;
mod error;
mod lexer;
mod string;

pub use ast::{parse, Key, Value};
pub use error::{ParseError, SpannedError};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
