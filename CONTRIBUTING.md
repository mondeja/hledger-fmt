# Contributing

## Running tests

```sh
cargo build && cargo test
```

## Running benchmarks

```sh
cargo bench --features bench
```

## Code Size Analysis

Analyze binary size and identify code bloat:

```sh
# Install cargo-bloat (one-time)
cargo install cargo-bloat

# Run bloat analysis
./scripts/bloat-analysis.sh

# Or install and run in one command
./scripts/bloat-analysis.sh --install
```

This helps identify:

- Large functions that could be optimized
- Generic code instantiations
- Dependency bloat
- Size optimization opportunities

## Profiling

See [PROFILING.md](PROFILING.md) for detailed profiling instructions.

Quick start:

```sh
# Profile the combined parse+format operation
./scripts/profile.sh combined

# Profile just parsing
./scripts/profile.sh parse

# Profile just formatting
./scripts/profile.sh format
```

This generates flamegraphs showing where CPU time is spent.

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
