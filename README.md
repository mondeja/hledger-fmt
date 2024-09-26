# hledger-fmt

An [hledger]'s journal files formatter.

## Installation

```bash
cargo install hledger-fmt
```

From pre-built binaries using [cargo-binstall]:

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

## Roadmap

This project is in an alpha stage and currently the formatting is
opinionated based on my preferences, but configuration options are
accepted to make it more flexible. Send a PR if you want to help.

[cargo-binstall]: https://github.com/cargo-bins/cargo-binstall
[hledger]: https://hledger.org