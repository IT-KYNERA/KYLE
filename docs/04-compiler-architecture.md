# Kyle Compiler Architecture

> The 9-crate pipeline, repository layout, and runtime internals.

---

## Pipeline

```
Source (.kl)
    │
    ▼
[Lexer]        klc_frontend/src/lexer.rs    — text → tokens
    │
    ▼
[Parser]       klc_frontend/src/parser.rs   — tokens → AST
    │
    ▼
[Semantic]     klc_semantic/                 — resolve symbols, check types
    │
    ▼
[MIR Lowering] klc_mir/src/lower.rs          — AST → MIR (our IR)
    │
    ▼
[Optimizer]    klc_mir/src/optimize.rs       — constant folding, DCE
    │
    ▼
[Ownership]    klc_mir/src/ownership.rs      — RAII retain/release inference
    │
    ▼
[Codegen]      klc_backend/src/codegen.rs   — MIR → LLVM IR (inkwell)
    │
    ▼
[Linker]       klc_backend/src/linker.rs     — .o + runtime → native binary
```

---

## Crates

| Crate | Responsibility | Key Files |
|-------|---------------|-----------|
| `klc_core` | AST, Span, Types, SourceMap, Diagnostics | `ast.rs` (1076 lines), `types.rs`, `span.rs`, `diagnostic.rs` |
| `klc_frontend` | Lexer + Parser | `lexer.rs` (809 lines), `parser.rs` (1353 lines), `token.rs` |
| `klc_semantic` | Symbol resolution, type checking | `type_checker.rs` (1380 lines), `symbol_table.rs`, `scope.rs` |
| `klc_mir` | MIR definition, lowering, optimization, ownership | `mir.rs`, `lower.rs` (860+ lines), `optimize.rs`, `ownership.rs` |
| `klc_backend` | LLVM codegen + linking | `codegen.rs` (479 lines), `linker.rs` |
| `klc_driver` | Pipeline orchestration | `pipeline.rs` (217 lines) |
| `klc_cli` | CLI binary (`kl` + `klc`) | `main.rs` (505 lines) |
| `klc_runtime` | RAII runtime, async, channels, string/list/dict ops | `string.rs`, `io.rs`, `list.rs`, `dict.rs`, `thread.rs`, `async_.rs`, `channel.rs`, `error.rs` |
| `klc_tools` | LSP, formatter, package manager | `lsp.rs`, `formatter.rs`, `package/manifest.rs`, `package/lock.rs`, `package/project.rs` |

---

## Repository Layout

```
kl/
├── AGENTS.md                  — AI agent context (session log, state)
├── Cargo.toml                 — Rust workspace root
├── kl.toml                    — Kyle project manifest
├── .cargo/config.toml         — LLVM config (Linux)
├── .github/workflows/ci.yml   — CI pipeline
│
├── crates/                    — 9 Rust crates
│   ├── klc_core/              — AST, Span, Types, Diagnostics
│   ├── klc_frontend/          — Lexer + Parser
│   ├── klc_semantic/          — Type checker, symbol resolver
│   ├── klc_mir/               — MIR definition, lowering, optimization
│   ├── klc_backend/           — LLVM codegen (inkwell), linker
│   ├── klc_driver/            — Pipeline orchestration
│   ├── klc_cli/               — CLI binary (kl + klc)
│   ├── klc_runtime/           — RAII runtime, async, channels, I/O
│   └── klc_tools/             — LSP, formatter, package manager
│
├── docs/                      — 6 specification documents
├── examples/                  — 50+ example .kl programs
├── std/                       — Standard library (8 .kl modules)
└── vscode-kl/                 — VS Code extension (.vsix)
```

---

## Runtime Crate (klc_runtime/src/)

| File | Responsibility |
|------|---------------|
| `string.rs` | String ops: contains, to_upper, to_lower, trim, replace, concat, input, char ops, ord |
| `io.rs` | File I/O: open, read_str, write_str, close, sleep, now |
| `list.rs` | List ops: new, add, get, len, pop, slice, extend, range |
| `dict.rs` | Dict ops: new, set, get, len, free |
| `thread.rs` | Thread spawn/join (kl_spawn_thread, kl_join_thread) |
| `async_.rs` | Async runtime (work-stealing pool — target design; current impl is thread-based) |
| `channel.rs` | Channel<T> for inter-thread communication |
| `error.rs` | Error handling, panic handler |
| `lib.rs` | Runtime entry point, kl_alloc/kl_free, entry_point wrapper |

### Memory Management

The runtime provides `kl_alloc(size)` and `kl_free(ptr)` for RAII heap allocation.
The compiler's Ownership Inference Pass (`klc_mir/src/ownership.rs`) automatically
inserts `kl_release` calls at block exits — the developer never writes `free`.

Key runtime ABI functions:

```
kl_alloc(size: i64) -> *void      — heap allocation
kl_free(ptr: *void)                — heap deallocation
kl_retain(ptr: *void)              — increment reference count
kl_release(ptr: *void)            — decrement + free if zero
kl_print(str, len) / kl_println   — output
kl_str_concat(a, a_len, b, b_len) — string concatenation
kl_list_new() / kl_list_add(...)  — list operations
kl_dict_new() / kl_dict_set(...)  — dict operations
kl_spawn_thread(fn, arg) -> handle — async task spawn
kl_join_thread(handle) -> result   — async task await
```

---

## Standard Library (std/)

8 flat `.kl` modules (no subdirectories):

| File | Contents |
|------|----------|
| `core.kl` | Option<T>, unwrap_or, is_some, is_none |
| `math.kl` | abs, pow, sqrt, gcd, min, max, clamp |
| `io.kl` | File read/write wrappers |
| `str.kl` | starts_with, ends_with, capitalize, repeat_str |
| `testing.kl` | assert, assert_eq, assert_str, assert_ne |
| `collections.kl` | list_sum, list_product, list_max, list_min, list_range |
| `json.kl` | json_parse + json_stringify (via runtime FFI) |
| `time.kl` | timestamp, sleep_ms, seconds_since |

---

## Compilation Modes

| Mode | Command | Optimization | Output |
|------|---------|-------------|--------|
| Debug (default) | `kl build` / `kl run` | O0 / Default | `target/debug/<name>` |
| Release | `kl build --release` | O2/O3 (Phase 9) | `target/release/<name>` |

**Note:** Currently both modes use LLVM `OptimizationLevel::Default`. Full O2/O3
optimization for release mode is planned for Phase 9.

---

## Development Commands

```bash
cargo build --workspace                    # Build all crates
cargo run --bin kl -- run <file.kl>        # Compile and run
cargo run --bin kl -- build <file.kl>       # Compile to native binary
cargo run --bin kl -- check <file.kl>      # Type-check only
cargo run --bin kl -- fmt <file.kl>        # Format source
cargo run --bin kl -- lsp                   # Start LSP server
cargo test -p klc_core -p klc_frontend -p klc_semantic -p klc_mir -p klc_runtime -p klc_tools  # 101 tests
```

---

## Version

```
Compiler Architecture v2.0
Last updated: 2026-06-26
```