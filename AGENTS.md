# Kyle — AI Agent Context

> This file is the bridge between AI agents (like Claude) and the Kyle codebase.
> It provides the essential context needed to continue development without
> re-reading the entire history. For full details, consult the 6 documentation
> files in `docs/`.

---

## What is Kyle?

Kyle is a compiled, statically-typed programming language targeting backend
systems and CLI tools. Written in Rust, compiled via LLVM 18. Combines:
- Python readability (indentation blocks, no semicolons, no `self`)
- Rust type safety (strong typing, generics, pattern matching)
- Go simplicity (fast compilation, built-in tooling)
- C performance (native code via LLVM)

**Implementation language:** Pure Rust. No C, no C++. The compiler, runtime,
and tooling are all Rust. This will NOT change.

---

## Current State (2026-06-27)

```
✅ Phases 0-6 COMPLETE   — Full compiler pipeline, all language features, std lib
✅ Phase 7 DONE          — macOS ARM ✅, Linux ARM ✅ (x64/Intel/Windows deferred)
✅ Phase 8 DONE          — Docs consolidation, kl binary alias, LSP polish,
                          VS Code .vsix, CI/CD, install scripts
⏸️ Phase 9 NEXT          — Backend & Systems (ENV, Process, FFI, HTTP, DB)

```
✅ Phases 0-6 COMPLETE   — Full compiler pipeline, all language features, std lib
✅ Phase 7 DONE          — macOS ARM ✅, Linux ARM ✅ (x64/Intel/Windows deferred)
✅ Phase 8 DONE          — Docs consolidation, kl binary alias, LSP polish,
                          VS Code .vsix, CI/CD, install scripts
⏸️ Phase 9 NEXT          — Backend & Systems (ENV, Process, FFI, HTTP, DB)
📅 Phase 10              — Std Library & Ergonomics (Iterators, Func ops, Collections)
📅 Phase 11              — Production Hardening (Errors, DWARF, TLS, WASM)
⏸️ Phase 12 DEFERRED     — Self-Hosting (compiler written in Kyle)
📅 Phase 13              — Ecosystem (Registry, Framework, Website)
```

**Tests:** 101 unit tests, 0 failures — all passing.

**Decision:** Phase 7 stays at macOS ARM + Linux ARM only. Linux x64, macOS
Intel, and Windows are deferred to future phases. Focus on the two platforms
that work.

---

## Documentation (the source of truth)

All docs are in `docs/`, all in English, all consolidated from 19 → 6:

| File | Content |
|------|---------|
| `00-vision.md` | Philosophy, design principles, comparison vs Python/Rust/Go |
| `01-language-reference.md` | Complete syntax + EBNF grammar + ✅/🔶/❌ status per construct |
| `02-types-errors-memory.md` | Type system, error handling, RAII memory, ABI, FFI |
| `03-modules-packages-tooling.md` | Modules, packages, CLI reference, getting started, VS Code |
| `04-compiler-architecture.md` | 9-crate pipeline, repo layout, runtime internals, std library |
| `05-roadmap-status.md` | Phases 0-13, implementation matrix, cross-platform, release checklist |

**Before starting any task, read `docs/05-roadmap-status.md`** — it has the
complete feature matrix (what works, what doesn't) and the phase breakdown.

---

## Known Issues

| Issue | Status |
|-------|--------|
| PIE relocation on x86_64 (R_X86_64_32) | ✅ Fixed — `RelocMode::PIC` in `pipeline.rs` |
| `error_test.kl` exits non-zero (by design, `Option` return) | ✅ Fixed — CI checks all, runs subset |
| Release: existing tag blocks re-create | ✅ Fixed — `gh release delete` before create |
| Node.js 20 deprecation (actions/checkout@v4) | ⚠️ Warning only, non-fatal |
| Install: `set -o pipefail` fatal in dash (Ubuntu `sh`) | ✅ Fixed — conditional pipefail only in bash |
| Linker: runtime lib path mismatch (`lib/klc/` vs `lib/kl/`) | ✅ Fixed — `find_runtime_lib()` looks in `lib/kl/` |

## Release v0.2.1

https://github.com/IT-KYNERA/KYLE/releases/tag/v0.2.1

| Asset | Platform |
|-------|----------|
| `kl-v0.2.1-macos-arm64.tar.gz` | macOS Apple Silicon |
| `kl-0.2.1.vsix` | VS Code extension (universal) |

## Development Commands

```bash
cargo build --workspace                    # Build all crates
cargo run --bin kl -- run <file.kl>        # Compile and run a Kyle file
cargo run --bin kl -- build <file.kl>      # Compile to native binary
cargo run --bin kl -- check <file.kl>      # Type-check only (fast)
cargo run --bin kl -- new <project>        # Create new project
cargo test -p klc_core -p klc_frontend -p klc_semantic -p klc_mir -p klc_runtime -p klc_tools  # Run tests
```

**Verify before any change:** build + 101 tests must pass. CI runs on push.

---

## Key Design Decisions (frozen — do not change)

| Decision | Choice |
|----------|--------|
| Blocks | Indentation (4 spaces) |
| Semicolons | None — newline terminates statements |
| Variables | Immutable by default, `mut` keyword for mutable |
| Constants | UPPERCASE convention (no `mut`) |
| Instance reference | `this` (NOT `self`) |
| Optionals | `Option<T>` syntax (NOT `T?`) |
| Error propagation | `?` operator (for errors only) |
| Error type | `T!` return type syntax |
| Exceptions | None — errors are values |
| `let`/`var` | None — `mut` keyword directly |
| `{` `}` for blocks | None — indentation |
| Visibility | Convention: `_` protected, `__` private |
| String encoding | UTF-8 |
| Integer overflow | Panic in debug, wrapping in release |
| Entry point | `fn main(args: [str]) -> i32` in `src/main.kl` |
| Memory | RAII + Compiler-Inferred Ownership (no GC, no manual free) |
| Compiler language | Pure Rust (LLVM via inkwell) — will NOT change |
| CLI binary | `kl` (primary) + `klc` (legacy alias) |

---

## Repository Structure

```
kl/
├── AGENTS.md              ← this file (AI agent context)
├── README.md              ← public-facing project README
├── LICENSE                ← MIT
├── Cargo.toml             ← Rust workspace root
├── kl.toml                ← Kyle project manifest
├── .github/workflows/ci.yml ← CI: build + tests + examples
│
├── crates/                ← 9 Rust crates (the compiler)
│   ├── klc_core/          ← AST, Span, Types, Diagnostics
│   ├── klc_frontend/      ← Lexer, Parser
│   ├── klc_semantic/      ← Type checker, symbol resolver
│   ├── klc_mir/           ← MIR definition, lowering, optimization
│   ├── klc_backend/       ← LLVM codegen (inkwell), linker
│   ├── klc_driver/        ← Pipeline orchestration
│   ├── klc_cli/           ← CLI binary (kl + klc)
│   ├── klc_runtime/       ← RAII runtime, async, channels, I/O
│   └── klc_tools/         ← LSP, formatter, package manager
│
├── std/                   ← Standard library (8 .kl modules)
├── docs/                  ← 6 specification documents
├── examples/              ← 51+ example .kl programs
└── vscode-kl/             ← VS Code extension (.vsix)
```

For the full crate-by-crate breakdown and runtime internals, see `docs/04-compiler-architecture.md`.

---

## LLVM Configuration

LLVM 18.1 required. Install on:

**Linux (Ubuntu ARM):**
```bash
sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev
```

**macOS (Apple Silicon):**
```bash
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)
```

---

## What NOT to Do

1. **Do not add new syntax features without checking `docs/01-language-reference.md`** — the syntax is frozen for Phase 6. New features go in Phase 9+.
2. **Do not write C/C++ code** — the compiler and runtime are pure Rust. Phase 9 FFI will let Kyle programs call C libraries, but the compiler itself stays Rust.
3. **Do not change the `let`/`var`/`self`/`{}` decisions** — they are frozen. If a user requests one, explain the design rationale instead.
4. **Do not skip the test run** — 101 tests must pass before any change is considered complete.
5. **Do not create documentation outside `docs/`** — all docs live in the 6-file structure. README.md is the only exception (root-level, public-facing).

---

## Glossary

| Term | Meaning |
|------|---------|
| AST | Abstract Syntax Tree — code as a tree (Parser output) |
| MIR | Mid-level IR — Kyle's own intermediate representation (between AST and LLVM) |
| IR | Intermediate Representation — any rep between source and machine code |
| LLVM | Low Level Virtual Machine — framework that generates optimized machine code |
| LSP | Language Server Protocol — editor ↔ compiler communication |
| ABI | Application Binary Interface — how functions are called, data is laid out |
| RAII | Resource Acquisition Is Initialization — memory freed at scope exit (no GC) |
| FFI | Foreign Function Interface — calling C library functions from Kyle (Phase 9) |
| CLI | Command Line Interface — the `kl` binary |
| DOT-completion | LSP feature: typing `obj.` shows fields/methods (`struct.field`, `obj.method`) |
| LSP rename | F2 on symbol → renames all references via `textDocument/rename` |
| LSP formatting | Shift+Option+F → formats code via `textDocument/formatting` |