# Contributing

## Running tests

```sh
cargo build && cargo test
```

## Running benchmarks

```sh
cargo bench --release --features bench
```

## Fuzzy testing

This project uses `cargo-fuzz` for fuzzy testing. Fuzzing requires nightly Rust.

### Installing cargo-fuzz

```sh
rustup install nightly
cargo install cargo-fuzz
```

### Running fuzz tests

Run a specific fuzz target:

```sh
# Fuzz the parser
cargo +nightly fuzz run fuzz_parse

# Fuzz the formatter
cargo +nightly fuzz run fuzz_format

# Fuzz the roundtrip (parse -> format -> parse)
cargo +nightly fuzz run fuzz_roundtrip
```

Run with a time limit:

```sh
cargo +nightly fuzz run fuzz_parse -- -max_total_time=60
```

### Reproducing crashes

If a fuzz target finds a crash, reproduce it with:

```sh
cargo +nightly fuzz run fuzz_parse fuzz/artifacts/fuzz_parse/crash-<hash>
```

Or run it directly with the CLI:

```sh
cargo run -- fuzz/artifacts/fuzz_parse/crash-<hash>
```

### Coverage

Generate coverage information:

```sh
cargo +nightly fuzz coverage fuzz_parse
```

View coverage:

```sh
cargo +nightly fuzz coverage fuzz_parse --html
```

## Debugging with tracing

```sh
echo ' ; comment' | cargo run --features tracing -- -
```

```sh
echo ' ; comment' | cargo run --features tracing -- - --trace-file trace.log
```
