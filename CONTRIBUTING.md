# Contributing

## Running tests

```sh
RUST_LOG=trace cargo test
```

## Running benchmarks

```sh
cargo bench --features bench
```

## Debugging with tracing

```sh
echo ' ; comment' | RUST_LOG=trace cargo run --features tracing -- -
```
