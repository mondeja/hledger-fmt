# hledger-fmt

[![Crates.io](https://img.shields.io/crates/v/hledger-fmt?logo=rust)](https://crates.io/crates/hledger-fmt)
[![License](https://img.shields.io/crates/l/hledger-fmt)][license-badge-link]
[![Tests](https://img.shields.io/github/actions/workflow/status/mondeja/hledger-fmt/ci.yml?label=tests&logo=github)][tests-badge-link]

<!-- markdown-link-check-disable -->

[license-badge-link]: https://github.com/mondeja/hledger-fmt/blob/master/LICENSE
[tests-badge-link]: https://github.com/mondeja/hledger-fmt/actions

<!-- markdown-link-check-enable -->

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

### VSCode

With hledger-fmt in your PATH, use the [VSCode Custom Local Formatters]
extension. Just install it and add the next configuration to your
_settings.json_:

```json
{
  "customLocalFormatters.formatters": [
    {
      "command": "hledger-fmt - --no-diff",
      "languages": ["hledger"]
    }
  ]
}
```

To format on save:

```json5
{
  "editor.formatOnSave": true,
}
```

Just ensure that `hledger-fmt` is in your PATH.

## Usage

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

## Features

- **`color`** (enabled by default): Build with terminal color support.

[cargo-binstall]: https://github.com/cargo-bins/cargo-binstall
[hledger]: https://hledger.org
[cargo]: https://doc.rust-lang.org/cargo/
[releases page]: https://github.com/mondeja/hledger-fmt/releases
[pre-commit]: https://pre-commit.com
[VSCode Custom Local Formatters]: https://marketplace.visualstudio.com/items?itemName=jkillian.custom-local-formatters
