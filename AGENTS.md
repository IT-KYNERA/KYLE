# Kyle — AI Agent Context

> **Read this first.** It is the single entry-point for AI agents working on
> the Kyle codebase. It tells you what Kyle is, where we are, and where to
> look for detailed information.

---

## What is Kyle?

A compiled, statically-typed language for backend systems and CLI tools.
Written in **Rust**, compiled via **LLVM 18**.

- Python readability (indentation blocks, no semicolons, no `self`)
- Rust type safety (strong typing, generics, pattern matching)
- Go simplicity (fast compilation, built-in tooling)
- C performance (native code via LLVM)

**Implementation language is pure Rust — this will NOT change.**

---

## Current Phase: Evolution to v1.0

The language is undergoing a major evolution. The complete plan is at
[`docs/05-roadmap-status.md`](docs/05-roadmap-status.md). Current focus is
**Fase 17: Optimization Pipeline** (cerrar gap de rendimiento con Rust).

| Phase | Focus | Status |
|---|---|---|
| **1–2** | Documentation + Spec updates | ✅ Done |
| **3** | Lexer (`:=`, `&T`, `T?`, `final class`) | ✅ Done |
| **4** | **Parser** (destructuring, `&T`, `^T`) | ✅ Done |
| **5** | **HIR — High-Level IR** | ✅ Done |
| **6** | **Semantic** | ✅ Done |
| **7** | **Borrow Semantics** | ✅ Done (refactorizado) |
| **14** | **References & Borrow Checker** | ✅ **COMPLETADO** (&T codegen, mutable fields, field defaults, borrow whitelist eliminada, region inference no aplica) |
| 15–18 | SSA, LLVM IR Quality, Optimization Pipeline, Zero-Cost | 📅 |

See [`docs/05-roadmap-status.md`](docs/05-roadmap-status.md) for full details.

---

## Key Syntax Decisions (NEW — Frozen)

### Variables — No `mut`, No `let`, No `const` Keywords

| Form | Syntax | Description |
|---|---|---|
| Immutable | `name = value` | Declaration + immutable binding |
| Mutable | `name: &T = value` or `name = &value` | `&` in type/value = mutable |
| Constant | `NAME := value` | Compile-time constant (replaces `::=`) |

### Types — Unification

- `T?` is the only public optional syntax (sugar for `Option<T>` internally)
- `T!` stays as error-returning type (sugar for `Result<T, Error>`)
- `&T` is the mutable type (for mutable variables, mutable borrow params, mutable fields)
- `^T` is the move/ownership type (for ownership-transfer parameters)
- `ptr` is the raw pointer type (for FFI/unsafe)
- `final class` replaces `struct` (lightweight, no inheritance)
- `abstract class` replaces `abs class`
- `struct` is a temporary alias, will be removed

### Other Frozen Decisions

| Decision | Choice |
|----------|--------|
| Blocks | Indentation (4 spaces) |
| Statement terminator | Newline (no semicolons) |
| Instance reference | `this` (NOT `self`) |
| Visibility | Convention: `_` = protected, `__` = private, none = public |
| Error return | `T!` syntax |
| Error propagation | `?` operator |
| Exceptions | None — errors are values |
| Entry point | `fn main(args: [str]) -> i32` in `src/main.ky` |
| Memory | Borrow-by-default, ownership via `^`, Copy types + Clone |
| Compiler | Pure Rust + LLVM 18 via `inkwell` |

---

## Documentation (read before any task)

| File | When to Read |
|------|-------------|
| [`docs/00-vision.md`](docs/00-vision.md) | Philosophy, design principles |
| [`docs/01-language-reference.md`](docs/01-language-reference.md) | **Every task.** Complete syntax with ✅/🔶/❌ |
| [`docs/02-types-errors-memory.md`](docs/02-types-errors-memory.md) | Type system, memory model, error handling |
| [`docs/03-modules-packages-tooling.md`](docs/03-modules-packages-tooling.md) | Modules, packages, CLI, VS Code |
| [`docs/04-compiler-architecture.md`](docs/04-compiler-architecture.md) | 9-crate pipeline, runtime internals |
| [`docs/05-roadmap-status.md`](docs/05-roadmap-status.md) | Feature matrix, phase details, release checklist |
| [`docs/05-roadmap-status.md`](docs/05-roadmap-status.md) | **Master roadmap** (phases, priorities, v1.0 checklist) |

---

## Test Suite (run before any change)

```bash
# Rust unit tests (all crates)
cargo test --workspace

# End-to-end syntax tests (type-check only, all .ky files in tests/)
ky check tests/*.ky

# Build all crates
cargo build --workspace
```

---

## Development Commands

```bash
ky run <file.ky>          # Compile and run
ky build <file.ky>        # Compile to native binary
ky check <file.ky>        # Type-check only (fast)
ky new <project>          # Create new project
ky test <project>         # Type-check all tests/ files
ky fmt src/               # Format project
```

---

## LLVM Configuration

LLVM 18.1 required.

**Linux (Ubuntu ARM):** `sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev`
**macOS (Apple Silicon):** `brew install llvm@18 && export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)`

---

## What NOT to Do

1. **Do not add new syntax features** without checking `docs/01-language-reference.md`.
2. **Do not write C/C++ code** — the compiler and runtime are pure Rust.
3. **Do not reintroduce `mut`, `let`, `var`, `const` keywords** for variables.
4. **Do not reintroduce `Option<T>` as a public syntax** — use `T?`.
5. **Do not reintroduce `struct`** as a separate keyword (use `final class`).
6. **Do not reintroduce `::=`** — constants use `:=`.
7. **Do not skip tests** — CI must pass before any merge.

---

*Version: v0.5.0 · Last updated: 2026-07-02*
