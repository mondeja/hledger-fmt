# CHANGELOG

## 2025-05-20 - [0.2.8]

### Bug fixes

- Fix core dump parsing transaction entry with only comment.

## 2025-05-19 - [0.2.7]

### Bug fixes

- Fix balance assertion with prices stripped from result.

## 2025-04-14 - [0.2.6]

### Enhancements

- Remove `walkdir` dependency.
- Add public API to use `hledger-fmt` as a library (see
  [docs.rs documentation](https://docs.rs/hledger-fmt)
  for more information).

## 2025-02-09 - [0.2.5]

### Enhancements

- Speed up parsing and formatting.
- Distribute Linux ARM64 binaries.

## 2025-01-20 - [0.2.4]

### Changes

- Add MSRV.
- Add `manpages` feature to build MAN pages (`clap_mangen` dependency
  not used by default).

## 2025-01-14 - [0.2.3]

### Changes

- Exit always with code 0 when `--no-diff` option is used.

## 2025-01-13 - [0.2.2]

### Bug fixes

- Fix substraction with overflow formatting some assertions.

## 2024-11-13 - [0.2.1]

### Bug fixes

- Set exitcode 2 when CLI detects possible changes for files.

## 2024-10-18 - [0.2.0]

### Breaking changes

- `--no-color` CLI flag has been removed. Use `NO_COLOR` environment variable
  instead.
- `-f` short CLI flag has been removed. Use `--fix` instead.

### New features

- Add `auto-color` compilation feature enabled by default to automatically
  detect if your terminal supports colors.

### Enhancements

- Reduce distributed binary sizes ~80%.
- Drop `clap-derive` dependency.
- Drop `colored` dependency.
- Generate man page at compile time.

## 2024-09-29 - [0.1.4]

### New features

- Allow to read from STDIN with `-` argument.
- Allow to print formatted content to STDOUT with `--no-diff` option.

### Changes

- Exit with code 0 when a file is formatted.

## 2024-09-28 - [0.1.3]

### Enhancements

- Provide standalone binaries for Windows X86_64 and Linux X86_64,
  MacOS X86_64 and MacOS ARM64.

## 2024-09-27 - [0.1.2]

### Bug fixes

- Exit code 1 when files formatted.

## 2024-09-27 - [0.1.1]

### Bug fixes

- Fixed bug reading files from CLI.

## 2024-09-27 - 0.1.0

First beta release

[0.2.8]: https://github.com/mondeja/hledger-fmt/compare/v0.2.7...v0.2.8
[0.2.7]: https://github.com/mondeja/hledger-fmt/compare/v0.2.6...v0.2.7
[0.2.6]: https://github.com/mondeja/hledger-fmt/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/mondeja/hledger-fmt/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/mondeja/hledger-fmt/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/mondeja/hledger-fmt/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/mondeja/hledger-fmt/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/mondeja/hledger-fmt/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/mondeja/hledger-fmt/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/mondeja/hledger-fmt/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/mondeja/hledger-fmt/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/mondeja/hledger-fmt/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/mondeja/hledger-fmt/compare/v0.1.0...v0.1.1
