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
