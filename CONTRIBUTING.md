# Contributing

## Running tests

```sh
cargo test
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
