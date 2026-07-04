# Kyle — AI Agent Context

> **Read this first.** It is the single entry-point for AI agents working on the Kyle codebase.
> It tells you what Kyle is, where we are, how to test, and **where to find documentation**.

---

## What is Kyle?

A compiled, statically-typed language for backend systems, CLI tools, and full-stack development.
Written in **Rust** (compiler + runtime), compiles via **LLVM 18**.

- Python readability (indentation blocks, no semicolons, no `self`)
- Rust type safety (strong typing, generics, pattern matching, borrow checker)
- Go simplicity (fast compilation, built-in tooling, package manager)
- C performance (native code via LLVM O3 pipeline)

**The compiler and runtime are written in Rust.** Packages (`http`, `json`, `sqlite`) are written in **100% Kyle** using `extern fn` + `@link` for FFI to C libraries.

---

## Current Status

| Area | Status |
|------|--------|
| **Compiler (Fases 1-17)** | ✅ **Complete** — Lexer, parser, semantic, MIR, SSA, LLVM codegen, O3 pipeline |
| **Syntax** | ✅ **Complete** — Generics, ranges, match, op overloading, is, ptr, for-else, static fn, ** |
| **Borrow checker** | ✅ **Complete** — `&T` mutable, `^T` move, field mutability |
| **Tooling** | ✅ **Complete** — LSP, VS Code ext, formatter, test framework, package manager |
| **FFI (extern fn, @link, ptr)** | ✅ **Phase 0 done** — Pure Kyle FFI to C libraries |
| **Runtime in Kyle** | 🔶 **Phase A in progress** — 18/88 functions rewritten in pure Kyle |
| **kyc_platform** | 🔶 **Phase 1 started** — FS (file I/O), Time in Rust crate |

See [ROADMAP.md](ROADMAP.md) for full implementation plan.

---

## CRITICAL — When Writing Kyle Code

**When you get a syntax error or unexpected behavior:**
1. **STOP trying random syntax**
2. **Check the docs** (see Documentation Map below)
3. The docs are the **canonical source of truth** for all syntax

**Key files to consult:**
- `docs/03-language-reference/` — **Read this for ANY syntax question** (15 focused files)
- `docs/06-reference/` — Quick lookup: keywords, operators, flags, CLI commands
- `docs/04-platform/standard-library/` — Available built-in functions

**This file (AGENTS.md) does NOT contain syntax reference.**
Do not guess Kyle syntax — always check the docs.

---

## Project Structure

```
ky/
├── crates/               # Rust crates (compiler + runtime + tools)
│   ├── kyc_core/         # Foundation: AST types, diagnostics
│   ├── kyc_frontend/     # Lexer + parser
│   ├── kyc_hir/          # HIR desugaring
│   ├── kyc_semantic/     # Type checker, scope resolver, borrow analysis
│   ├── kyc_mir/          # MIR lowering, SSA construction, optimizations
│   ├── kyc_backend/      # LLVM codegen (via inkwell), linker
│   ├── kyc_driver/       # Compilation pipeline orchestration
│   ├── kyc_cli/          # CLI binary (`ky`)
│   ├── kyc_runtime/      # Runtime static library (memory, strings, lists, dicts, I/O, threads)
│   ├── kyc_tools/        # LSP server, formatter, package manager
│   └── kyc_platform/     # 🔜 Platform API: FS, networking, time (in progress)
│
├── packages/             # Official Kyle packages (100% Kyle)
│   ├── http/             # HTTP client via libcurl FFI
│   ├── json/             # JSON parse + stringify
│   └── sqlite/           # SQLite database bindings
│
├── std/                  # Standard library (.ky files, 8 modules)
├── docs/                 # Documentation (72 files, reorganized)
├── vscode-ky/            # VS Code extension
├── examples/             # Example .ky project
├── tests/                # End-to-end type-check test files
└── ROADMAP.md            # Feature roadmap with phases and implementation order
```

---

## Documentation Map

| Section | Files | Content |
|---------|:-----:|---------|
| [01-overview/](docs/01-overview/README.md) | 5 | Vision, philosophy, principles, layered architecture |
| [02-guide/](docs/02-guide/README.md) | 7 | Tutorial: install, first program, testing, debugging, patterns, performance, CI/CD |
| [03-language-reference/](docs/03-language-reference/README.md) | **15** | **Formal language specification** (read for ANY syntax question) |
| [04-platform/](docs/04-platform/README.md) | 17 | Compiler CLI, build system, standard library (8 modules), tools, targets (WASM) |
| [05-packages/](docs/05-packages/README.md) | 4 | Official package specs: HTTP, JSON, SQLite, PostgreSQL |
| [06-reference/](docs/06-reference/README.md) | 4 | Quick lookup: keywords, operators, flags, CLI commands |
| [07-engineering/](docs/07-engineering/README.md) | 5 | Compiler architecture, SSA, optimization pipeline, codegen |
| [08-design/](docs/08-design/README.md) | 3 | ADRs, RFCs (architecture decisions, move semantics) |
| [09-project/](docs/09-project/README.md) | 1 | Changelog |
| [10-history/](docs/10-history/README.md) | 1 | Migration guide |

### Quick reference links

| You need... | Go to |
|-------------|-------|
| **ANY syntax question** | `docs/03-language-reference/` (15 focused files) |
| Quick keyword/operator lookup | `docs/06-reference/` |
| Compiler CLI flags | `docs/06-reference/cli-commands.md` + `docs/06-reference/compiler-flags.md` |
| How to test | `docs/02-guide/testing.md` |
| Standard library functions | `docs/04-platform/standard-library/overview.md` |
| Package manager usage | `docs/05-packages/registry.md` |
| VS Code extension | `docs/04-platform/tools/vscode.md` |
| Performance tips | `docs/02-guide/performance.md` |
| Common patterns | `docs/02-guide/patterns.md` |
| FFI (extern fn, @link, ptr) | `docs/03-language-reference/ffi.md` |

---

## Packages (100% Kyle, no Rust)

| Package | Description | Location |
|---------|-------------|----------|
| `http` | HTTP client via libcurl FFI | `packages/http/` |
| `json` | JSON parse + stringify | `packages/json/` |
| `sqlite` | SQLite database bindings | `packages/sqlite/` |

All packages use `extern fn` + `@link` for FFI. See `docs/03-language-reference/ffi.md`.

---

## Testing

```bash
# Rust unit tests (all crates)
cargo test --workspace

# Build (debug)
cargo build --workspace

# Build release
cargo build --release --bin ky

# Kyle checks
ky check <file.ky>       # Type-check only
ky build <file.ky>        # Compile to binary
ky run <file.ky>           # Compile and run

# Kyle tests
ky test                    # Run #[test] functions in tests/

# Format
ky fmt src/                # Format source directory

# Package tests
cd packages/<name> && ky check src/lib.ky
```

---

## Development Commands

```bash
ky build <file.ky>        # Compile to binary
ky run <file.ky>          # Compile and run
ky check <file.ky>        # Type-check only (fast)
ky fmt [file/dir]         # Format source
ky test                   # Run test suite
ky new <project>          # Create new project
ky add <dep>[@<ver>]      # Add dependency (uses GitHub Pages registry by default)
ky remove <dep>           # Remove dependency (cleans std/ + ky.toml)
ky install                # Install all dependencies from ky.lock
ky publish                # Publish package (creates tarball in registry/)
ky lsp                    # Start LSP server (for editors)
```

---

## LLVM Configuration

LLVM 18.1 required.

**macOS (Apple Silicon):** `brew install llvm@18 && export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)`
**Linux (Ubuntu ARM):** `sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev`

---

## What NOT to Do

1. **Do not guess syntax** — check `docs/03-language-reference/` first
2. **Do not add new syntax features** without checking the docs
3. **Do not reintroduce `mut`, `let`, `var`, `const`** — use `&T` or `:=`
4. **Do not reintroduce `Option<T>` as public syntax** — use `T?`
5. **Do not use `struct`** — use `final class`
6. **Do not write C/C++ code** — the compiler and runtime are pure Rust
7. **Do not skip tests** — CI must pass before any merge

---

*Version: v0.5.0 · Last updated: 2026-07-03 — Fases 1-17 completadas, Phase 0 (FFI) ✅, packages iniciados, Phase A (runtime en Kyle) en progreso*
