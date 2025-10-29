# Profiling Guide

This document describes how to profile `hledger-fmt` to identify performance
bottlenecks.

## Quick Start

The easiest way to profile is using the included script:

```sh
# Profile the combined parse+format operation
./scripts/profile.sh combined

# Profile just parsing or formatting
./scripts/profile.sh parse
./scripts/profile.sh format
```

This generates `flamegraph-*.svg` files showing where CPU time is spent.

## Benchmarking

Run benchmarks to measure performance:

```sh
cargo bench --features bench
```

This measures parse, format, and combined operations and generates reports in
`target/criterion/`.

## Manual Profiling

### Using Flamegraph

Install flamegraph:

```sh
cargo install flamegraph
```

Generate flamegraphs:

```sh
cargo flamegraph --bench parse --features bench -- --bench
```

### Using Perf (Linux)

Record performance data:

```sh
perf record --call-graph dwarf ./target/release/hledger-fmt file.journal
perf report
```

## Analyzing Results

Look for functions appearing frequently in profiles:

- **Parser hot paths**: `parse_content`, `maybe_start_with_directive`,
  `parse_transaction_entry`
- **Formatter hot paths**: `format_nodes`, `extend_entry`, `spaces::extend`
- **Utilities**: `utf8_chars_count`,
  `split_value_in_before_decimals_after_decimals`

## Tips

- Always profile in release mode
- Use representative workloads (real journal files)
- Focus on hot paths (most frequent functions)
- Measure before and after each optimization
- See [Rust Performance Book](https://nnethercote.github.io/perf-book/) for
  more details
