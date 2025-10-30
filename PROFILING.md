# Profiling Guide

This document describes how to profile `hledger-fmt` to identify performance
bottlenecks and code size issues.

## Quick Start

The easiest way to profile is using the included scripts:

```sh
# Profile the combined parse+format operation
./scripts/profile.sh combined

# Profile just parsing or formatting
./scripts/profile.sh parse
./scripts/profile.sh format

# Analyze binary size and code bloat
./scripts/bloat-analysis.sh
```

This generates `flamegraph-*.svg` files showing where CPU time is spent.

## Benchmarking

Run benchmarks to measure performance:

```sh
cargo bench --features bench
```

This measures parse, format, and combined operations and generates reports in
`target/criterion/`.

## Code Size Analysis

### Using cargo-bloat

Analyze binary size to identify large functions and code bloat:

```sh
# Install cargo-bloat
cargo install cargo-bloat

# Run the bloat analysis script
./scripts/bloat-analysis.sh

# Or install automatically
./scripts/bloat-analysis.sh --install

# Generate JSON output for programmatic analysis
./scripts/bloat-analysis.sh --json
```

The script will show:

- **Top functions by size**: Identifies the largest functions in the binary
- **Crate-level breakdown**: Shows which dependencies contribute most to binary
  size
- **Optimization opportunities**: Suggests ways to reduce binary size

### Interpreting bloat results

Look for:

- **Generic instantiations**: Same function compiled multiple times for
  different types
- **Large formatting code**: String formatting can be surprisingly large
- **Dependency bloat**: Unused features from dependencies
- **Debug code**: Ensure debug symbols are stripped in release builds

### Size optimization strategies

1. **Mark large cold functions with `#[cold]` or `#[inline(never)]`**
2. **Use dynamic dispatch (`dyn Trait`) for rarely-used generics**
3. **Enable LTO**: `lto = true` in `Cargo.toml`
4. **Reduce codegen-units**: `codegen-units = 1` for better optimization
5. **Use `opt-level = "z"`**: Optimize for size instead of speed
6. **Strip symbols**: `strip = true` in release profile
7. **Minimize dependencies**: Remove unused features with `default-features =
   false`

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

## Performance Optimization Guidelines

### General Principles

1. **Measure First**: Always benchmark before and after changes
2. **Focus on Hot Paths**: Optimize the most frequently called functions
3. **Profile in Release Mode**: Use `--release` for accurate measurements
4. **Use Real Data**: Test with representative journal files

### Optimization Techniques

#### 1. Inlining

Mark hot functions with `#[inline]` or `#[inline(always)]`:

```rust
#[inline(always)]
pub fn hot_function() {
    // Frequently called code
}
```

#### 2. Reduce Allocations

- Pre-allocate vectors with appropriate capacity
- Reuse buffers when possible
- Use `&[u8]` slices instead of copying data

#### 3. Use SIMD-Friendly Patterns

```rust
// Good: Iterator-based, SIMD-friendly
buf.iter().filter(|&&b| b & 0b1100_0000 != 0b1000_0000).count()

// Avoid: Manual loops with complex branching
let mut count = 0;
for &b in buf {
    if b & 0b1100_0000 != 0b1000_0000 {
        count += 1;
    }
}
count
```

#### 4. Strategic Unsafe Code

Use `unsafe` judiciously for performance-critical paths:

```rust
// When bounds are already checked
if after.len() == 3 {
    unsafe {
        after.get_unchecked(0).is_ascii_digit()
    }
}
```

#### 5. Memory Layout

- Use `#[repr(u8)]` for C-like enums
- Pack structs to reduce memory footprint
- Use `u16` instead of `usize` for bounded values

### Clippy Lints for Performance

Enable performance lints in your code:

```rust
#![warn(clippy::perf)]
#![warn(clippy::missing_inline_in_public_items)]
#![warn(clippy::large_types_passed_by_value)]
#![warn(clippy::inefficient_to_string)]
```

### Known Optimization Opportunities

#### Parser

1. **Value Parsing**: `split_value_in_before_decimals_after_decimals`
   - Uses `memchr` for fast searching
   - Optimized with iterator methods
   - Strategic unsafe for bounds checking

2. **UTF-8 Character Counting**: `utf8_chars_count`
   - Counts non-continuation bytes
   - SIMD-friendly implementation
   - Already well-optimized

3. **Line Parsing**
   - Uses `memchr` for newline detection
   - Minimal allocations
   - Pre-allocated vectors

#### Formatter

1. **Space Generation**: `spaces::extend`
   - Constant arrays up to 256 bytes
   - `ptr::write_bytes` for large counts
   - Fast paths for 0, 1, and small counts

2. **Buffer Management**
   - Pre-allocate with estimated length
   - Reuse buffers across entries
   - Minimize reallocations

3. **Entry Formatting**: `extend_entry`
   - Pre-calculated maximum lengths
   - Cached character counts
   - Minimal string operations

### Performance Testing

Run the full benchmark suite:

```sh
# Quick benchmark
cargo bench --features bench

# With profiling
./scripts/profile.sh combined

# Check for regressions (replace 'my-baseline' with your baseline name)
cargo bench --features bench -- --baseline my-baseline
```

### Performance History

Track performance over time using Criterion's baseline feature:

```sh
# Save current performance as baseline (choose a descriptive name)
cargo bench --features bench -- --save-baseline my-baseline

# Compare against baseline
cargo bench --features bench -- --baseline my-baseline
```

## Tips

- Always profile in release mode
- Use representative workloads (real journal files)
- Focus on hot paths (most frequent functions)
- Measure before and after each optimization
- See [Rust Performance Book](https://nnethercote.github.io/perf-book/) for
  more details
- Consider compile-time optimizations in `Cargo.toml`:
  - `lto = true` - Enable Link-Time Optimization
  - `codegen-units = 1` - Better optimization (slower compile)
  - `opt-level = 3` or `opt-level = "z"` - Optimization level
  - `strip = true` - Remove debug symbols from binary
- Use `cargo-bloat` to identify code size issues
