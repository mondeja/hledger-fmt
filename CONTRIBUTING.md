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
echo ' ; comment' | cargo run --features tracing -- -
```

```sh
echo ' ; comment' | cargo run --features tracing -- - --trace-file trace.log
```
