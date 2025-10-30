# Real-World Hledger Journal Corpus Files

This document describes the real-world Hledger journal examples added to the
`fuzz/corpus/` directory for comprehensive testing.

## Files Added

### 1. basic.journal

**Source**: Existing basic test file
**Features**:

- Simple transaction with two postings
- Basic date and account formatting

### 2. cheatsheet.hledger

**Source**: Existing comprehensive example file
**Features**:

- Complete syntax reference
- All directive types
- Various transaction patterns

### 3. multicurrency.journal

**Source**: Official hledger repository examples
**Features**:

- Multi-currency transactions (HRK, EUR)
- Currency exchange operations using @ syntax
- Balance assertions with ==\*
- Account declarations with type tags
- Multiline comments with example output

### 4. multi-bank-currencies.journal

**Source**: Real user's multi-bank financial tracking setup
**Features**:

- Multiple currency tracking (ILS, USD, EUR)
- Bank-specific currency commodities (USD:BOI, USD:BOJ)
- Price directives (P) for exchange rates
- Commodity format declarations with subdirectives
- Virtual postings with parenthesized accounts
- Slash date format (YYYY/MM/DD)
- Complex account hierarchies

### 5. uk-finances.journal

**Source**: Real UK personal finance journal
**Features**:

- Balance assertions with multiple forms (=)
- Transaction codes in parentheses (BGC, DEB, BP, FOREIGN CCY)
- Foreign currency conversions with @@ syntax
- Multiple account types (assets, liabilities, income, expenses)
- Pound sterling (£) and dollar ($) currencies
- Interest calculations with descriptive notes

### 6. stock-trading.journal

**Source**: Investment tracking with hledger-lots integration
**Features**:

- Custom directives (#+hledger-lots, #+args)
- Stock lot tracking with @ price syntax
- Capital gains calculations
- Commodity declarations with comments/tags
- Price history (P directives) with high-precision decimals
- Inline semicolon comments with metadata (cost_method, buy_date, etc.)
- Complex commodity symbols with quotes ("AAPL", "GOOG", "PETR4.SA")

### 7. timelog.journal

**Source**: Time tracking integration (taskwarrior)
**Features**:

- Include directives
- Default commodity directive (D)
- Integration with external time tracking systems

## Test Coverage

Each corpus file is used to test:

1. **Parsing**: Files can be successfully parsed without errors
2. **Formatting**: Files can be formatted with consistent spacing and alignment
3. **Round-trip**: Parse → Format → Parse produces identical AST
4. **Fuzzing**: Files serve as seed corpus for fuzz testing

## Unit Tests Added

New unit tests in `src/formatter/tests.rs`:

- `corpus_multicurrency_example`: Multi-currency transaction formatting
- `corpus_balance_assertions`: Balance assertions with multiple currencies
- `corpus_transaction_codes`: Transactions with parenthesized codes
- `corpus_stock_trading`: Stock trading with lot prices
- `corpus_price_directives`: Market price directives
- `corpus_commodity_with_format`: Commodity declarations with format subdirectives
- `corpus_virtual_postings`: Virtual postings (parenthesized accounts)
- `corpus_include_directive`: Include directives
- `corpus_custom_directives`: Custom directives (hledger-lots style)

## Verification

All corpus files have been verified to:

1. Parse successfully ✓
2. Format without errors ✓
3. Work with all three fuzz targets (parse, format, roundtrip) ✓
4. Pass unit test assertions ✓

Total tests: 86 (9 new corpus tests added)

## Usage for Fuzzing

The corpus files in this directory serve as seed inputs for cargo-fuzz:

```bash
# Run fuzz tests with the corpus
cargo +nightly fuzz run fuzz_parse
cargo +nightly fuzz run fuzz_format
cargo +nightly fuzz run fuzz_roundtrip
```

When fuzzing runs, it copies files from `fuzz/corpus/` into target-specific
directories like `fuzz/corpus/fuzz_parse/`, which are gitignored and can grow
during fuzzing sessions.
