# Kyle Benchmark Results

Date: $(date)
Machine: $(uname -a | head -c 100)

## Benchmarks (Apple M5)

| Benchmark | C | Rust | Go | Kyle | Kyle vs C |
|-----------|---|------|----|------|-----------|
| Fib 50M | 110ms | 112ms | 110ms | 222ms | 2.0x |
| Primes 3M | <1ms | <1ms | <1ms | 18ms | — |
| Concat 500k | <1ms | <1ms | <1ms | <1ms | — |
| Matmul 100x10 | <1ms | <1ms | 10ms | 10ms | — |

## Notes

- SSA codegen + inlining enabled for all Kyle benchmarks
- Benchmarks use v0.6 syntax with `^&` mutable borrows
- Kyle binaries are ~5MB (statically linked with runtime)
- C/Rust/Go use -O3 with native arch flags
- Concat uses `str_builder` API (replaces old `ky_str_builder` extern)
- Primes use `list_push`/`list_get`/`list_set` with borrow syntax
