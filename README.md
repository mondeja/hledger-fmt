# hledger-fmt

[![Crates.io](https://img.shields.io/crates/v/hledger-fmt?logo=rust)](https://crates.io/crates/hledger-fmt)
[![docs.rs](https://img.shields.io/docsrs/hledger-fmt?logo=docs.rs)](https://docs.rs/hledger-fmt)
[![Tests](https://img.shields.io/github/actions/workflow/status/mondeja/hledger-fmt/ci.yml?label=tests&logo=github)](https://github.com/mondeja/hledger-fmt/actions)
[![License](https://img.shields.io/crates/l/hledger-fmt)](https://github.com/mondeja/hledger-fmt/blob/master/LICENSE)

An opinionated [hledger]'s journal files formatter.

## Installation

[![Crates.io downloads](https://img.shields.io/crates/d/hledger-fmt?label=Crate%20downloads)](https://crates.io/crates/hledger-fmt)
![GitHub downloads](https://img.shields.io/github/downloads/mondeja/hledger-fmt/total?label=GitHub%20downloads)

### Standalone pre-built binaries

Download standalone pre-built binaries from [releases page].

### Cargo binaries

Install from pre-built binaries using [cargo-binstall]:

```sh
cargo binstall hledger-fmt
```

### Build from source

Build from source using [cargo]:

```sh
cargo install hledger-fmt
```

### pre-commit

Use it with [pre-commit] by adding the hook to your _.pre-commit-config.yaml_:

```yaml
repos:
  - repo: https://github.com/mondeja/hledger-fmt
    rev: vX.Y.Z
    hooks:
      # id: hledger-fmt       # Use this id to format files in place
      - id: hledger-fmt-check # Use this id to check files without formatting
```

### VS Code

With hledger-fmt in your PATH, use the [Custom Local Formatters]
extension. Just install it and add the next configuration to your
_settings.json_:

```json
{
  "customLocalFormatters.formatters": [
    {
      "command": "hledger-fmt - --no-diff --exit-zero-on-changes",
      "languages": ["hledger"]
    }
  ]
}
```

To format on save:

```json
{
  "editor.formatOnSave": true
}
```

### Zed

With hledger-fmt in your PATH, add the next configuration to your
_settings.json_:

```json
{
  "languages": {
    "Ledger": {
      "formatter": {
        "external": {
          "command": "hledger-fmt",
          "arguments": ["--no-diff", "--exit-zero-on-changes"]
        }
      }
    }
  }
}
```

To format on save:

```json
{
  "format_on_save": "on"
}
```

### Library

You can use `hledger-fmt` as a standalone library in your Rust projects. Add the
following to your _Cargo.toml_:

```toml
[dependencies]
hledger-fmt = { version = "0.3", default-features = false, features = ["std"] }
```

## Usage

### CLI

When you don't pass files to format, it reads all the files with
the extensions `.journal`, `.hledger` and `.j` in the current directory
and its subdirectories.

```sh
hledger-fmt [OPTIONS] [FILES]...
```

To fix them in place, use the `--fix` option:

> [!WARNING]\
> This is a potentially destructive operation. Make sure to make a backup
> of your files before running this command for the first time.

```sh
hledger-fmt --fix [FILES]...
```

See `hledger-fmt --help` for more information.

### Library

```rust
use hledger_fmt::{format_journal, format_journal_bytes};

fn main() {
    let journal = r#"
2024-01-01 * "Sample transaction"
    Assets:Cash  $100
    Expenses:Food  $100
"#;
    match format_journal(journal) {
        Ok(formatted) => println!("{formatted}"),
        Err(e) => eprintln!("Error formatting journal: {e}"),
    }

    match format_journal_bytes(journal.as_bytes()) {
        Ok(formatted_bytes) => println!("{}", String::from_utf8_lossy(&formatted_bytes)),
        Err(e) => eprintln!("Error formatting journal: {e}"),
    }
}
```

### Features

- **`color`** (enabled by default): Build with terminal color support.
- **`auto-color`** (enabled by default): Automatically detects if your terminal
  supports colors.
- **`diff`** (enabled by default): Show a diff of the changes made to the files.
  Adds the `--no-diff` option to disable it.
- **`cli`** (enabled by default): Build the CLI binary tool.
- **`std`**: Enable building with the standard library. Disable it to build with
  `no_std` + `alloc` support.
- **`env`**: Enable reading configuration from environment variables at run time.
- **`tracing`**: Enable tracing support. Adds the CLI argument `--trace-file` to
  write tracing logs to a file.

### Configuration

By default, hledger-fmt comes with sensible opinionated defaults. You can customize
them at compile or run time by using the next environment variables:

<!-- markdownlint-disable line-length -->

| Variable name               | Default value | Description                                     |
| --------------------------- | ------------- | ----------------------------------------------- |
| `HLEDGER_FMT_ENTRY_SPACING` | `2`           | Minimum number of spaces between entry columns. |

<!-- markdownlint-enable line-length -->

By default, environment variables are read at compile time only. Enabling the `env`
feature adds support for reading these variables at runtime.

[cargo-binstall]: https://github.com/cargo-bins/cargo-binstall
[hledger]: https://hledger.org
[cargo]: https://doc.rust-lang.org/cargo/
[releases page]: https://github.com/mondeja/hledger-fmt/releases
[pre-commit]: https://pre-commit.com
[Custom Local Formatters]: https://marketplace.visualstudio.com/items?itemName=jkillian.custom-local-formatters
