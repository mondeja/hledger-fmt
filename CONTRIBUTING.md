# Contributing

## Running tests

```sh
cargo build && cargo test
```

## Running benchmarks

```sh
cargo bench --release --features bench
```

## Debugging with tracing

```sh
echo ' ; comment' | cargo run --features tracing -- -
```

```sh
echo ' ; comment' | cargo run --features tracing -- - --trace-file trace.log
```
