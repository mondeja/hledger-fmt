use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use hledger_fmt::parse_content;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn benchmark_parser(c: &mut Criterion) {
    let mut corpus_files = collect_corpus_files();
    corpus_files.sort();

    let mut group = c.benchmark_group("parse_content");
    for file_path in corpus_files.iter() {
        let content = fs::read(file_path).unwrap();
        let file_name = file_path.file_name().unwrap().to_str().unwrap();

        group.bench_with_input(
            BenchmarkId::from_parameter(file_name),
            &content,
            |b, input| b.iter(|| parse_content(std::hint::black_box(input))),
        );
    }
    group.finish();
}

fn collect_corpus_files() -> Vec<PathBuf> {
    let corpus_dir = Path::new("fuzz/corpus");
    let mut corpus_files: Vec<_> = fs::read_dir(corpus_dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            path.is_file().then_some(path)
        })
        .collect();

    if let Ok(filter) = std::env::var("HLEDGER_FMT_BENCH_FILES") {
        let allowed: HashSet<String> = filter
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(|name| name.to_owned())
            .collect();
        if !allowed.is_empty() {
            corpus_files.retain(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| allowed.contains(name))
                    .unwrap_or(false)
            });
        }
    }

    corpus_files
}

// Registramos benchmarks
criterion_group!(benches, benchmark_parser);
criterion_main!(benches);
