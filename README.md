<div align="center">

<img src="vscode-kl/icons/kl_128.png" width="128" alt="Kyle">

# Kyle

**A compiled, statically-typed language for backend systems.**

Readable like Python · Typed like Rust · Simple like Go · Fast like C

[![License: MIT](https://img.shields.io/badge/license-MIT-6C3FC5?style=for-the-badge)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-101%20passing-6C3FC5?style=for-the-badge)](#development)
[![Platform](https://img.shields.io/badge/platform-macOS%20ARM%20%7C%20Linux%20ARM-6C3FC5?style=for-the-badge)](#platform-support)
[![Release](https://img.shields.io/badge/release-v0.2.2-6C3FC5?style=for-the-badge)](https://github.com/IT-KYNERA/KYLE/releases/latest)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-6C3FC5?style=for-the-badge)](https://www.rust-lang.org)

</div>

---

## Why Kyle?

Kyle is a single language that gives you the **readability of Python**, the **type
safety of Rust**, the **simplicity of Go**, and the **performance of C** — without
the tradeoffs.

| | Kyle | Python | Rust | Go | C |
|---|---|---|---|---|---|
| Compiles to native machine code | ✅ | ❌ | ✅ | ✅ | ✅ |
| Static typing | ✅ | ❌ | ✅ | ✅ | ❌ |
| Generics | ✅ | partial | ✅ | ✅ | ❌ |
| Pattern matching | ✅ | ❌ | ✅ | partial | ❌ |
| RAII (no GC, no manual free) | ✅ | ❌ | ✅ | ❌ | ❌ |
| No borrow checker / lifetimes | ✅ | ✅ | ❌ | ✅ | ❌ |
| No garbage collector | ✅ | ❌ | ✅ | ❌ | ✅ |
| No null / no exceptions | ✅ | ❌ | ✅ | ❌ | ❌ |
| First-class closures | ✅ | ✅ | ✅ | ✅ | ❌ |
| Error values (not exceptions) | ✅ | ❌ | ✅ | ❌ | ❌ |
| Compiles in seconds | ✅ | — | ❌ | ✅ | ✅ |
| Single binary, zero runtime | ✅ | ❌ | ✅ | ✅ | ❌ |

---

## At a Glance

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

- **Indentation-based** — no braces, no semicolons
- **Optional types** — type inference by default, annotate when you want
- **Mutability explicit** — `mut x = 10` for mutable, `x = 10` for immutable
- **`this`** for instance reference, **no `self`**
- **Errors are values** — return `T!` and propagate with `?`

---

## Quick Start

```bash
# 1. Install Kyle (macOS / Linux)
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh | sh

# 2. Install the VS Code extension (optional, but recommended)
curl -fsSL -o /tmp/kl.vsix https://github.com/IT-KYNERA/KYLE/releases/latest/download/kl-0.2.2.vsix
code --install-extension /tmp/kl.vsix

# 3. Create and run a project
kl new hello
cd hello
kl run
```

You should see something like:

```
Hello from hello v0.1.0!
```

That's it. No toolchain to set up, no PATH gymnastics, no system dependencies.

---

## Install

Kyle is distributed as a single static binary. The install script auto-detects
your OS and architecture.

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh | sh
```

The installer downloads the latest pre-compiled release, places `kl` in
`~/.kl/bin` (or `/usr/local/bin` if writable), and adds it to your `PATH`.

### Verify

```bash
kl --version
# kl v0.2.2
```

### VS Code Extension

For syntax highlighting, autocomplete, hover, go-to-definition, semantic
coloring, and built-in snippets:

```bash
curl -fsSL -o /tmp/kl.vsix https://github.com/IT-KYNERA/KYLE/releases/latest/download/kl-0.2.2.vsix
code --install-extension /tmp/kl.vsix
```

Then in VS Code, run **Developer: Reload Window** (Cmd+Shift+P) to activate it.

### Build from Source

If you want to hack on the compiler itself:

```bash
git clone https://github.com/IT-KYNERA/KYLE
cd kl
cargo build --workspace
```

See [`docs/04-compiler-architecture.md`](docs/04-compiler-architecture.md) for
the internals.

---

## Platform Support

| Platform | Status |
|---|---|
| macOS ARM (Apple Silicon) | ✅ Supported |
| Linux ARM (aarch64) | ✅ Supported |
| Linux x64 | 📅 Planned |
| macOS Intel | 📅 Planned |
| Windows x64 | 📅 Planned |

The install script is ready for new platforms — each new release adds a new
`<platform>.tar.gz` asset, and the script picks the right one.

---

## Language Tour

### Variables & Types

```kl
name = "Kyle"           # immutable by default
mut count = 0           # mutable with `mut`
PI = 3.14159            # constant (UPPERCASE convention)

x: i32 = 42             # explicit type
items: [str] = ["a"]    # list of strings
point: (i32, i32) = (3, 4)
```

### Functions & Closures

```kl
fn add(a: i32, b: i32) -> i32:
    return a + b

# Generic
fn first<T>(items: [T]) -> Option<T>:
    if len(items) > 0:
        return Some(items[0])
    return None

# Closure
double = (x: i32) => x * 2
```

### Structs, Enums, Classes

```kl
struct Point:
    x: i32
    y: i32

p = Point { x: 10, y: 20 }
println(p.x)            # 10

enum Result:
    Ok(i32)
    Err(str)

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

### Pattern Matching

```kl
match parse("123"):
    Ok(v)  => println("got: " + str(v))
    Err(e) => println("error: " + e)

# Match as expression
label = match x:
    0 => "zero"
    1 => "one"
    _ => "many"
```

### Error Handling

Errors are values, not exceptions. Functions that can fail return `T!`, and `?`
propagates them.

```kl
fn div(a: i32, b: i32) -> i32!:
    if b == 0:
        return error("division by zero")
    return a / b

result = div(10, 2)?    # 5 (or propagates the error)
```

### Optionals

```kl
name = user?.name       # None if user is None
age = user?.age ?: 0    # default 0 if None
```

### Control Flow

```kl
for i in 0..5:
    println(i)

for item in items:
    process(item)
else:
    println("list was empty")

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

### Async / Await

```kl
task = async fetch_data()    # spawn concurrent task
result = await task          # wait for result
```

### Spread & Slicing

```kl
a = [1, 2, 3]
b = [...a, 4, 5]     # [1, 2, 3, 4, 5]
c = a[0..2]          # [1, 2]
```

---

## Features

| | |
|---|---|
| Compiled to native via LLVM 18 | RAII memory management |
| Indentation-based blocks | No GC, no manual `free()` |
| Static typing with inference | No null, no exceptions |
| Generics (functions & structs) | First-class closures |
| Pattern matching + match expressions | Async / await (thread-based) |
| Structs, classes, enums | Dict / Map literals |
| Inheritance + polymorphism | `defer` / `guard` for control flow |
| Error values with `?` propagation | Ternary, spread, range slicing |
| Mutability explicit (`mut`) | Standard library included |
| Class visibility (`_`, `__`) | Type aliases |
| Language Server (LSP) | Code formatter (`kl fmt`) |
| VS Code extension with semantic coloring | Package manager (`kl new`, `kl add`) |

---

## CLI Reference

| Command | Description |
|---|---|
| `kl new <name>` | Create a new project |
| `kl run [<file>]` | Compile and execute |
| `kl build [<file>]` | Compile to native binary |
| `kl build --release` | Optimised binary in `target/release/` |
| `kl check <file>` | Type-check only (no codegen, fast) |
| `kl parse <file>` | Print the AST |
| `kl mir <file>` | Print the MIR (mid-level IR) |
| `kl fmt <file>` | Format source in-place |
| `kl test` | Run project tests |
| `kl add <dep@ver>` | Add a dependency to `kl.toml` |
| `kl info` | Show project info |
| `kl lsp` | Start the language server |
| `kl --version` | Show the compiler version |

---

## Project Layout

`kl new <name>` creates a clean, conventional layout:

```
myapp/
├── src/
│   └── main.kl           ← entry point (fn main)
├── tests/
│   └── test_main.kl
├── kl.toml               ← project manifest
└── .gitignore
```

---

## Examples

The `examples/` directory contains 50+ working programs:

```bash
kl run examples/hello.kl              # Hello, World!
kl run examples/fibonacci.kl         # fibonacci(10) = 55
kl run examples/enum_test.kl         # enums + match
kl run examples/async_test.kl        # async / await
kl run examples/generic_struct.kl    # generics
kl run examples/dict_test.kl         # dict / map
kl run examples/closure_test.kl      # closures
kl run examples/json_test.kl         # JSON parse / stringify
kl run examples/input_prompt_test.kl # input("prompt")
kl run examples/inheritance_test.kl  # class inheritance
kl run examples/polymorphism_test.kl # method override
```

---

## Documentation

The full language specification lives in [`docs/`](docs/):

| | |
|---|---|
| [00 — Vision](docs/00-vision.md) | Philosophy, design principles, comparison |
| [01 — Language Reference](docs/01-language-reference.md) | Complete syntax + EBNF grammar + status per construct |
| [02 — Types, Errors & Memory](docs/02-types-errors-memory.md) | Type system, error handling, RAII, ABI |
| [03 — Modules, Packages & Tooling](docs/03-modules-packages-tooling.md) | CLI reference, getting started, VS Code |
| [04 — Compiler Architecture](docs/04-compiler-architecture.md) | 9-crate pipeline, repo layout, runtime |
| [05 — Roadmap & Status](docs/05-roadmap-status.md) | Phases 0–13, implementation matrix, release checklist |

---

## Development

The compiler is written in pure Rust and uses LLVM 18 via `inkwell`.

```bash
git clone https://github.com/IT-KYNERA/KYLE
cd kl
cargo build --workspace
cargo test -p klc_core -p klc_frontend -p klc_semantic -p klc_mir -p klc_runtime -p klc_tools
```

**Tests:** 101 unit tests, 0 failures.

```bash
# Run all unit tests
cargo test -p klc_core -p klc_frontend -p klc_semantic -p klc_mir -p klc_runtime -p klc_tools

# Check every example in examples/
for f in examples/*.kl; do kl check "$f"; done
```

---

## Roadmap

| Phase | Focus | Status |
|---|---|---|
| 0–6 | Language design + compiler + all syntax features | ✅ |
| 7 | Cross-platform: macOS ARM + Linux ARM | ✅ |
| 8 | Tooling: VS Code, LSP, distribution, CI/CD | ✅ |
| 9 | Backend & systems: FFI, HTTP, DB, ENV | 🔜 Next |
| 10 | Std library: iterators, collections, ergonomics | 📅 |
| 11 | Production hardening: errors, DWARF, TLS, WASM | 📅 |
| 12 | Self-hosting (compiler written in Kyle) | ⏸ Deferred |
| 13 | Ecosystem: registry, framework, website | 📅 |

See [`docs/05-roadmap-status.md`](docs/05-roadmap-status.md) for the full
breakdown and feature matrix.

---

## Contributing

Contributions are welcome.

1. Read [`AGENTS.md`](AGENTS.md) for project context, design decisions, and
   frozen rules
2. Check [`docs/05-roadmap-status.md`](docs/05-roadmap-status.md) for current
   priorities
3. Make sure `cargo build --workspace` succeeds and all 101 tests pass
4. Follow Rust standard style (`cargo fmt`)
5. Open a pull request against `main`

---

## License

[MIT](LICENSE) — Copyright (c) 2026 Kynera
