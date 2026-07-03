# Roadmap

## Platform Architecture

Kyle is a layered platform. Each layer only knows the layer below it.

```
Applications (IDE, Desktop, KyleOS...)
    │
Kyle UI (widgets, navigation)
    │
Kyle Scene (scene graph, layout)
    │
Kyle Graphics (canvas, GPU)
    │
Kyle Windowing (windows, events)
    │
Kyle Platform (FS, net, threads, audio, sensors)
    │
Kyle Runtime (memory, strings, collections)
    │
Kyle Language (compiler)
```

The **compiler and runtime are complete** (Phases 1-17).  
The **upper layers (Windowing → Applications) are future work**.  
The **Platform layer is next** — it enables backend packages (HTTP, SQLite, etc.).

### Current crate structure

```
kyc_core/          ✅ foundation
kyc_frontend/      ✅ lexer + parser
kyc_hir/           ✅ desugaring
kyc_semantic/      ✅ type checker, scope
kyc_mir/           ✅ MIR lowering, SSA
kyc_backend/       ✅ LLVM codegen
kyc_driver/        ✅ pipeline
kyc_cli/           ✅ CLI binary
kyc_runtime/       ✅ runtime (memory, strings, lists, dicts)
kyc_tools/         ✅ LSP, formatter, package manager
```

### Planned crate additions

| Crate | When | Purpose |
|-------|------|---------|
| `kyc_platform` | After Phase 0 | Platform API (FS, net, time, env) — Rust crate |
| `kyc_platform_macos` | After Phase 0 | macOS platform adapter |
| `kyc_platform_linux` | Future | Linux platform adapter |
| Various `ky-*` | After Phase 0 | Kyle packages (HTTP, SQLite, JSON, etc.) |
| `kyc_graphics` | Long-term | Canvas, GPU rendering |
| `kyc_ui` | Long-term | Widget library |
| `kyc_scene` | Long-term | Scene graph |

---

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
**Crate restructuring: NOT required for Phase 0** — all changes are in existing compiler crates.

Goal: Enable packages written in 100% Kyle without Rust.

| Step | Task | Files | Est. |
|------|------|-------|------|
| 0.1 | `extern fn` declaration — parser, semantic, MIR, codegen | `parser.rs`, `type_checker.rs`, `lower.rs`, `codegen.rs` | 1 day |
| 0.2 | `@link` directive — parser + linker integration | `parser.rs`, `pipeline.rs` | 0.5 day |
| 0.3 | `ptr` type complete — load, store, offset, arithmetic | `lower.rs`, `codegen.rs` | 1 day |

After Phase 0: packages like `ky-http`, `ky-sqlite`, `ky-json` can be written in pure Kyle with FFI to C libraries.

---

## 🏗️ Phase 1 — Platform Crate Setup (after Phase 0)

**Estimated: 2-3 days**

| Step | Task |
|------|------|
| 1.1 | Create `kyc_platform` crate with platform-independent API interfaces |
| 1.2 | Create `kyc_platform_macos` adapter with macOS implementations |
| 1.3 | Move file I/O, networking, time from `kyc_runtime` to `kyc_platform` |
| 1.4 | Implement `@link` + `extern fn` workflow for platform FFI |
| 1.5 | Create package template for `ky-*` packages with native/ FFI |

## 📦 Package Implementation (after Phase 1)

| Package | Description | Depends On |
|---------|-------------|------------|
| `http` | HTTP/1.1 client + server (libcurl FFI) | Phase 1 |
| `json` | JSON parse + stringify | Phase 1 |
| `sqlite` | SQLite database bindings | Phase 1 |
| `postgres` | PostgreSQL driver | Phase 1 |
| `websocket` | WebSocket client | Phase 1 |
| `crypto` | Hashing, encryption (OpenSSL FFI) | Phase 1 |

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
      Phase 1 (Platform crate setup) — 2-3 days
         ↓
            Packages (ky-http, ky-json, ky-sqlite, etc.) — weeks
               ↓
                  Phase 18 (Zero-Cost Abstractions) — months
                     ↓
                        Windowing + Graphics + Scene — months
                           ↓
                              UI + Desktop + KyleOS — long-term
```

**Total language phases (1–17): 100% complete**  
**Next milestone: Phase 0 (FFI foundation)** unlocks pure-Kyle packages.
