<!-- markdownlint-disable MD013 MD040 -->

# Architecture Overview

`hledger-fmt` follows a deliberately slim architecture: a CLI thin-layer feeds a
library that performs a single-pass parse into a compact AST, and a formatter
renders that AST back to aligned text. The implementation prefers borrowed
byte-slices (`&[u8]`) to avoid copying, and caches character counts so the
formatter can align output without walking strings repeatedly.

```
CLI (src/main.rs)
    └─ Library API (src/lib.rs)
         ├─ Parser (src/parser/mod.rs)
         │     └─ Borrowed AST (JournalFile<'a>)
         └─ Formatter (src/formatter/mod.rs)
               └─ Rendered bytes / String
```

## Object Graph

### JournalFile Tree

`JournalFile<'a> = Vec<JournalCstNode<'a>>`

Each `JournalCstNode` stores borrowed slices (`ByteStr<'a>`) into the original
input. The tree mirrors the structure visible to the formatter:

| Node                                               | Description                                     | Children               |
| -------------------------------------------------- | ----------------------------------------------- | ---------------------- |
| `EmptyLine`                                        | blank line (including whitespace-only lines)    | —                      |
| `MultilineComment { content }`                     | `comment … end comment` block                   | —                      |
| `SingleLineComment`                                | top-level inline comment (`# foo`)              | —                      |
| `DirectivesGroup { nodes, max_name_content_len }`  | consecutive directives separated by empty lines | `DirectiveNode` list   |
| `Transaction { title, title_comment, entries, … }` | full postings block                             | `TransactionNode` list |

Supporting nodes:

- `DirectiveNode`
  - `Directive` – name/content/comment + cached UTF-8 counts
  - `Subdirective` – e.g. `subdirective foo`
  - `SingleLineComment`
- `TransactionNode`
  - `TransactionEntry` – postings with cached widths and optional comment
  - `SingleLineComment`

All counts are stored as `u16` because the formatter only needs widths up to a
few thousand characters.

### Temporary Parser State

`ParserTempData` is a mutable scratchpad holding:

- Accumulators for pending directive groups or transactions
- Byte indices for multi-line comments
- Cached maxima for alignment (entry name length, value segments)
- A reusable `EntryValueParser`

The parser never allocates new strings; it only stores ranges into the original
byte buffer.

## Parser Pipeline

Entry point: `parse_content(bytes: &[u8]) -> Result<JournalFile<'a>, SyntaxError>`.

1. Walk the input line-by-line using `memchr` to find `\n`.
2. For each line, classify by first byte:
   - blank / whitespace-only ⇒ `EmptyLine`
   - `comment` / `end comment` ⇒ multi-line comment state machine
   - `#` / `;` ⇒ `SingleLineComment`
   - directive keyword ⇒ `parse_directive`
   - otherwise ⇒ transaction title or entry
3. `parse_directive` uses the lightweight `maybe_start_with_directive`
   (tab/space tolerant) and records name/content/comment widths.
4. Transactions accumulate into `ParserTempData::transaction_entries` until a
   blank line or EOF flushes the transaction.
5. `EntryValueParser` (state machine) slices a posting’s value into up to three
   parts (`amount`, `=`, `@`) without allocations.

Error handling is incremental: the parser reports the first syntax issue with
line/column metadata (`errors::SyntaxError`).

## Formatter Pipeline

`format_content_with_options(nodes, opts)` traverses the AST:

1. Pre-allocate a buffer using the caller’s estimated length.
2. Match each `JournalCstNode`:
   - Comments render directly with stored indent/prefix.
   - Directive groups compute padding using cached char counts and
     `max_name_content_len`.
   - Transactions align postings using the maxima cached in the parser.
3. `spaces::extend` provides fast padding via pre-built space slabs.

Because the parser provided character counts (UTF-8 aware), the formatter never
re-scans slices to compute display widths.

## Supporting Components

- **CLI (`src/main.rs`)** – wraps the library, handles input discovery,
  diffing, and exit codes.
- **Library facade (`src/lib.rs`)** – exposes `format_journal`,
  `format_journal_bytes`, `format_content_with_options`, etc., for both CLI and
  embedding.
- **Tests**
  - Unit tests live alongside parser/formatter logic.
  - Integration tests in `tests/` exercise CLI behaviors.

## Implementation Notes

- The entire AST borrows from the original buffer, so the input must outlive the
  `JournalFile`. This keeps parsing allocation-free aside from vector growth.
- Unsafe code (`get_unchecked`) is confined to hot loops where bounds are
  statically guaranteed (e.g. directive keyword matching).
- Character counts rely on `ByteStr::chars_count` (SIMD-friendly) to remain
  locale-agnostic.
- Formatting alignment assumes monospace output; cached counts handle multi-byte
  UTF-8 but not East Asian width—sufficient for the current CLI use case.
