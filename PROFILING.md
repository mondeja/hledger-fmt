# Profiling Guide

This document describes how to profile `hledger-fmt` to identify performance bottlenecks.

## Prerequisites

Install profiling tools:

```sh
# For Linux perf profiling
cargo install flamegraph

# For CPU profiling with valgrind
sudo apt-get install valgrind kcachegrind  # On Debian/Ubuntu
```

## Methods

### 1. Criterion Benchmarking (Built-in)

Run benchmarks to measure performance:

```sh
cargo bench --features bench
```

This will:

- Measure parse, format, and combined operations
- Generate HTML reports in `target/criterion/`
- Compare against previous runs

View reports:

```sh
open target/criterion/report/index.html
```

### 2. Flamegraph Profiling (CPU Time)

Generate flamegraphs to visualize where CPU time is spent:

```sh
# Profile the parsing benchmark
cargo flamegraph --bench parse --features bench -- --bench

# Profile the formatting benchmark
cargo flamegraph --bench format --features bench -- --bench

# Profile the combined benchmark
cargo flamegraph --bench parse_and_format --features bench -- --bench
```

This generates `flamegraph.svg` showing call stacks and time spent.

### 3. Perf Profiling (Linux)

For detailed CPU profiling on Linux:

```sh
# Build with profiling symbols
cargo build --release --features bench

# Record performance data
perf record --call-graph dwarf ./target/release/hledger-fmt-bench parse

# View the report
perf report

# Generate flamegraph from perf data
perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg
```

### 4. Valgrind/Callgrind (Cache and Instruction Analysis)

Profile cache usage and instruction counts:

```sh
# Build release binary
cargo build --release

# Run with callgrind
valgrind --tool=callgrind ./target/release/hledger-fmt test-file.journal

# Visualize results
kcachegrind callgrind.out.*
```

### 5. Memory Profiling (Heap Usage)

Profile memory allocations:

```sh
# Using valgrind massif
valgrind --tool=massif ./target/release/hledger-fmt test-file.journal

# Visualize memory usage
ms_print massif.out.*

# Or use heaptrack (if installed)
heaptrack ./target/release/hledger-fmt test-file.journal
heaptrack_gui heaptrack.hledger-fmt.*.gz
```

### 6. Criterion with Profiling

Enable criterion profiling mode:

```sh
# Profile with criterion's built-in profiler
cargo bench --features bench -- --profile-time=5
```

## Analyzing Results

### Hot Path Identification

Look for functions that appear frequently in flamegraphs or perf reports:

- **Parser**: `parse_content`, `maybe_start_with_directive`, `parse_transaction_entry`
- **Formatter**: `format_nodes`, `extend_entry`, `spaces::extend`
- **Utilities**: `utf8_chars_count`, `split_value_in_before_decimals_after_decimals`

### Memory Bottlenecks

Check for:

- Frequent allocations in tight loops
- Large vector growths (indicates poor capacity estimation)
- Unnecessary cloning or copying

### Cache Performance

Look for:

- Cache misses in hot loops
- Poor data locality
- Branch mispredictions

## Optimization Workflow

1. **Baseline**: Run benchmarks before changes

   ```sh
   cargo bench --features bench > baseline.txt
   ```

2. **Profile**: Identify bottlenecks using flamegraphs

   ```sh
   cargo flamegraph --bench parse_and_format --features bench -- --bench
   ```

3. **Optimize**: Make targeted improvements to hot paths

4. **Verify**: Re-run benchmarks and compare

   ```sh
   cargo bench --features bench > optimized.txt
   ```

5. **Iterate**: Repeat for next bottleneck

## Example Profiling Session

```sh
# 1. Initial benchmark
cargo bench --features bench

# 2. Generate flamegraph
cargo flamegraph --bench parse_and_format --features bench -- --bench

# 3. Identify that `chars_count()` is called frequently
#    Open flamegraph.svg in browser

# 4. Make optimization (e.g., cache the result)

# 5. Re-benchmark to verify improvement
cargo bench --features bench

# 6. Check the difference
#    Look for "Performance has improved" in output
```

## Profiling Features

The project includes a `bench` feature flag specifically for benchmarking:

```toml
[features]
bench = []
```

This exposes internal functions for benchmarking without making them public API.

## Tips

- Always profile in release mode (`--release` or benchmarks)
- Use representative workloads (real journal files)
- Focus on hot paths (functions appearing most in profiles)
- Measure before and after each optimization
- Small files may not show bottlenecks - use larger files
- Profile on the target deployment platform when possible

## Additional Resources

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)
- [Linux perf Tutorial](https://perf.wiki.kernel.org/index.php/Tutorial)
- [Flamegraph Guide](https://www.brendangregg.com/flamegraphs.html)
