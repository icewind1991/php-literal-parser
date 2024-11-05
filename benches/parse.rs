use criterion::{black_box, criterion_group, criterion_main, Criterion};
use php_literal_parser::{from_str, Value};

fn perf_parse_int_basic(b: &mut Criterion) {
    let input = "12345676";

    b.bench_function("parse int", |b| {
        b.iter(|| {
            assert_eq!(
                black_box(from_str::<Value>(black_box(input)).unwrap()),
                12345676
            );
        });
    });
}

fn perf_str_double_basic(b: &mut Criterion) {
    let input = r#""aut dolores excepturi rerum est velit ad natus eveniet quo tenetur et fugiat sit velit ipsam nesciunt sint et architecto""#;

    b.bench_function("parse double quote string without escapes", |b| {
        b.iter(|| {
            assert!(black_box(from_str::<Value>(black_box(input)).unwrap()).is_string());
        });
    });
}

fn perf_str_double_escape(b: &mut Criterion) {
    let input = r#""aut dolores excepturi rerum est velit ad natus \"eveniet\" quo tenetur et fugiat sit velit ipsam nesciunt sint et architecto""#;

    b.bench_function("parse double quote escaped string", |b| {
        b.iter(|| {
            assert!(black_box(from_str::<Value>(black_box(input)).unwrap()).is_string());
        });
    });
}

fn perf_str_single_basic(b: &mut Criterion) {
    let input = r#"'aut dolores excepturi rerum est velit ad natus eveniet quo tenetur et fugiat sit velit ipsam nesciunt sint et architecto'"#;

    b.bench_function("parse single quote string without escapes", |b| {
        b.iter(|| {
            assert!(black_box(from_str::<Value>(black_box(input)).unwrap()).is_string());
        });
    });
}

fn perf_str_single_escape(b: &mut Criterion) {
    let input = r#"'aut dolores excepturi rerum est velit ad natus \"eveniet\" quo tenetur et fugiat sit velit ipsam nesciunt sint et architecto'"#;

    b.bench_function("parse single quote escaped string", |b| {
        b.iter(|| {
            assert!(black_box(
                from_str::<Value>(black_box(input)).unwrap().is_string()
            ));
        });
    });
}

criterion_group!(
    benches,
    perf_str_single_escape,
    perf_str_single_basic,
    perf_str_double_escape,
    perf_str_double_basic,
    perf_parse_int_basic
);
criterion_main!(benches);
