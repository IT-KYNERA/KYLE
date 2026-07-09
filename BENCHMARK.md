# Kyle Benchmark Results

Date: $(date '+%Y-%m-%d')
Machine: Apple Silicon (M5)

All Kyle benchmarks use `--release` (LLVM O3 pipeline with inline list ops).
C/Rust use `-O3 -march=native`, Go uses `go build -o`.

## Results (average of 5 runs, 3 warmup)

| Benchmark | C | Rust | Go | **Kyle** | vs C | 
|-----------|---|---|----|----------|------|
| **Fib 50M** | 114ms | 112ms | 114ms | **180ms** | 1.58x |
| **Primes 3M** | <1ms | <1ms | <1ms | **16ms** | — |
| **Concat 500k** | <1ms | <1ms | <1ms | **2ms** | — |
| **Matmul 100x100** | <1ms | <1ms | 10ms | **<1ms** | — |

## Notes

- **Fib**: 1.58x vs C. The gap exists because Kyle's while-loop and i32 arithmetic go through LLVM without `nsw` flags (signed overflow is defined in Kyle), preventing some optimizations. Future work: add `nsw`/`nuw` for checked arithmetic.
- **Primes**: Runtime calls (`list_set`) are now inlined at the LLVM IR level, eliminating FFI overhead. The remaining gap vs C is because each access loads the data pointer from the list struct (LLVM can't hoist it due to opaque pointer aliasing). Future work: TBAA metadata.
- **Concat**: `str_builder` API wraps C-level string operations. Efficient enough (<2ms for 500k appends).
- **Matmul**: Inline list ops reduced time from 10ms to <1ms — same as C. LLVM hoists the data pointer and unrolls the inner loops.

## What Changed (v0.5.3 → v0.6)

### Syntax
- `l.push(v)` instead of `list_push(^&l, v)`
- `l.get(i)` instead of `list_get(&l, i)`
- `l.set(i, v)` instead of `list_set(^&l, i, v)`
- `l.len()` instead of `list_len(&l)`
- Method calls work inside loops (borrow checker fix)

### Performance
- Inline LLVM IR for `list_get`, `list_set`, `list_len` (no FFI overhead)
- Borrow checker dataflow fix (correct loop-body tracking)
- `ParamMode::MutableBorrow` for `this` in class methods
