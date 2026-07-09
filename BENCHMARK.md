# Kyle Benchmark Results

Date: $(date '+%Y-%m-%d')
Machine: Apple Silicon (M5)

All Kyle benchmarks use `--release` (LLVM O3 pipeline with inline list ops).
C/Rust use `-O3 -march=native`, Go uses `go build -o`.

## Results (average of 5 runs, 3 warmup)

| Benchmark | C | Rust | **Kyle** | vs C | 
|-----------|---|---|----------|------|
| **Fib 50M** | 110ms | 110ms | **170ms** | 1.55x |
| **Primes 3M** | <1ms | <1ms | **14ms** | — |
| **Concat 500k** | <1ms | <1ms | **2ms** | — |
| **Matmul 100x100** | <1ms | <1ms | **<1ms** | — |

## Notes

- **Fib**: 1.55x vs C. All add/sub/mul now have `nsw`+`nuw` flags for LLVM optimization. The remaining gap is from Kyle's `while` loop generating a 2-block structure (header + body), while C's `for` is optimized to a single-block loop. Future work: loop rotation in MIR or `!llvm.loop` metadata.
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
