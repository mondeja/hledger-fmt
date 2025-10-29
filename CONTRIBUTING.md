# Contributing

## AI Contributors

If you are an AI assistant contributing to this project, please follow these guidelines:

### Pre-commit Hooks

Install [prek](https://github.com/j178/prek/) and set up the pre-commit hooks:

```sh
prek install
```

Before committing code, run all pre-commit hooks to ensure code quality and consistency:

```sh
prek run -a
```

### Building and Testing

Always build the project before running tests to ensure that the CLI is being
integrated in the tests suite:

```sh
cargo build && cargo test
```

### Coding Style

Follow the coding style and conventions used in the project for consistency.
The pre-commit hooks will help enforce these standards.

### Commit Messages

Write clear and concise commit messages that describe the changes made.

### CHANGELOG.md

Write very brief PR descriptions in _CHANGELOG.md_ following the structure of
the current changelog. Add entries under the appropriate section names (e.g.,
"New features", "Enhancements", "Bug fixes", "Breaking changes").

Add the relevant links to the _CHANGELOG.md_ sections in Markdown reference
links at the bottom of the file.

### Benchmarks

Run benchmarks with the `--release` flag to get accurate performance measurements:

```sh
cargo bench --release --features bench
```

### Additional Guidelines

Read this _CONTRIBUTING.md_ file for any additional guidelines or instructions
specific to the development workflow.

## Running tests

```sh
cargo build && cargo test
```

## Running benchmarks

```sh
cargo bench --features bench
```

## Debugging with tracing

```sh
echo ' ; comment' | cargo run --features tracing -- -
```

```sh
echo ' ; comment' | cargo run --features tracing -- - --trace-file trace.log
```
