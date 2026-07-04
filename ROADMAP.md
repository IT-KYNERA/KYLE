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

**Benchmarks (Kyle vs Rust vs C) — actualizado Jul 2026:**

Ejecutado en Apple M4 (ARM64). Compiladores: `clang -O3`, `rustc -O`, `ky build`.

| Benchmark | Kyle | Rust | C | Relación (K/C) |
|-----------|:----:|:----:|:-:|:--------------:|
| Primes 3M | 672ms | 467ms | 444ms | 1.51x |
| Fibonacci 40 | 406ms | 423ms | 410ms | **0.99x** |
| Mandelbrot (float) | 294ms | 274ms | 271ms | 1.08x |
| String concat 100k | 1.98s | 272ms | 238ms | 8.3x |
| **Compile memory** | **7.0MB** | 18.9MB | 2.6MB | **2.7x vs Rust** |

Kyle compite directamente con C y Rust en CPU-bound (primes, fib, mandelbrot).
La brecha en strings se debe a que Kyle usa `ky_concat` (nueva alloc + copy en cada `+`).
Rust usa `String::push` que amplía el buffer existente.

Kyle usa 2.7x menos RAM que Rust durante compilación (sin LTO).

Ver `BENCHMARK.md` para el suite completo y `docs/` para features faltantes.

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

## Runtime Rewrite Analysis — What Kyle Can and Cannot Do

The runtime (`libkyc_runtime.a`) has **88 `extern "C"` functions**. Here's exactly what can be rewritten in pure Kyle (+ `extern fn` to libc) and what cannot.

### ✅ Can rewrite NOW (65 functions — 74%)

These use only arithmetic, raw pointers, and libc FFI. All doable with current Kyle features.

| Module | Count | Examples |
|--------|:-----:|----------|
| `string.rs` | 20 | `ky_strlen`, `ky_concat`, `ky_str_to_i64`, `ky_str_contains`, `ky_substr`, etc. |
| `list.rs` | 28 | `ky_list_new/push/get/set/pop/len`, `ky_list_map/filter/fold`, `ky_range`, etc. |
| `io.rs` | 10 | `ky_print/println`, `ky_open/read/write/close`, `ky_sleep/now` |
| `lib.rs` | 4 | `ky_pow`, `ky_add_pct`, `ky_sub_pct`, `ky_mul_pct` |
| `assert.rs` | 1 | `ky_assert` |
| `async_.rs` | 1 | `ky_yield` |

**Total: 65 functions. Estimated: 2-3 days.**

### 🔶 Needs missing Kyle feature (12 functions — 14%)

| Function | What's Missing | Workaround |
|----------|---------------|------------|
| `ky_alloc/ky_free` | **Heap allocator** | Add `extern fn malloc(size: i64) ptr` ✅ already possible |
| `ky_retain/ky_release` | **Atomic operations** | Needs LLVM `atomicrmw` instruction or `__sync_fetch_and_add` via extern |
| `ky_iter_new/next/map/filter/free` (5) | **`Box::new` + function ptr transmute** | Use `extern fn malloc` for heap + manual vtable |
| `ky_f64_to_str` | **f64 formatting** | Use `extern fn snprintf(buf, size, fmt, val)` from libc |
| `ky_assert_eq/assert_ne` | **i64 error message formatting** | Use `ky_i64_to_str` (from ✅ group) |

**Estimated: 2-3 days (after adding missing extern fn declarations).**

### ❌ Cannot rewrite (12 functions — 14%)

| Module | Count | Why |
|--------|:-----:|-----|
| `dict.rs` | 10 | **Needs hash map** — all 8 dict functions + 2 unimplemented (`ky_dict_contains`, `ky_dict_remove`). Dict is `HashMap<String, i64>` from Rust std. Kyle needs a hash table implementation (FNV-1a + open addressing, ~200 lines of Kyle). |
| `thread.rs` | 2 | **Needs OS threads** — `ky_spawn_thread` needs `pthread_create` with a C-compatible trampoline. `ky_join_thread` needs `pthread_join`. The trampoline requires codegen support for `extern "C"` closures. |
| `async_.rs` | 2 | **Needs async executor** — `ky_spawn_task` and `ky_await_task` depend on threads, channels, mutexes, atomics, and a global executor singleton. This is the most complex piece. |

**Estimated: 2-3 weeks (hash map: 1 week, threads: 1-2 weeks, async: depends on threads).**

### Summary

| Status | Count | % |
|--------|:-----:|:-:|
| ✅ Can rewrite NOW | 65 | 74% |
| 🔶 Needs minor feature | 12 | 14% |
| ❌ Needs major feature | 12 | 14% |
| **Total** | **88** | **100%** |

---

## Runtime Rewrite Plan

### Phase A — Low-hanging fruit (2-3 days)
Rewrite 65 functions that are pure Kyle + libc FFI:
- All of `string.rs` (except `ky_f64_to_str`)
- All of `list.rs` (iterator functions use `extern fn malloc`/`free`)
- All of `io.rs` (direct `extern fn` to libc: read, write, open, close, clock_gettime, etc.)
- `lib.rs` utilities, `ky_assert`, `ky_yield`

### Phase B — Missing extern declarations (1 day)
Add remaining `extern fn` declarations for:
- `malloc`, `free`, `calloc`, `realloc` from libc
- `snprintf` from libc (for `ky_f64_to_str`)
- `__sync_fetch_and_add`, `__sync_fetch_and_sub`, `__sync_synchronize` (GCC builtins)

### Phase C — Hash map in Kyle (1-2 weeks)
Implement `final class Dict<K, V>` in pure Kyle with:
- FNV-1a hash function
- Open addressing with linear probing
- Dynamic resizing
- Wraps `extern fn malloc`/`free` for backing store

### Phase D — Threading & Async (2-4 weeks)
- `ky_spawn_thread`/`ky_join_thread` via `pthread_create` + C trampoline
- Rebuild async executor on top of Kyle threads + channels

### Phase E — Self-hosting compiler (4-8 weeks, low priority)
- Declare ~85 LLVM C API functions as `extern fn`
- Rewrite codegen logic (~2,400 lines) from Rust to Kyle
- Compiler written in Kyle compiles itself

---

## Implementation Order

```
NOW → Phase 0 (extern fn, @link, ptr) — ✅ DONE
   ↓
      Packages (http, json, sqlite, env — pure Kyle) — ✅ DONE
         ↓
             Fase 1 — Function pointers (fn() como tipo) — ✅ DONE
               ↓
                  Fase 2 — JsonValue + auto-serialize — ✅ DONE
                     ↓
                        Fase 3 — HTTP Client JSON integrado — ✅ DONE
                           ↓
                              Fase 4 — HTTP Server routing real — 🔜
                                 ↓
                                    Fase 5 — WebSocket + SSE — 🔜
                                       ↓
                                          Runtime Phase A (65 functions) — 📅
                                             ↓
                                                Runtime Phase B (missing externs) — 📅
                                                   ↓
                                                      Runtime Phase C (hash map) — 📅
                                                         ↓
                                                            Runtime Phase D (threading) — 📅
                                                               ↓
                                                                  Self-hosting — low priority 📅
```

### Fases HTTP/JSON

| Fase | Descripción | Depende de | Estado |
|------|-------------|------------|--------|
| 1 | Function pointers (`fn()` como tipo de primera clase) | Compiler | ✅ |
| 2 | `JsonValue` type + auto-serialize de `final class` | Union types | ✅ |
| 3 | HTTP Client: `client.post(url, class)` auto-JSON | Fase 2 | ✅ |
| 4 | HTTP Server: callbacks, `{id:i32}` params, middleware | Fase 1 + 3 | 🔜 |
| 5 | WebSocket + SSE sobre Server | Fase 4 | 🔜 |

**Current state:** Packages work (http client, json, sqlite, env) in 100% Kyle with FFI. HTTP Server TCP accept working. Function pointers implemented. Benchmarks completados (Kyle vs Rust vs C). Bugs recientes arreglados: constructor sin `fn`, `@link` propagation, `from X import a, b, c`, tuple destructuring, `close()` name conflict. Runtime is 74% rewritable now.

### Bugs conocidos (no críticos)

| Bug | Impacto | Workaround |
|-----|---------|------------|
| Closures como argumentos truncados a i32 | Bajo | Usar funciones nombradas |

## Package Registry

Kyle packages are distributed via **GitHub Pages registry** at `https://IT-KYNERA.github.io/KYLE/docs`. No `KL_REGISTRY` needed.

### Available packages

| Package | Registry | Status |
|---------|----------|--------|
| `http` | GitHub Pages | ✅ v3.0 (cliente) |
| `json` | GitHub Pages | ✅ v0.1.0 |
| `sqlite` | GitHub Pages | ✅ v0.1.0 |
| `env` | GitHub Pages | ✅ v0.1.0 |

### Development

```bash
# Local — apuntar al file registry del repo
export KL_REGISTRY=file:///Users/me/KYLE/registry
ky add http
```

See `docs/05-packages/registry.md` for full documentation.

## Self-Hosting — Codegen Analysis

To compile Kyle with Kyle, the compiler's codegen (`kyc_backend/src/codegen.rs`, ~2,400 lines of Rust) must be rewritten in Kyle. It currently uses **inkwell** (Rust wrapper for LLVM C API).

### What would be needed

| Component | Lines | Difficulty |
|-----------|:-----:|:----------:|
| LLVM C API `extern fn` declarations | ~85 functions | 🟢 Easy (one-time typing) |
| LLVM type wrappers (`LLVMValueRef`, etc.) | ~100 | 🟢 Easy (all `ptr`) |
| Codegen logic (translate MIR → LLVM IR) | ~2,400 | 🔴 Hard (complex dispatch logic) |
| SSA construction | ~920 | 🟡 Medium (already pure algorithms) |
| Linker driver | ~150 | 🟢 Easy (`system("clang ...")` already works) |

### LLVM is NOT replaced

LLVM stays as the machine-code backend. The same way we call libcurl via `extern fn`, we would call LLVM via `extern fn LLVMBuildAdd(...)`. The ~85 LLVM C API functions map 1:1 to the existing inkwell calls.

**Verdict:** Technically feasible. Not a priority — the Rust compiler is stable and the codegen doesn't need to be self-hosted to ship packages.

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

## 📊 Fases de Madurez del Lenguaje (Benchmark gaps)

Features identificadas en los benchmarks que Kyle necesita para ser competitivo como lenguaje de bajo nivel:

| Feature | Importancia | Dependencia | ETA |
|---------|-------------|-------------|-----|
| **SIMD intrinsics** (AVX, NEON) | ⭐⭐⭐⭐⭐ | Codegen + runtime | 📅 Fase 19 |
| **Threads + Concurrency** | ⭐⭐⭐⭐⭐ | Runtime | 📅 Fase D |
| **Async/Await en Kyle** | ⭐⭐⭐⭐ | Runtime + compiler | 📅 Fase D |
| **HashMap completo** (String→any) | ⭐⭐⭐⭐ | Runtime | 📅 Fase C |
| **Networking TCP/UDP** | ⭐⭐⭐⭐ | Runtime + packages | 📅 Fase 4-5 |
| **WebSocket/SSE** | ⭐⭐⭐ | Packages | 📅 Fase 5 |
| **Regex** | ⭐⭐⭐ | Package | 📅 Futuro |
| **Crypto** | ⭐⭐⭐ | Package | 📅 Futuro |
| **Compression** (gzip, brotli) | ⭐⭐⭐ | Package | 📅 Futuro |
| **PGO** (Profile Guided Optimization) | ⭐⭐⭐ | Toolchain | 📅 Futuro |
| **Cache Miss / IPC profiling** | ⭐⭐ | Toolchain | 📅 Futuro |
| **Arena/Pool allocators** | ⭐⭐ | Runtime | 📅 Futuro |
| **Vectorization control** (LLVM) | ⭐⭐ | Compiler | 📅 Futuro |

Ver `BENCHMARK.md` para resultados completos.
