use criterion::{criterion_group, criterion_main, Criterion};
use hledger_fmt::format_journal_bytes;
use std::fs;

fn benchmark_formatter(c: &mut Criterion) {
    let content = fs::read("fuzz/corpus/cheatsheet.hledger").unwrap();
    let input = &content;

    c.bench_function("format_journal", |b| {
        b.iter(|| format_journal_bytes(std::hint::black_box(input)))
    });
}

// Registramos benchmarks
criterion_group!(benches, benchmark_formatter);
criterion_main!(benches);
