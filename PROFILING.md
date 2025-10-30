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

# Check for regressions
cargo bench --features bench -- --baseline main
```

### Performance History

Track performance over time using Criterion's baseline feature:

```sh
# Save current performance as baseline
cargo bench --features bench -- --save-baseline main

# Compare against baseline
cargo bench --features bench -- --baseline main
```

## Tips

- Always profile in release mode
- Use representative workloads (real journal files)
- Focus on hot paths (most frequent functions)
- Measure before and after each optimization
- See [Rust Performance Book](https://nnethercote.github.io/perf-book/) for
  more details
- Consider compile-time optimizations in `Cargo.toml`
- Use `cargo-bloat` to identify code size issues
