use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn collect_corpus_files() -> Vec<PathBuf> {
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
            if corpus_files.len() == 0 {
                eprintln!("None of the specified files in HLEDGER_FMT_BENCH_FILES were found.");
                std::process::exit(1);
            }
        }
    }

    corpus_files
}
