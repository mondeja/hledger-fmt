# hledger-fmt

[![Crates.io](https://img.shields.io/crates/v/hledger-fmt?logo=rust)](https://crates.io/crates/hledger-fmt)
[![License](https://img.shields.io/crates/l/hledger-fmt)](https://github.com/mondeja/hledger-fmt/blob/master/LICENSE)
[![Tests](https://img.shields.io/github/actions/workflow/status/mondeja/hledger-fmt/ci.yml?label=tests&logo=github)](https://github.com/mondeja/hledger-fmt/actions)
[![Crates.io downloads](https://img.shields.io/crates/d/hledger-fmt)](https://crates.io/crates/hledger-fmt)

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

## Usage

When you don't pass files to format, it reads all the files with
the extensions `.journal`, `.hledger` and `.j` in the current directory
and its subdirectories.

```bash
hledger-fmt [files...]
```

To fix them, use the `--fix` option:

```bash
hledger-fmt --fix [files...]
```

See `hledger-fmt --help` for more information.

## Features

- **color** (enabled by default): Build with terminal color support.

## Roadmap

This project is in an alpha stage and currently the formatting is
opinionated based on my preferences, but configuration options are
accepted to make it more flexible. Send a PR if you want to help.

[cargo-binstall]: https://github.com/cargo-bins/cargo-binstall
[hledger]: https://hledger.org
[cargo]: https://doc.rust-lang.org/cargo/
