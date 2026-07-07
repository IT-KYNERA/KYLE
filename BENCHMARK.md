# Kyle Benchmark Results

Date: Sat Jul  4 17:03:02 -04 2026
Machine: Darwin macbook.local 25.5.0 Darwin Kernel Version 25.5.0: Mon Apr 27 20:39:29 PDT 2026; root:xnu-12377.121.6~2/RELEASE_ARM64_T8142 arm64

## 1. Primes (3M)
### Compilation
| Metric | Kyle | Rust | C |
|--------|------|------|---|
| Binary size | 1514584 | 465816 | 33464 |
| Stripped | 1062088 | 341512 | 33448 |

### Execution
| Lang | Time | Result |
|------|------|--------|
| Kyle | 672ms|216816 |
| Rust | 467ms|216816 |
| C    | 444ms|216816 |

### Compilation Memory
| Lang | Peak Memory |
|------|-------------|
| Kyle | 7045576 |
| Rust | 18858608 |
| C    | 2572600 |

## ## 2. Fibonacci 40
| Lang | Time | Result |
|------|------|--------|
| Kyle | 406ms|102334155 |
| Rust | 423ms|102334155 |
| C    | 410ms|102334155 |

## ## 3. String concat (100k)
| Lang | Time | Result |
|------|------|--------|
| Kyle | 1982ms|100000 |
| Rust | 272ms|100000 |
| C    | 238ms|100000 |

## ## 4. List/Vector push (100k)
| Lang | Time | Result |
|------|------|--------|
| Kyle | BIN_NOT_FOUND| |
| Rust | 258ms|100000 |
| C    | 244ms|100000 |

## ## 5. Mandelbrot (float)
| Lang | Time | Result |
|------|------|--------|
| Kyle | 294ms|54151 |
| Rust | 274ms|54151 |
| C    | 271ms|21090 |

---
## Features NOT benchmarked
| Feature | Why | ETA |
|---------|-----|-----|
| SIMD (AVX, NEON) | Kyle no expone intrinsics | 📅 Fase 19 |
| Concurrency (threads, mutex) | No implementado en Kyle | 📅 Fase D |
| Async/Await | Runtime existe, no expuesto | 📅 Fase D |
| TCP/UDP/HTTP Server | Packages en desarrollo | 📅 Fase 4-5 |
| HashMap avanzado | Dict → i64 limitado | 📅 Fase C |
| regex/Crypto/Compression | No existen packages | 📅 Futuro |
| PGO | No implementado en toolchain | 📅 Futuro |
| Cache Miss, IPC, Branch Miss | Requiere perf (Linux) | 📅 Futuro |
| websocket/SSE | No implementado | 📅 Fase 5 |
| LLVM Vectorization control | Kyle no expone atributos | 📅 Futuro |
| Arena/pool allocators | No implementados | 📅 Futuro |

