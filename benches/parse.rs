#![feature(test)]

extern crate test;

use php_literal_parser::parse;
use test::Bencher;

#[bench]
fn perf_parse_int_basic(b: &mut Bencher) {
    let input = "12345676";

    b.iter(|| {
        assert_eq!(parse(input).unwrap(), 12345676);
    });
}

#[bench]
fn perf_str_basic(b: &mut Bencher) {
    let input = r#""aut dolores excepturi rerum est velit ad natus eveniet quo tenetur et fugiat sit velit ipsam nesciunt sint et architecto""#;

    b.iter(|| {
        assert!(parse(input).unwrap().is_string());
    });
}

#[bench]
fn perf_str_escape(b: &mut Bencher) {
    let input = r#""aut dolores excepturi rerum est velit ad natus \"eveniet\" quo tenetur et fugiat sit velit ipsam nesciunt sint et architecto""#;

    b.iter(|| {
        assert!(parse(input).unwrap().is_string());
    });
}
