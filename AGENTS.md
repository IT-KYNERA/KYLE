# Kyle — AI Agent Context

> **Read this first.** Single entry-point for AI agents working on the Kyle codebase.
> Tells you what Kyle is, current state, project structure, and where to find details.

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

## Current Status: Fases 1–17 COMPLETED

| Area | Status | Details |
|------|--------|---------|
| **Language syntax** | ✅ Complete | Generics, ranges, match, operator overloading, is, ptr, for-else, static fn, `**` |
| **Borrow checker** | ✅ Complete | `&T` mutable, `^T` move, field mutability, call-site borrowing |
| **SSA optimization** | ✅ Complete | mem2reg, phi nodes, GVN, dominator fix |
| **LLVM IR quality** | ✅ Complete | nsw flags, TBAA, inbounds, noalias, readonly, noundef, !range |
| **Optimization** | ✅ Complete | O3 pipeline, constant folding, alloca elimination |
| **Tooling** | ✅ Complete | LSP, VS Code extension, formatter, package manager, test framework |
| **Package manager** | ✅ Complete | Registry, cache, lock, publish, login, dependency resolution |
| **FFI (extern fn, @link, ptr)** | ✅ **Phase 0 done** | Pure Kyle FFI to C libraries — no Rust needed for packages |
| **kyc_platform** | ✅ **Phase 1 started** | FS (file I/O), Time (now, sleep) in Rust crate |

## What's Next

| Priority | Task | Est. |
|----------|------|------|
| 🔜 **Packages** | Build `http`, `json`, `sqlite`, `postgres`, `websocket`, `crypto` in Kyle | Weeks |
| 🔜 **kyc_platform** | Networking module, macOS adapter, move I/O from runtime | Days |
| 📅 **Phase 18** | Zero-Cost Abstractions (escape analysis, SSO, inlining, monomorphization) | Months |
| 📅 **Async V3** | State machine, work-stealing, non-blocking I/O | Months |
| 📅 **Windowing → UI** | Windowing system, graphics, scene graph, widget library | Long-term |
| 📅 **KyleOS** | Operating system based on Linux kernel + Kyle platform | Long-term |

See [ROADMAP.md](ROADMAP.md) for complete implementation order.

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
│   ├── http/             # HTTP client via libcurl FFI (GET, POST, PUT, DELETE)
│   ├── json/             # JSON parse + stringify
│   └── sqlite/           # SQLite database bindings
│
├── std/                  # Standard library (.ky files)
│   ├── core.ky           # Option<T> enum (unwrap_or, is_some, is_none)
│   ├── io.ky             # read_file, write_file
│   ├── str.ky            # starts_with, ends_with, capitalize, repeat_str
│   ├── collections.ky    # list_sum, list_product, list_max, list_min, list_range
│   ├── math.ky           # pow, sqrt, gcd, clamp, absolute
│   ├── json.ky           # parse, stringify
│   ├── time.ky           # timestamp, sleep_ms, seconds_since
│   └── testing.ky        # assert, assert_eq, assert_str, assert_ne
│
├── docs/                 # Documentation (72 files, reorganized)
├── vscode-ky/            # VS Code extension (LSP client, debugger, syntax highlighting)
├── examples/             # Example .ky project
├── tests/                # End-to-end type-check test files
└── ROADMAP.md            # Feature roadmap with phases and implementation order
```

---

## Documentation Map (docs/)

The docs are organized by **knowledge layer**, not by compiler component.

| Section | Files | Content |
|---------|:-----:|---------|
| [01-overview/](docs/01-overview/README.md) | 5 | Vision, philosophy, principles, layered architecture |
| [02-guide/](docs/02-guide/README.md) | 7 | Tutorial: install, first program, testing, debugging, patterns, performance, CI/CD |
| [03-language-reference/](docs/03-language-reference/README.md) | **15** | **Formal language specification** (read for ANY syntax question) |
| [04-platform/](docs/04-platform/README.md) | 17 | Compiler CLI, build system, standard library (8 modules), tools (PM, formatter, LSP, VS Code, editors), targets (WASM) |
| [05-packages/](docs/05-packages/README.md) | 4 | Official package specs: HTTP, JSON, SQLite, PostgreSQL |
| [06-reference/](docs/06-reference/README.md) | 4 | Quick lookup: keywords, operators, flags, CLI commands |
| [07-engineering/](docs/07-engineering/README.md) | 5 | Compiler architecture, SSA, optimization pipeline, codegen |
| [08-design/](docs/08-design/README.md) | 3 | ADRs, RFCs (architecture decisions, move semantics) |
| [09-project/](docs/09-project/README.md) | 1 | Changelog |
| [10-history/](docs/10-history/README.md) | 1 | Migration guide |

### Quick reference links

| You need... | Go to |
|-------------|-------|
| Syntax of `if`/`while`/`for`/`match` | `docs/03-language-reference/statements.md` |
| Type system (`T?`, `T!`, `&T`, `^T`) | `docs/03-language-reference/types.md` |
| Functions, methods, closures | `docs/03-language-reference/functions.md` |
| Variables, constants, mutability | `docs/03-language-reference/variables.md` |
| Ownership, borrowing, move semantics | `docs/03-language-reference/ownership.md` |
| Error handling with `T!` and `?` | `docs/03-language-reference/error-handling.md` |
| Pattern matching | `docs/03-language-reference/pattern-matching.md` |
| FFI: `extern fn`, `@link`, `ptr` | `docs/03-language-reference/ffi.md` |
| Quick keyword/operator lookup | `docs/06-reference/README.md` |
| Compiler CLI flags | `docs/06-reference/cli-commands.md` + `docs/06-reference/compiler-flags.md` |
| How to test | `docs/02-guide/testing.md` |
| Standard library functions | `docs/04-platform/standard-library/overview.md` |
| Package manager usage | `docs/04-platform/tools/package-manager.md` |
| VS Code extension | `docs/04-platform/tools/vscode.md` |

---

## Language Syntax Reference (Quick)

### Variables

| Form | Syntax | Example |
|------|--------|---------|
| Immutable | `name = expr` | `count = 42` |
| Mutable | `name: &T = expr` | `count: &i32 = 0` |
| Mutable (sugar) | `name = &expr` | `count = &0` |
| Constant | `NAME := expr` | `MAX_SIZE := 1024` |

### Functions

| Form | Syntax | Example |
|------|--------|---------|
| Regular | `fn name(params) ret_type:` | `fn add(a: i32, b: i32) i32:\n    a + b` |
| Method | `fn name(params) ret_type:` inside class | `fn len() f64:\n    sqrt(...)` |
| Static | `static fn name(params) ret_type:` | `static fn square(x: i32) i32:\n    x * x` |
| Extern (FFI) | `extern fn name(params) ret_type` | `extern fn curl_easy_init() ptr` |
| Constructor | `fn ClassName(params):` | `fn Config(name: str, port: i32 = 8080):\n    this.name = name` |

**No `this`/`self` parameter** in method signatures — it's implicit.

### Parameters

| Mode | Syntax | Semantics |
|------|--------|-----------|
| Borrow (default) | `s: str` | Immutable borrow |
| Mutable borrow | `s: &str` | Mutable borrow |
| Move | `^s: str` | Ownership transfer |

### Imports

```ky
from math import square
from std import io, json
from packages.http.src.lib import get
```

### Visibility

| Prefix | Scope |
|--------|-------|
| `name` | Public |
| `_name` | Protected (same package / subclasses) |
| `__name` | Private (same module) |

### Types

| Type | Description |
|------|-------------|
| `i8`, `i16`, `i32`, `i64` | Signed integers |
| `f32`, `f64` | Floating point |
| `bool` | Boolean |
| `str` | Heap-allocated immutable string |
| `char` | Unicode code point (integer value) |
| `ptr` | Raw pointer (FFI, unsafe) |
| `T?` | Optional (`Option<T>`) |
| `T!` | Fallible (`Result<T, Error>`) |
| `&T` | Mutable type |
| `^T` | Move/ownership type |
| `[T]` | List of T |

### Comments

```ky
# Line comment
## Doc comment (before declarations)
```

### Classes

```ky
# Lightweight struct (no inheritance)
final class Vec2:
    x: i32
    y: i32

# Full class with inheritance
class Animal:
    name: str
    fn speak():
        println("...")
class Dog :: Animal:
    fn speak():
        println("woof")

# Abstract class
abstract class Shape:
    fn area() f64

# Constructor
class Config:
    name: str
    port: i32
    fn Config(name: str, port: i32 = 8080):
        this.name = name
        this.port = port
```

### Operator Overloading

```ky
final class Vec2:
    fn op_+(other: Vec2) Vec2:
        Vec2 { x: this.x + other.x, y: this.y + other.y }
a + b  # calls op_+
```

Supported operators: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`

### FFI

```ky
@link "curl"
extern fn curl_easy_init() ptr
extern fn curl_easy_setopt(handle: ptr, option: i32, value: ptr) i32
```

---

## Packages (100% Kyle)

| Package | Description | Location |
|---------|-------------|----------|
| `http` | HTTP client via libcurl FFI | `packages/http/` |
| `json` | JSON parse + stringify | `packages/json/` |
| `sqlite` | SQLite database bindings | `packages/sqlite/` |

All packages are written in pure Kyle using `extern fn` + `@link`. No Rust involved.

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
ky fmt --check             # Check formatting (CI mode)

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
ky add <dep>[@<ver>]      # Add dependency
ky publish                # Publish package
ky lsp                    # Start LSP server (for editors)
```

---

## LLVM Configuration

LLVM 18.1 required.

**macOS (Apple Silicon):** `brew install llvm@18 && export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)`
**Linux (Ubuntu ARM):** `sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev`

---

## What NOT to Do

1. **Do not add syntax features** without checking `docs/03-language-reference/` first
2. **Do not write C/C++ code** — the compiler and runtime are pure Rust
3. **Do not reintroduce `mut`, `let`, `var`, `const`** — use `&T` or `:=`
4. **Do not reintroduce `Option<T>` as public syntax** — use `T?`
5. **Do not reintroduce `struct`** — use `final class`
6. **Do not reintroduce `::=`** — constants use `:=`
7. **Do not use `self`** — use `this` (instance reference)
8. **Do not skip tests** — CI must pass before any merge

---

*Version: v0.5.0 · Last updated: 2026-07-03 — Fases 1-17 completadas, Phase 0 (FFI) ✅, packages iniciados*
