# Additional Performance Optimization Opportunities

Based on profiling and code analysis, here are additional opportunities for
performance gains in hledger-fmt.

## Current Performance

**Baseline (after optimizations):**

- Parsing: 10.16 µs
- Formatting: 4.37 µs
- Combined: 15.07 µs

**Improvements from original baseline:**

- Parsing: ~12% faster
- Formatting: ~17% faster

## Potential Further Optimizations

### 1. Buffer Pre-allocation Tuning (Estimated: 1-2% gain)

**Current state:** Using estimated capacity based on input size

```rust
// src/parser/mod.rs:190
let estimated_nodes = (bytes.len() / 75).max(16);
let mut journal = Vec::with_capacity(estimated_nodes);
```

**Opportunity:** Profile real-world files to refine the estimation ratio. The
current `/75` may be sub-optimal for typical journal files.

**Implementation:**

- Collect statistics from real journal files
- Adjust ratio based on average nodes per byte
- Test with various file sizes

**Effort:** Low  
**Risk:** Low

### 2. Reduce Repeated Bounds Checking (Estimated: 2-3% gain)

**Current state:** Many `get_unchecked` calls already optimize hot paths

**Opportunity:** Some safe array accesses in loops could benefit from unsafe
optimization:

```rust
// Example in split_value_in_before_decimals_after_decimals
if after.len() == 3 
    && after[0].is_ascii_digit()  // Could use get_unchecked
    && after[1].is_ascii_digit()
    && after[2].is_ascii_digit()
```

**Implementation:** Add unsafe blocks with proper safety comments

**Effort:** Low  
**Risk:** Medium (requires careful safety analysis)

### 3. String Interning for Common Values (Estimated: 3-5% gain)

**Opportunity:** Many journal files reuse the same account names, currencies,
and payees. String interning could reduce allocations.

**Implementation:**

- Add optional interning for account names
- Use a simple hash map for deduplication
- Trade memory for speed

**Effort:** High  
**Risk:** Medium (increases code complexity)

### 4. Specialized Fast Path for ASCII-only Files (Estimated: 5-10% gain)

**Current state:** UTF-8 character counting handles all cases

**Opportunity:** Most journal files are pure ASCII. Detect this early and use
faster byte-based operations.

**Implementation:**

```rust
// Quick ASCII check at file start
let is_ascii = bytes.iter().all(|&b| b < 128);
if is_ascii {
    // Use simpler, faster byte-based operations
    // Skip UTF-8 validation and character counting
}
```

**Effort:** Medium  
**Risk:** Low (can fallback to current implementation)

### 5. Lazy Evaluation for Max Length Calculations (Estimated: 1-2% gain)

**Opportunity:** Maximum length calculations in transactions might be computed
even when not needed (e.g., for transactions without values).

**Implementation:** Compute max lengths only when actually formatting, not
during parsing.

**Effort:** Medium  
**Risk:** Medium (requires restructuring data flow)

### 6. SIMD for Whitespace and Digit Scanning (Estimated: 3-7% gain)

**Opportunity:** Use explicit SIMD instructions for scanning operations like:

- Finding whitespace
- Validating digit sequences
- Scanning for comment markers

**Implementation:**

- Use `std::simd` or platform-specific intrinsics
- Provide scalar fallback for portability
- Test on target architectures

**Effort:** High  
**Risk:** High (complexity, portability concerns)

### 7. Reduce Vec Resizing (Estimated: 1-2% gain)

**Current state:** Some vectors may resize during growth

**Opportunity:**

```rust
// src/formatter/mod.rs:190
let mut entry_line_buffer = Vec::with_capacity(e.name.len() + 32);
```

The `+ 32` heuristic could be improved based on typical entry sizes.

**Implementation:** Profile typical entry sizes and adjust capacity estimates

**Effort:** Low  
**Risk:** Low

### 8. Custom Allocator (Estimated: 5-15% gain)

**Opportunity:** Use a custom allocator optimized for the parsing workload:

- Arena allocator for parse tree nodes
- Bump allocator for temporary buffers

**Implementation:**

- Integrate `bumpalo` or similar
- Requires lifetime management changes

**Effort:** Very High  
**Risk:** High (API changes, complexity)

## Recommended Next Steps

### High Priority (Best ROI)

1. **ASCII fast path** - Good gain potential, low risk
2. **Buffer pre-allocation tuning** - Easy to implement, reliable gains
3. **Reduce bounds checking** - Targeted improvements to hot paths

### Medium Priority

4. **Vec resizing optimization** - Low hanging fruit
5. **Lazy evaluation** - Requires design changes but good potential

### Low Priority (High effort/risk)

6. **String interning** - Complexity vs. gain trade-off
7. **SIMD operations** - Portability concerns
8. **Custom allocator** - Major refactoring required

## Profiling Recommendations

To identify the best next optimization:

1. Generate flamegraphs with `./scripts/profile.sh combined`
2. Identify widest bars (most time spent)
3. Focus on functions called repeatedly in tight loops
4. Measure improvement after each change

## Conclusion

Current optimizations have achieved strong results (~12-17% improvement).
Further gains are possible but with diminishing returns and increasing
complexity. The recommended next steps focus on low-risk, targeted
improvements that maintain code quality while achieving additional
performance gains.
