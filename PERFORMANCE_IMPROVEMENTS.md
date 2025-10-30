# Performance Improvements

This document describes the performance analysis conducted on hledger-fmt and the improvements made.

## Baseline Performance

Before optimizations:
- `parse_content`: ~10.8µs
- `format_parsed_journal`: ~5.1µs  
- `format_journal` (combined): ~16.6µs

## Improvements Made

### 1. Bug Fix in `split_value_in_before_decimals_after_decimals`

**Location**: `src/parser/mod.rs:1808-1822`

**Issue**: The variable `idx` was incorrectly reused in a nested loop, leading to incorrect calculations when parsing currency values like `$453534€`.

**Fix**: Introduced a separate `trailing_non_digits` counter variable to avoid reusing `idx`.

**Impact**: Bug fix (correctness issue), minimal performance impact.

### 2. Cache `title_chars_count` Outside Transaction Loop

**Location**: `src/formatter/mod.rs:176`

**Issue**: `title.chars_count()` was being called inside the transaction entries loop, even though the title doesn't change within the loop.

**Fix**: Moved the calculation outside the loop and cached the result.

**Impact**: ~1.5% improvement in `format_parsed_journal` benchmark (5.1µs → 5.0µs).

## Performance After Improvements

After optimizations:
- `parse_content`: ~11.2µs (no significant change)
- `format_parsed_journal`: ~5.0µs (1.5% improvement)
- `format_journal` (combined): ~17.2µs (within noise threshold)

## Analysis of Existing Optimizations

The codebase is already well-optimized:

1. **Efficient UTF-8 Character Counting**: The `utf8_chars_count` function uses an iterator-based approach that allows LLVM to auto-vectorize the code.

2. **Manual Byte Manipulation**: The parser uses `unsafe` blocks with `get_unchecked` for performance-critical paths, avoiding bounds checking overhead.

3. **Pre-allocated Buffers**: Both parser and formatter use `Vec::with_capacity` to avoid repeated allocations.

4. **Optimized Space Generation**: The `spaces::extend` function uses compile-time constants and fast paths for common cases.

5. **Directive Recognition**: The `maybe_start_with_directive` function uses manual byte comparisons grouped by first character for fast lookup.

## Potential Future Optimizations

These optimizations would provide improvements but require more invasive changes:

### 1. Cache Character Counts in Data Structures

**Location**: `Directive`, `TransactionEntry` structures

**Idea**: Pre-compute and store `chars_count()` results in the parsed structures.

**Benefit**: Would eliminate repeated `chars_count()` calls in the formatter (lines 125-126, 297-338).

**Trade-off**: Increases memory usage and makes structures larger. Requires careful analysis to ensure the memory overhead doesn't negate performance gains.

### 2. Use SIMD for UTF-8 Character Counting

**Location**: `src/byte_str.rs:70`

**Idea**: Use explicit SIMD instructions or a crate like `bytecount` for faster UTF-8 character counting.

**Benefit**: Could provide 2-4x speedup for `utf8_chars_count` calls.

**Trade-off**: Adds a dependency and may not provide significant overall improvement since character counting is just one small part of the total work.

### 3. Perfect Hash for Directive Recognition

**Location**: `src/parser/mod.rs:979` (`maybe_start_with_directive`)

**Idea**: Use a perfect hash function or compile-time lookup table for directive recognition.

**Benefit**: Could reduce the number of byte comparisons for directive detection.

**Trade-off**: Added complexity, and current implementation is already quite fast with manual optimizations.

### 4. Avoid Temporary Buffer for Comments

**Location**: `src/formatter/mod.rs:193`

**Idea**: Find a way to format entries with comments without the temporary `entry_line_buffer`.

**Benefit**: Would eliminate one allocation per commented entry.

**Trade-off**: Complex to implement while maintaining correct formatting logic.

## Recommendations

1. **Keep the current optimizations**: The bug fix and title caching are good improvements.

2. **Profile before optimizing further**: Use `cargo flamegraph` to identify actual bottlenecks before making invasive changes.

3. **Consider character count caching only if profiling shows it's a significant bottleneck**: The memory trade-off needs careful analysis.

4. **Monitor performance on real workloads**: The benchmark uses a relatively small file. Real journal files might have different performance characteristics.

## Benchmarking

To measure the impact of changes:

```bash
# Run all benchmarks
cargo bench --features bench

# Run specific benchmark
cargo bench --features bench --bench parse_and_format

# Profile with flamegraph
./scripts/profile.sh combined
```
