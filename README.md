<div align="center">

<img src="vscode-kl/icons/kl_128.png" width="160" alt="Kyle logo">

# Kyle

**A backend programming language that doesn't make you choose.**

Readable like Python · Typed like Rust · Simple like Go · Fast like C

[![License: MIT](https://img.shields.io/badge/license-MIT-6C3FC5?style=flat-square)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-101%20passing-6C3FC5?style=flat-square)](#testing)
[![Platform](https://img.shields.io/badge/platform-macOS%20ARM%20%7C%20Linux%20ARM-6C3FC5?style=flat-square)](#install)
[![LLVM](https://img.shields.io/badge/LLVM-18-6C3FC5?style=flat-square)](#building-from-source)

</div>

---

Kyle is a compiled, statically-typed programming language for building backend
services, CLI tools, and systems software. It compiles to native machine code via
LLVM — no interpreter, no runtime overhead, no garbage collector.

```kl
import http
import db

fn main(args: [str]) -> i32:
    app = http.Server()
    app.get("/users", get_users)
    app.listen(8080)
    return 0

fn get_users(req: http.Request, res: http.Response):
    pool = db.sqlite.open("app.db")
    users = pool.query("SELECT * FROM users")?
    res.json(users)
```

---

## Install

One command for all supported platforms — the script auto-detects your OS and
architecture:

```bash
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh | sh
```

**Supported platforms:**

| Platform | Status |
|----------|--------|
| macOS ARM (Apple Silicon) | ✅ |
| Linux ARM (aarch64) | ✅ |
| macOS Intel | 📅 Future |
| Linux x64 | 📅 Future |
| Windows x64 | 📅 Future |

### Verify

```bash
kl --version
```

No dependencies required. The installer downloads a pre-compiled native binary.

### VS Code Extension

Install the Kyle VS Code extension for syntax highlighting, autocompletion,
LSP integration, and semantic coloring:

**Option 1 — From the terminal:**

```bash
code --install-extension vscode-kl/vscode-kl-0.2.1.vsix
```

**Option 2 — Via the command palette:**

1. Open VS Code
2. Press `Cmd+Shift+P` (macOS) / `Ctrl+Shift+P` (Linux)
3. Type **Extensions: Install from VSIX...** and select the file
   `vscode-kl/vscode-kl-0.2.1.vsix` from the Kyle repository

**Option 3 — From the GitHub release:**

Download `kl-0.2.1.vsix` from the
[latest release](https://github.com/IT-KYNERA/KYLE/releases/latest) and install via
`code --install-extension kl-0.2.1.vsix`.

### Building from Source

> You only need this if you want to contribute to the compiler itself.
> For everyday use, use the one-line installer above.

**Prerequisites:**

- **Rust toolchain** (stable, 1.70+) — install via [`rustup.rs`](https://rustup.rs)
- **LLVM 18.1** — see platform-specific instructions below

**macOS (Apple Silicon):**

```bash
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)
git clone https://github.com/IT-KYNERA/KYLE
cd kl
cargo build --workspace
```

**Linux (Ubuntu ARM):**

```bash
sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev
git clone https://github.com/IT-KYNERA/KYLE
cd kl
cargo build --workspace
```

The compiled binary is at `target/debug/kl`.

---

## Quick Start

```bash
# Create a new project
kl new myapp
cd myapp

# Run it
kl run
# → Hello from myapp v0.1.0!
# → args: 0

# Build a release binary
kl build --release
# → target/release/myapp

# Type-check without compiling
kl check src/main.kl
```

### Project Structure

`kl new` creates a professional project layout:

```
myapp/
├── src/
│   └── main.kl           ← entry point (fn main)
├── tests/
│   └── test_main.kl      ← test stub
├── kl.toml                ← project manifest
└── .gitignore             ← target/, build artifacts
```

---

## Hello, World

```kl
fn main(args: [str]) -> i32:
    println("Hello, World!")
    return 0
```

```console
$ kl run hello.kl
Hello, World!
```

---

## Why Kyle?

| Problem | Kyle's Solution |
|---------|-----------------|
| Python is too slow | Compiled to native code via LLVM |
| Rust is too complex | No borrow checker, no lifetimes, no `self` |
| Go is too limited | Full generics, enums, pattern matching |
| TypeScript isn't native | True native compilation, zero runtime |
| C has no safety | Strong typing, RAII, no nulls, no exceptions |

Kyle gives you the **readability of Python**, the **type safety of Rust**, the
**simplicity of Go**, and the **performance of C** — all in one language.

---

## Language Tour

### Variables & Types

```kl
name = "Kyle"          # immutable by default
mut count = 0          # mutable with `mut`
PI = 3.14159           # constants (UPPERCASE)

x: i32 = 42            # explicit type annotation
items: [str] = ["a"]   # list of strings
```

### Functions & Closures

```kl
fn add(a: i32, b: i32) -> i32:
    return a + b

# Generic function
fn first<T>(items: [T]) -> Option<T>:
    if len(items) > 0:
        return Some(items[0])
    return None

# Closure
double = (x: i32) => x * 2
```

### Structs & Classes

```kl
struct Point:
    x: i32
    y: i32

p = Point { x: 10, y: 20 }
println(p.x)            # 10

class Counter(start: i32):
    count: i32

    Counter(start: i32):
        this.count = start

    fn increment() -> i32:
        this.count = this.count + 1
        return this.count

c = Counter(10)
println(c.increment())  # 11
```

### Enums & Pattern Matching

```kl
enum Result:
    Ok(i32)
    Err(str)

match parse("123"):
    Ok(v)  => println("got: " + str(v))
    Err(e) => println("error: " + e)
```

### Error Handling

```kl
fn div(a: i32, b: i32) -> i32!:
    if b == 0:
        return error("division by zero")
    return a / b

# Propagate errors with ?
result = div(10, 2)?    # 5 (or propagates the error)
```

### Optional Chaining

```kl
name = user?.name       # None if user is None
age = user?.age ?: 0    # default 0 if None
```

### Async / Await

```kl
task = async fetch_data()    # spawn concurrent task
result = await task          # wait for result
```

### Dict / Map

```kl
users = {"alice": 30, "bob": 25}
users["charlie"] = 35
println(users["alice"])       # 30
println(users.len())           # 3
```

### Control Flow

```kl
if x > 0:
    println("positive")
elif x < 0:
    println("negative")
else:
    println("zero")

for i in 0..5:
    println(i)

for item in items:
    process(item)
else:
    println("list was empty")

# Match as expression
label = match x:
    0 => "zero"
    1 => "one"
    _ => "many"

# Ternary
status = age >= 18 ? "adult" : "minor"

# Guard (early return)
fn process(data: [i32]):
    guard len(data) > 0:
        return
    # ... rest of function

# Defer (LIFO cleanup)
fn process_file(path: str):
    file = open(path)
    defer close(file)
    # file auto-closed on return
```

### Spread & Slicing

```kl
a = [1, 2, 3]
b = [...a, 4, 5]     # [1, 2, 3, 4, 5]
c = a[0..2]          # [1, 2]
```

---

## CLI Reference

| Command | Description |
|---------|-------------|
| `kl new <project>` | Create a new project (src/, tests/, kl.toml, .gitignore) |
| `kl build [<file>]` | Compile to native binary (project mode if no file) |
| `kl run [<file>]` | Compile and execute |
| `kl build --release` | Compile with optimizations (target/release/) |
| `kl check <file>` | Type-check without codegen |
| `kl parse <file>` | Parse and print AST |
| `kl mir <file>` | Print MIR (intermediate representation) |
| `kl fmt <file>` | Format source code in-place |
| `kl test` | Run project tests |
| `kl add <dep@ver>` | Add a dependency to kl.toml |
| `kl remove <dep>` | Remove a dependency |
| `kl info` | Show project info |
| `kl lsp` | Start language server (for editor integration) |
| `kl --version` | Show version |

---

## VS Code Extension Features

- **Syntax highlighting** — keywords, types, builtins, strings, numbers, operators
- **Semantic coloring** — variables, types, functions, params colored by meaning
- **Autocompletion** — 44 builtins, all declarations, keywords, with prefix filter
- **Dot-completions** — typing `obj.` shows fields/methods (struct, class, enum, str, list, dict)
- **Scope-aware** — local variables, function params, block-scoped declarations
- **Go-to-definition** — jump to declaration of any symbol
- **Hover** — function signatures, builtin docs, identifier info
- **Snippets** — 20 common patterns (fn, class, enum, match, for, if, etc.)
- **Commands** — `KL: Run`, `KL: Build`, `KL: Check` from the command palette

---

## Feature Matrix

| Feature | Status |
|---------|--------|
| Variables, functions, control flow | ✅ |
| Structs / classes with inheritance | ✅ |
| Enums + pattern matching | ✅ |
| Generics (structs + functions) | ✅ |
| Closures (first-class functions) | ✅ |
| Async / await (thread-based) | ✅ |
| Dict / Map literals | ✅ |
| Error types (`!` / `?`) | ✅ |
| Optional chaining (`?.`) | ✅ |
| Defer, guard | ✅ |
| Match as expression | ✅ |
| Ternary operator | ✅ |
| Spread operator | ✅ |
| Range slicing | ✅ |
| Type aliases | ✅ |
| String operations | ✅ |
| RAII memory (no GC) | ✅ |
| Package manager | ✅ |
| Code formatter | ✅ |
| Language server (LSP) | ✅ |
| VS Code extension | ✅ |
| **FFI (extern "C")** | 🔜 Phase 9 |
| **HTTP server / client** | 🔜 Phase 9 |
| **Database drivers** | 🔜 Phase 9 |
| **Iterators / map / filter** | 🔜 Phase 10 |

---

## Testing

```bash
# Run all unit tests (101 tests)
cargo test -p klc_core -p klc_frontend -p klc_semantic -p klc_mir -p klc_runtime -p klc_tools

# Run all example programs (50+ programs)
for f in examples/*.kl; do kl run "$f"; done
```

---

## Examples

See the [`examples/`](examples/) directory for 50+ working programs:

```bash
kl run examples/hello.kl          # Hello, World!
kl run examples/fibonacci.kl     # fibonacci(10) = 55
kl run examples/enum_test.kl     # enums + match
kl run examples/async_test.kl   # async/await
kl run examples/generic_struct.kl  # generics
kl run examples/dict_test.kl     # dict/map
kl run examples/closure_test.kl  # closures
kl run examples/json_test.kl     # JSON parse/stringify
```

---

## Documentation

Comprehensive documentation lives in [`docs/`](docs/):

| Document | Content |
|----------|---------|
| [Vision](docs/00-vision.md) | Philosophy, design principles, language comparison |
| [Language Reference](docs/01-language-reference.md) | Complete syntax + EBNF grammar + status per construct |
| [Types, Errors & Memory](docs/02-types-errors-memory.md) | Type system, error handling, RAII, ABI, FFI |
| [Modules, Packages & Tooling](docs/03-modules-packages-tooling.md) | CLI reference, getting started, VS Code |
| [Compiler Architecture](docs/04-compiler-architecture.md) | 9-crate pipeline, repo layout, runtime |
| [Roadmap & Status](docs/05-roadmap-status.md) | Phases 0-13, implementation matrix, release checklist |

---

## Roadmap

Kyle is developed in phases. See [`docs/05-roadmap-status.md`](docs/05-roadmap-status.md)
for the full breakdown.

| Phase | Focus | Status |
|-------|-------|--------|
| 0-6 | Language design + compiler + all syntax features | ✅ Complete |
| 7 | Cross-platform (macOS ARM + Linux ARM) | 🔶 Current |
| 8 | Tooling polish (VS Code, LSP, distribution) | 🔶 Current |
| 9 | Backend & systems (FFI, HTTP, DB, ENV) | ⏸️ Next |
| 10 | Std library & ergonomics (iterators, collections) | 📅 Planned |
| 11 | Production hardening (debug, errors, testing) | 📅 Planned |
| 12 | Self-hosting (compiler in Kyle) | ⏸️ Deferred |
| 13 | Ecosystem (registry, framework, WASM) | 📅 Future |

---

## Contributing

Contributions are welcome! Please:

1. Read `AGENTS.md` for project context and design decisions
2. Check `docs/05-roadmap-status.md` for current priorities
3. Ensure `cargo build --workspace` and all 101 tests pass before submitting
4. Follow the existing code style (Rust standard, `cargo fmt`)

---

## License

[MIT](LICENSE) — Copyright (c) 2026 Kynera