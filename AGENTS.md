# KL Programming Language — Project Context for AI Agents

## Overview

KL (Kynera Language) is a compiled, statically-typed programming language that combines:
- Python readability (indentation-based blocks, clean syntax)
- Rust type safety (strong typing, generics, pattern matching)
- Go simplicity (fast compilation, built-in tooling)
- LLVM performance (native compilation via LLVM backend)

## Project Structure

```
/Users/kynera/HCA/KYNERA/kl/
├── AGENTS.md                 ← this file — context for AI agents
├── Cargo.toml                ← Rust workspace root
├── .cargo/config.toml        ← LLVM path config (Homebrew LLVM)
├── kl.toml                   ← KL project manifest
├── .gitignore
│
├── crates/                   ← Rust compiler crates
│   ├── klc_core/             → Core types, AST, Span, diagnostics
│   ├── klc_frontend/         → Lexer, Parser (KL source → AST)
│   ├── klc_semantic/         → Type checker, symbol resolver
│   ├── klc_mir/              → Mid-level IR, optimization
│   ├── klc_backend/          → LLVM codegen, linker
│   ├── klc_driver/           → Pipeline orchestration
│   ├── klc_cli/              → CLI binary (klc)
│   ├── klc_runtime/          → GC, async executor, panic handler
│   └── klc_tools/            → LSP, formatter, completion
│
├── runtime/                  → KL runtime (Rust source)
│   ├── memory/
│   ├── async/
│   ├── collections/
│   └── io/
│
├── std/                      → Standard library (KL source)
│   ├── core/  math/  json/  io/  net/
│   ├── time/  filesystem/  collections/  crypto/
│   └── async/  testing/
│
├── docs/                     ← Language specification (16 docs)
├── examples/                 → Example KL programs
├── tests/                    → Compiler test suite
├── benchmarks/               → Performance benchmarks
└── tools/                    → Developer tooling scripts
```

## Key Design Decisions

| Decision | Choice |
|----------|--------|
| Blocks | Indentation-based (4 spaces) |
| Semicolons | None — newline is statement terminator |
| Variables | Mutable by default (lowercase) |
| Constants | UPPERCASE naming, immutable, compile-time |
| Instance ref | `this` (not `self`) |
| Optional | `Option<T>` (not `T?`) |
| Error prop | `?` (exclusively for error propagation) |
| Abstract | `abs class` / `abs fn` |
| Visibility | Naming convention (`_` protected, `__` private) |
| Exceptions | None — explicit errors with `!` and `match` |
| `let`/`var`/`mut` | None — variables are mutable by default |
| `{}` for blocks | None — indentation-based |
| Export | None — naming-based visibility only |
| String encoding | UTF-8 |
| Integer overflow | Panic in debug, wrapping in release |
| Entry point | `fn main(args: [str]) -> i32` in `src/main.kl` |

## LLVM Configuration

**CRITICAL**: This project uses Homebrew LLVM, NOT Apple's default LLVM.

```
LLVM path:     /opt/homebrew/opt/llvm
LLVM version:  22.1.7
Configuration: .cargo/config.toml (LLVM_SYS_220_PREFIX)
```

All LLVM commands must use the Homebrew path:
```bash
/opt/homebrew/opt/llvm/bin/clang --version
/opt/homebrew/opt/llvm/bin/llvm-config --version
```

NOT:
```bash
clang --version          # ← this is Apple's LLVM, WRONG
/usr/bin/clang           # ← also Apple's, WRONG
```

## Compiler Pipeline

```
KL Source (.kl)
    ↓
[Lexer]          → Token stream
    ↓
[Parser]         → AST (recursive descent)
    ↓
[Semantic]       → Typed AST (symbols, types, contracts)
    ↓
[MIR]            → Mid-level IR (optimized)
    ↓
[Backend]        → LLVM IR → Object file
    ↓
[Linker]         → Native binary
```

## Current Phase

**Phase 1: Compiler Frontend** — Lexer + Parser implementation.
The language specification is complete (Phase 0). All 16 docs in `docs/`
are final and frozen.

## How to Build

```bash
# Build the compiler
cargo build --workspace

# Run the CLI
cargo run --bin klc -- build main.kl

# Run tests
cargo test --workspace
```

## Documentation References

| File | Content |
|------|---------|
| `docs/00-vision.md` | Language vision and philosophy |
| `docs/01-language-specification.md` | Complete language syntax |
| `docs/02-formal-grammar.md` | Formal EBNF grammar |
| `docs/03-ast-specification.md` | AST node structure |
| `docs/04-type-system.md` | Type system details |
| `docs/05-error-system.md` | Error handling design |
| `docs/06-module-system.md` | Module and visibility system |
| `docs/07-standard-library.md` | Standard library API |
| `docs/08-async-runtime.md` | Async runtime design |
| `docs/09-memory-model.md` | Memory management |
| `docs/10-compiler-architecture.md` | Compiler architecture |
| `docs/11-project-architecture.md` | Project structure |
| `docs/12-package-manager.md` | Package manager design |
| `docs/13-roadmap.md` | Development roadmap |
| `docs/14-error-catalog.md` | Error codes and diagnostics |
| `docs/15-abi-specification.md` | ABI and FFI specification |
