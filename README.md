# hledger-fmt

[![Crates.io](https://img.shields.io/crates/v/hledger-fmt?logo=rust)](https://crates.io/crates/hledger-fmt)
[![License](https://img.shields.io/crates/l/hledger-fmt)][license-badge-link]
[![Tests](https://img.shields.io/github/actions/workflow/status/mondeja/hledger-fmt/ci.yml?label=tests&logo=github)][tests-badge-link]
[![Crates.io downloads](https://img.shields.io/crates/d/hledger-fmt)](https://crates.io/crates/hledger-fmt)

<!-- markdown-link-check-disable -->

[license-badge-link]: https://github.com/mondeja/hledger-fmt/blob/master/LICENSE
[tests-badge-link]: https://github.com/mondeja/hledger-fmt/actions

<!-- markdown-link-check-enable -->

An opinionated [hledger]'s journal files formatter.

## Installation

Build from source using [cargo]:

```bash
cargo install hledger-fmt
```

Install from pre-built binaries using [cargo-binstall]:

```bash
cargo binstall hledger-fmt
```

We don't currently provide standalone pre-built binaries.

### pre-commit

Use it with [pre-commit] by adding the hook to your _.pre-commit-config.yaml_:

```yaml
repos:
  - repo: https://github.com/mondeja/hledger-fmt
    rev: vX.Y.Z
    hooks:
      # id: hledger-fmt        # Use this id to format files in place
      - id: hledger-fmt-check # Use this id to check files without formatting
```

## Usage

When you don't pass files to format, it reads all the files with
the extensions `.journal`, `.hledger` and `.j` in the current directory
and its subdirectories.

```bash
hledger-fmt [OPTIONS] [FILES]...
```

To fix them in place, use the `--fix` option:

```bash
hledger-fmt --fix [FILES]...
```

See `hledger-fmt --help` for more information.

## Features

- **`color`** (enabled by default): Build with terminal color support.

## Roadmap

This project is in an alpha stage and currently the formatting is
opinionated based on my preferences, but configuration options are
accepted to make it more flexible. Send a PR if you want to help.

[cargo-binstall]: https://github.com/cargo-bins/cargo-binstall
[hledger]: https://hledger.org
[cargo]: https://doc.rust-lang.org/cargo/
[pre-commit]: https://pre-commit.com
