use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use hledger_fmt::{format_parsed_journal, parse_content};
use std::fs;
use std::path::Path;

fn benchmark_formatter(c: &mut Criterion) {
    let corpus_files = [
        "fuzz/corpus/basic.journal",
        "fuzz/corpus/cheatsheet.hledger",
        "fuzz/corpus/multi-bank-currencies.journal",
        "fuzz/corpus/multicurrency.journal",
        "fuzz/corpus/stock-trading.journal",
        "fuzz/corpus/timelog.journal",
        "fuzz/corpus/uk-finances.journal",
    ];

    let mut group = c.benchmark_group("format_parsed_journal");
    for file_path in corpus_files.iter() {
        let content = fs::read(file_path).unwrap();
        let journal = parse_content(&content).unwrap();
        let file_name = Path::new(file_path).file_name().unwrap().to_str().unwrap();

        group.bench_with_input(
            BenchmarkId::from_parameter(file_name),
            &journal,
            |b, input| b.iter(|| format_parsed_journal(std::hint::black_box(input))),
        );
    }
    group.finish();
}

// Registramos benchmarks
criterion_group!(benches, benchmark_formatter);
criterion_main!(benches);
