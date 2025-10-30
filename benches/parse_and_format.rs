use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use hledger_fmt::format_journal_bytes;
use std::fs;
use std::path::Path;

fn benchmark_formatter(c: &mut Criterion) {
    let corpus_dir = Path::new("fuzz/corpus");
    let mut corpus_files: Vec<_> = fs::read_dir(corpus_dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    corpus_files.sort();

    let mut group = c.benchmark_group("format_journal");
    for file_path in corpus_files.iter() {
        let content = fs::read(file_path).unwrap();
        let file_name = file_path.file_name().unwrap().to_str().unwrap();

        group.bench_with_input(
            BenchmarkId::from_parameter(file_name),
            &content,
            |b, input| b.iter(|| format_journal_bytes(std::hint::black_box(input))),
        );
    }
    group.finish();
}

// Registramos benchmarks
criterion_group!(benches, benchmark_formatter);
criterion_main!(benches);
