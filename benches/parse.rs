use criterion::{criterion_group, criterion_main, Criterion};
use hledger_fmt::parse_content;
use std::fs;

fn benchmark_parser(c: &mut Criterion) {
    let content = fs::read("fuzz/corpus/cheatsheet.hledger").unwrap();
    let input = &content;

    c.bench_function("parse_content", |b| {
        b.iter(|| parse_content(std::hint::black_box(input)))
    });
}

// Registramos benchmarks
criterion_group!(benches, benchmark_parser);
criterion_main!(benches);
