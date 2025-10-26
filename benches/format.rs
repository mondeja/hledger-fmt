use criterion::{criterion_group, criterion_main, Criterion};
use hledger_fmt::{format_parsed_journal, parse_content};
use std::fs;

fn benchmark_formatter(c: &mut Criterion) {
    let content = fs::read("fuzz/corpus/cheatsheet.hledger").unwrap();
    let journal = parse_content(&content).unwrap();
    let input = &journal;

    c.bench_function("parse_content", |b| {
        b.iter(|| format_parsed_journal(std::hint::black_box(input)))
    });
}

// Registramos benchmarks
criterion_group!(benches, benchmark_formatter);
criterion_main!(benches);
