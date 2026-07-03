# Roadmap

> **Fases 1-17 completadas.** Lenguaje compila, tooling listo, rendimiento = C/Rust.

## ✅ Completado (Fases 1-17)

| Fase | Descripción | Estado |
|------|-------------|--------|
| 1-2 | Documentación y especificación | ✅ |
| 3 | Lexer | ✅ |
| 4 | Parser | ✅ |
| 5 | HIR + Desugaring | ✅ |
| 6 | Semantic Analysis | ✅ |
| 7 | Borrow Semantics | ✅ |
| 8 | Backend Release Mode | ✅ |
| 9 | Async Scheduler V2 (thread pool) | ✅ |
| 10 | Iterators (17 métodos) | ✅ |
| 11 | Package Manager | ✅ |
| 12 | Tooling (LSP, VS Code, formatter, test) | ✅ |
| 13 | Sintaxis completa (genéricos, rangos, match, op overload, etc.) | ✅ |
| 14 | References & Borrow Checker | ✅ |
| 15 | SSA Form (mem2reg, phi, GVN) | ✅ |
| 16 | LLVM IR Quality (nsw, TBAA, inbounds, readonly) | ✅ |
| 17 | Optimization Pipeline (O3, const-fold, alloca elim) | ✅ |

## 📅 Post-v1.0

| Área | Items |
|------|-------|
| **Fase 18 — Zero-Cost** | Escape analysis, SSO, inlining, monomorfización, vtables, RVO, devirt |
| **Async V3** | State machine, work-stealing, non-blocking I/O |
| **Iterators avanzados** | Closures funcionales, lazy iterators |
| **Backends alternativos** | Cranelift (rápido), WASM (web) |

## Paquetes oficiales (próximos)

- `ky-http` — HTTP cliente + servidor
- `ky-json` — JSON parse + stringify
- `ky-sqlite` — SQLite bindings
- `ky-postgres` — PostgreSQL driver
- `ky-websocket` — WebSocket
- `ky-crypto` — Hashing, encriptación

## Benchmarks

| Benchmark | Kyle | C (gcc -O3) | Rust (rustc -O) |
|-----------|:----:|:-----------:|:---------------:|
| Primos 3M | **0.18s** | 0.18s | 0.20s |
| Fibonacci 40 | **0.16s** | 0.15s | 0.15s |
| Aritmética 500M | 0.00s* | 0.00s* | 0.00s* |

*\* LLVM constant-folding*
