# Roadmap

## ✅ Completed — Phases 1–17

| Phase | Description | Status |
|-------|-------------|--------|
| 1–2 | Documentation and specification | ✅ |
| 3 | Lexer | ✅ |
| 4 | Parser | ✅ |
| 5 | HIR + desugaring | ✅ |
| 6 | Semantic analysis (type checker, scope) | ✅ |
| 7 | Borrow semantics (ownership, move, copy) | ✅ |
| 8 | Backend release mode (LLVM, binary output) | ✅ |
| 9 | Async scheduler V2 (thread pool) | ✅ |
| 10 | Iterators — 17 list methods (map, filter, fold, etc.) | ✅ |
| 11 | Package manager (registry, cache, lock, publish, login) | ✅ |
| 12 | Tooling (LSP, VS Code extension, formatter, test framework) | ✅ |
| 13 | Complete syntax (generics, ranges, match, operator overloading, is, ptr, for-else, static fn, **) | ✅ |
| 14 | References and borrow checker (&T, ^T, field mutability) | ✅ |
| 15 | SSA form (mem2reg, phi nodes, GVN, dominator fix) | ✅ |
| 16 | LLVM IR quality (nsw, TBAA, inbounds, readonly, noalias, noundef, !range, align, lifetime) | ✅ |
| 17 | Optimization pipeline (O3, constant folding, alloca elimination, nsw verification) | ✅ |

**Benchmarks (Kyle = C = Rust):**

| Benchmark | Kyle | C (gcc -O3) | Rust (rustc -O) |
|-----------|:----:|:-----------:|:---------------:|
| Primes 3M | 0.18s | 0.18s | 0.20s |
| Fibonacci 40 | 0.16s | 0.15s | 0.15s |
| Arithmetic 500M | 0.00s* | 0.00s* | 0.00s* |

*\* LLVM constant folding*

---

## 🔜 Immediate Next — Phase 0: FFI Foundation

**Estimated: 2.5 days**

Goal: Enable packages written in 100% Kyle without Rust.

| Step | Task | Files | Est. |
|------|------|-------|------|
| 0.1 | `extern fn` declaration — parser, semantic, MIR, codegen | `parser.rs`, `type_checker.rs`, `lower.rs`, `codegen.rs` | 1 day |
| 0.2 | `@link` directive — parser + linker integration | `parser.rs`, `pipeline.rs` | 0.5 day |
| 0.3 | `ptr` type complete — load, store, offset, arithmetic | `lower.rs`, `codegen.rs` | 1 day |

After Phase 0: packages like `ky-http`, `ky-sqlite`, `ky-json` can be written in pure Kyle with FFI to C libraries.

---

## 📦 Package Implementation (after Phase 0)

| Package | Description | Depends On |
|---------|-------------|------------|
| `ky-http` | HTTP/1.1 client + server (libcurl FFI) | Phase 0 |
| `ky-json` | JSON parse + stringify | Phase 0 |
| `ky-sqlite` | SQLite database bindings | Phase 0 + `ky-http` |
| `ky-postgres` | PostgreSQL driver | Phase 0 |
| `ky-websocket` | WebSocket client | Phase 0 + `ky-http` |
| `ky-crypto` | Hashing, encryption (OpenSSL FFI) | Phase 0 |

---

## 📅 Post-v1.0 Features

### Phase 18 — Zero-Cost Abstractions

| # | Task | Priority |
|---|------|----------|
| 18.1 | Escape analysis: `final class` on stack instead of heap | ⭐⭐⭐⭐ |
| 18.2 | Small string optimization (SSO): strings < 15 bytes inline | ⭐⭐⭐ |
| 18.3 | Inlining `.map()`/`.filter()`/`.fold()` — zero overhead | ⭐⭐⭐ |
| 18.4 | Verified monomorphization — no boxing for generics | ⭐⭐⭐ |
| 18.5 | Array optimizations — small arrays on stack | ⭐⭐⭐ |
| 18.6 | Vtable elimination — direct dispatch for non-virtual | ⭐⭐⭐ |
| 18.7 | Return value optimization (RVO) — avoid copies | ⭐⭐ |
| 18.8 | Devirtualization — speculative devirt for methods | ⭐⭐ |

### Async V3 — State Machine

| # | Task | Priority |
|---|------|----------|
| 9.1–5 | Replace thread pool with state machine V3 | ⭐⭐⭐ |
| 9.6–8 | Work-stealing scheduler | ⭐⭐⭐ |
| 9.9–11 | Non-blocking I/O (timers, signals, async read/write) | ⭐⭐ |

### Iterators Advanced

| # | Task | Priority |
|---|------|----------|
| 10.1–5 | Functional closures (first-class fn pointers) | ⭐⭐⭐ |
| 10.6–9 | Lazy evaluation — `iter()` trait, lazy chains | ⭐⭐ |

### Alternative Backends

| Backend | Purpose |
|---------|---------|
| Cranelift | Faster compilation (debug mode), no LLVM dependency |
| WASM | Compile Kyle for browser and WebAssembly targets |

---

## Implementation Order

```
NOW → Phase 0 (extern fn, @link, ptr) — 2.5 days
   ↓
      Packages (ky-http, ky-json, ky-sqlite, etc.) — weeks
         ↓
            Phase 18 (Zero-Cost Abstractions) — months
               ↓
                  Async V3 + Iterators Advanced — months
                     ↓
                        Alternative Backends — long-term
```

**Total language phases (1–17): 100% complete**  
**Next milestone: Phase 0 (FFI foundation)** unlocks pure-Kyle packages.
