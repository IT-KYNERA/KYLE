# Kyle Benchmark Results

Date: Sat Jul  4 16:59:02 -04 2026
Machine: Darwin macbook.local 25.5.0 Darwin Kernel Version 25.5.0: Mon Apr 27 20:39:29 PDT 2026; root:xnu-12377.121.6~2/RELEASE_ARM64_T8142 arm64

## 1. Primes (3M)
### Compilation
| Metric | Kyle | Rust |
|--------|------|------|
| Binary size | 1514584 | 465816 |
| Stripped size | 1062088 | 341512 |

### Execution
| Lang | Time | Result |
|------|------|--------|
| Kyle | 613ms|216816 |
| Rust | 448ms|216816 |

## 2. Fibonacci 40
| Lang | Time | Result |
|------|------|--------|
| Kyle | 439ms|102334155 |
| Rust | 398ms|102334155 |

## 3. String concat (100k)
| Lang | Time | Result |
|------|------|--------|
| Kyle | 1958ms|100000 |
| Rust | 252ms|100000 |

## 4. List push (100k)
| Lang | Time | Result |
|------|------|--------|
| Kyle | BIN_NOT_FOUND| |
| Rust | 244ms|100000 |

## 5. Mandelbrot (float)
| Lang | Time |
|------|------|
| Kyle | 264ms|54151 |
| Rust | 232ms|54151 |

## 6. Compilation Memory (primes)
| Lang | Peak Memory |
|------|-------------|
| Kyle | 6996424 |
| Rust | 18874992 |

---
## Features NOT benchmarked
| Feature | Why | ETA |
|---------|-----|-----|
| SIMD (AVX, NEON) | Kyle no expone intrinsics | 📅 Fase 19 |
| Concurrency (threads, mutex) | No implementado en Kyle | 📅 Fase D |
| Async/Await | Runtime existe, no expuesto | 📅 Fase D |
| TCP/UDP/HTTP Server | Packages en desarrollo | 📅 Fase 4-5 |
| HashMap avanzado | Dict → i64 limitado | 📅 Fase C |
| Regex/Crypto/Compression | No existen packages | 📅 Futuro |
| PGO | No implementado en toolchain | 📅 Futuro |
| Cache Miss, IPC, Branch Miss | Requiere perf (Linux) | 📅 Futuro |
| WebSocket/SSE | No implementado | 📅 Fase 5 |
| LLVM Vectorization control | Kyle no expone atributos | 📅 Futuro |
| Arena/Pool allocators | No implementados | 📅 Futuro |

