# Performance Report: Buffer Resizing Optimization (PR #65)

This report analyzes the performance impact of the buffer resizing optimization introduced in PR #65.

## Summary

The optimization adds proactive buffer reservation in the `format_nodes` function to avoid multiple reallocations during formatting. The change reserves an additional 1024 bytes when the buffer is larger than 256 bytes and has less than 1024 bytes of remaining capacity.

## Code Change

```rust
fn format_nodes(nodes: &JournalFile, buffer: &mut Vec<u8>, entry_spacing: usize) {
    let buffer_capacity = buffer.capacity();
    if buffer_capacity > 256 {
        // Avoid to allocate space for small buffers
        if buffer.len() + 1024 > buffer_capacity {
            buffer.reserve(1024);
        }
    }
    // ... rest of function
}
```

## Benchmark Results

Benchmarks were run using `cargo bench --features bench` on both the master branch (baseline) and the reserve-buffers branch (with optimization).

### Format-Only Benchmark (`format_parsed_journal`)

This benchmark measures the performance of formatting already-parsed journal data.

| File | Master (baseline) | Reserve-buffers | Change | Impact |
|------|-------------------|-----------------|--------|--------|
| basic.journal | 155.26 ns | 162.46 ns | **+5.38%** | ⚠️ Regression |
| cheatsheet.hledger | 4.4716 µs | 4.5468 µs | **+1.57%** | ⚠️ Regression |
| multi-bank-currencies.journal | 2.8205 µs | 2.7899 µs | **-0.89%** | ✓ Within noise |
| multicurrency.journal | 838.16 ns | 857.47 ns | **+2.18%** | ⚠️ Regression |
| stock-trading.journal | 2.8149 µs | 2.7527 µs | **-2.27%** | ✓ Improvement |
| timelog.journal | 88.863 ns | 86.512 ns | **-0.84%** | ✓ Within noise |
| uk-finances.journal | 2.5569 µs | 2.5282 µs | **-1.01%** | ✓ Within noise |

**Key Observations:**
- Small files (basic.journal, timelog.journal) show slight regressions, likely due to the overhead of checking buffer capacity
- The optimization shows mixed results across different file sizes
- The stock-trading.journal shows a modest improvement (~2.3%)

### Parse+Format Benchmark (`format_journal`)

This benchmark measures the full end-to-end performance including both parsing and formatting.

| File | Master (baseline) | Reserve-buffers | Change | Impact |
|------|-------------------|-----------------|--------|--------|
| basic.journal | 618.34 ns | 603.98 ns | **-2.46%** | ✓ Improvement |
| cheatsheet.hledger | 16.740 µs | 16.763 µs | **+0.06%** | ✓ No change |
| multi-bank-currencies.journal | 12.232 µs | 12.467 µs | **+1.93%** | ⚠️ Regression |
| multicurrency.journal | 3.6862 µs | 3.7422 µs | **+1.13%** | ✓ Within noise |
| stock-trading.journal | 10.687 µs | 10.854 µs | **+1.48%** | ⚠️ Regression |
| timelog.journal | 547.97 ns | 620.67 ns | **+13.42%** | ⚠️ Significant regression |
| uk-finances.journal | 10.578 µs | 10.732 µs | **+0.88%** | ✓ Within noise |

**Key Observations:**
- The timelog.journal file shows a significant regression (+13.4%) in the combined parse+format benchmark
- basic.journal shows a small improvement (-2.5%)
- Most other files show minor regressions or changes within the noise threshold

## Analysis

### Unexpected Results

The benchmark results show **unexpected behavior** that differs from the intended optimization goal:

1. **Small files regress**: Files like basic.journal and timelog.journal show performance regressions in the format-only benchmark, suggesting the overhead of the capacity check outweighs any benefit for small buffers.

2. **Timelog.journal anomaly**: The 13.4% regression in the parse+format benchmark for timelog.journal is concerning and suggests the optimization may interact poorly with certain file patterns.

3. **Mixed results**: The lack of consistent improvement across files suggests the 1024-byte reservation size and 256-byte threshold may not be optimal for typical hledger journal files.

### Potential Issues

1. **Premature optimization**: For small files that don't need buffer resizing, the capacity check adds unnecessary overhead.

2. **Fixed reservation size**: The 1024-byte reservation may be too large for small files and too small for large files, leading to either wasted allocations or continued reallocations.

3. **Threshold selection**: The 256-byte threshold for enabling the optimization may not align well with typical journal file sizes.

## Recommendations

Based on these benchmark results, I recommend:

1. **Re-evaluate the optimization approach**: Consider whether a different strategy would be more effective, such as:
   - Using the input file size to estimate buffer capacity upfront (in `format_content_with_options`)
   - Making the reservation size proportional to current buffer size
   - Removing the optimization entirely if the benefits don't outweigh the costs

2. **Investigate the timelog.journal regression**: The 13.4% regression in parse+format for timelog.journal needs further investigation to understand the root cause.

3. **Consider profiling**: Use profiling tools (see PROFILING.md) to understand where time is actually being spent and whether buffer allocations are a significant bottleneck.

4. **Test with larger files**: The current corpus files may be too small to show the benefits of this optimization. Testing with larger, more realistic journal files could provide better insights.

## Conclusion

While the optimization was intended to improve performance by reducing buffer reallocations, the benchmark results show **mixed performance impact with several regressions**, particularly for smaller files. The most concerning result is the 13.4% regression for timelog.journal in the combined parse+format benchmark.

Further investigation and potentially a different optimization strategy may be needed to achieve the desired performance improvements without introducing regressions for common use cases.

---

**Benchmark Environment:**
- Rust version: See Cargo.toml
- CPU: GitHub Actions runner
- Benchmark tool: Criterion v0.7.0
- Iterations: 100 samples per benchmark
