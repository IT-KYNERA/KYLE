<div align="center">

<img src="vscode-ky/icons/kl_128.png" width="128" alt="Kyle">

# Kyle

**A compiled, statically-typed language for backend systems and CLI tools.**

Readable like Python · Typed like Rust · Simple like Go · Fast like C

[![License: MIT](https://img.shields.io/badge/license-MIT-6C3FC5?style=for-the-badge)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-157%20passing-6C3FC5?style=for-the-badge)](#development)
[![CI](https://github.com/IT-KYNERA/KYLE/actions/workflows/ci.yml/badge.svg)](https://github.com/IT-KYNERA/KYLE/actions/workflows/ci.yml)
[![Release](https://img.shields.io/badge/release-v0.4.0-6C3FC5?style=for-the-badge)](https://github.com/IT-KYNERA/KYLE/releases/latest)
[![Platform](https://img.shields.io/badge/platform-Linux%20ARM%20%7C%20Linux%20x64%20%7C%20macOS%20ARM-6C3FC5?style=for-the-badge)](#download)
[![VS Code](https://img.shields.io/badge/VS%20Code-extension-6C3FC5?style=for-the-badge)](vscode-ky/)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-6C3FC5?style=for-the-badge)](https://www.rust-lang.org)

</div>

---

## Download

### Compiler (`ky`)

```bash
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh | sh
```

Auto-detects OS/arch, downloads binary to `/usr/local/bin/ky` (or `~/.ky/bin/kl`).

| Platform | Arch | Direct link |
| :--- | :--- | :--- |
| **Linux** | ARM64 | [kl-v0.4.0-linux-arm64.tar.gz](https://github.com/IT-KYNERA/KYLE/releases/download/v0.4.0/kl-v0.4.0-linux-arm64.tar.gz) |
| **Linux** | x64 | [kl-v0.4.0-linux-x64.tar.gz](https://github.com/IT-KYNERA/KYLE/releases/download/v0.4.0/kl-v0.4.0-linux-x64.tar.gz) |
| **macOS** | ARM64 | [kl-v0.4.0-macos-arm64.tar.gz](https://github.com/IT-KYNERA/KYLE/releases/download/v0.4.0/kl-v0.4.0-macos-arm64.tar.gz) |
| **Windows** | x64 | [Build from source](#build-from-source) |

### VS Code Extension

![VS Code Extension](vscode-ky/icons/ky.png)

**Syntax highlighting, LSP diagnostics, snippets, debugging, testing UI, and more.**

**One-command install** (requires VS Code `code` CLI in PATH):
```bash
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/vscode-ky/install-extension.sh | sh
```

The script downloads the latest VSIX from GitHub Releases and installs it via `code --install-extension`.

#### Alternative install methods

| Method | Steps |
|--------|-------|
| **VSIX from Releases** | Download `.vsix` from [Releases page](https://github.com/IT-KYNERA/KYLE/releases) → VS Code: `Extensions: Install from VSIX...` |
| **VSIX from CI** | [Actions tab](https://github.com/IT-KYNERA/KYLE/actions) → latest run → artifact `kl-vscode-extension` → unzip → Install from VSIX |
| **Build from source** | `git clone ... && cd vscode-ky && npm install && npx @vscode/vsce package` → Install from VSIX |

**Requirements:** VS Code ^1.85 and the `ky` binary in PATH.

---

## Quick Start

```bash
# Run a script (no main() needed — works like Python)
cat > hello.ky << 'EOF'
println("Hello from Kyle!")
EOF
ky run hello.ky
# → Hello from Kyle!

# Or create a project
ky new myapp
cd myapp
cat > src/main.ky << 'EOF'
fn main() i32:
    println("Hello, World!")
    0
EOF
ky build --release
./target/release/myapp
# → Hello, World!
```

---

## Language Syntax

```kyle
# Variables — no `let`, no `mut`, no `const`
name: &str = "Kyle"     # mutable (& in type)
version = "1.0"         # immutable
PI := 3.14159           # compile-time constant

# Types — T? is Option<T>, T! is Result<T, Error>
age: i32 = 30
maybe: str? = None
result: i32! = parse("42")

# Function
fn add(a: i32, b: i32) i32:
    a + b

# Final class (lightweight, no inheritance overhead)
final class Point:
    x: i32
    y: i32

# Abstract class
abstract class Animal:
    fn speak() str: "..."

# Class with inheritance
final class Dog < Animal:
    name: str

# Property with get/set
final class Counter:
    value: i32
        get: return this._val
        set(v): this._val = v
    _val: i32

# Match with or-patterns
fn describe(n: i32) str:
    match n:
        0 | 1: "small"
        2 | 3: "medium"
        _: "other"

# Pattern matching with guards
fn classify(x: i32?) str:
    match x:
        v if v > 10: "big"
        _: "small"

# Async/await
async fn fetch(n: i32) i32:
    n * n

t := async fetch(42)
result := await t

# Lists with methods
items := [1, 2, 3]
items.add(4)
items.pop()
items.contains(2)

# String methods
s := "Hello"
s.upper()     # "HELLO"
s.lower()     # "hello"
s.contains("ll")

# Closures
double := (x) => x * 2
double(5)  # 10

# Function pointer type
op : fn(i32, i32) i32 := (a, b) => a + b

# Imports
import math
from testing import assert_eq
from str import capitalize as cap

# Error handling
fn parse(s: str) i32!:
    n := int(s)?
    if n < 0: return error("negative")
    n

# Tests
#[test]
fn test_addition():
    assert(1 + 1 == 2)
    assert_eq(add(2, 3), 5)
```

---

## Features

| | | |
| :--- | :--- | :--- |
| Compiled to native via LLVM 18 | Move semantics (Copy vs Move types) | Indentation-based blocks (no braces) |
| Static typing with inference | No GC, no manual free | Generics (monomorphized) |
| Pattern matching with or-patterns | Final classes & abstract classes | Properties (get/set) |
| Contracts (interfaces) | Enums with payload | Match guards |
| Error values with `?` propagation | `T?` optionals, `T!` error types | Async/await (thread pool) |
| `=` immutable, `&T` mutable, `:=` constant | Labeled loops | Destructuring |
| First-class closures | Function pointer types `fn(T) U` | Imports with aliases |
| List methods (add/pop/insert/contains/etc) | String methods (upper/lower/trim/etc) | Dict literals |
| Defer, guard, unsafe | String interpolation | Variadic functions |
| Stdlib: core, io, math, str, time, json, collections, testing | LSP: diagnostics, completions, go-to-def, hover, inlay hints, code lens | VS Code: snippets (35+), tasks, testing UI, debug adapter, color theme |
| `ky fmt` formatter with `--check` | `ky test` test runner | Shell completions (bash/zsh/fish/powershell) |
| Package manager: `ky add/remove/update/outdated/publish` | Semantic version resolution with lock file | Local package cache (`~/.ky/cache/`) |

---

## CLI

### Project Commands (from a project directory with `ky.toml`)

| Command | Description |
| :--- | :--- |
| `ky build [--release]` | Compile project to native binary |
| `ky run [--release]` | Compile and execute project |
| `ky test` | Run all `#[test]` functions |
| `ky info` | Show project info |
| `ky add <dep>[@<ver>]` | Add dependency |
| `ky remove <dep>` | Remove dependency |
| `ky update` | Update lock file to latest compatible versions |
| `ky outdated` | List outdated dependencies |
| `ky publish` | Publish package to registry |
| `ky login` | Login to package registry |

### File Commands

| Command | Description |
| :--- | :--- |
| `ky build <file.ky>` | Compile single file |
| `ky run <file.ky>` | Compile and run single file |
| `ky check <file.ky>` | Type-check without codegen |
| `ky parse <file.ky>` | Parse and dump AST |
| `ky mir <file.ky>` | Parse and dump MIR |
| `ky test <file.ky>` | Run tests in single file |
| `ky fmt [file/dir]` | Format sources (project, file, or directory) |
| `ky fmt --check [file]` | Check formatting (CI mode) |

### Tools

| Command | Description |
| :--- | :--- |
| `ky new <project>` | Create new KL project |
| `ky lsp` | Start LSP server (stdio) |
| `ky completions <shell>` | Generate shell completions (bash, zsh, fish, powershell) |
| `ky help` | Show help |

---

## Benchmarks

Kyle compiles to native code via **LLVM 18**. Results compared to Rust, Java 21, and Python 3:

| Benchmark | Rust | **Kyle** | Java 21 | Python 3 |
| :--- | :--- | :--- | :--- | :--- |
| Primes (100K) | 0.003s | **0.006s** | 0.019s | 0.080s |
| Mandelbrot | 0.002s | **0.005s** | 0.059s | 0.030s |
| Arithmetic (10M) | 0.001s | **0.029s** | 0.026s | 0.547s |
| **vs Rust** | **1x** | **~7x** | ~15x | ~150x |

Kyle is **~2x faster than Java 21** in CPU-intensive workloads and **~20x faster than Python**.

---

## Documentation

| Doc | Description |
| :--- | :--- |
| [Language Reference](docs/01-language-reference.md) | Complete syntax reference |
| [Types, Errors & Memory](docs/02-types-errors-memory.md) | Type system, move semantics |
| [Compiler Architecture](docs/04-compiler-architecture.md) | Pipeline, crates, LLVM codegen |
| [Roadmap & Status](docs/05-roadmap-status.md) | Phases, benchmarks, optimization plan |
| [Test Checklist](TEST_CHECKLIST.md) | Manual verification checklist |

---

## Build from Source

Requires **LLVM 18** and Rust.

```bash
# Linux (ARM/x64)
sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev

# macOS (Apple Silicon)
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)

# Build
git clone https://github.com/IT-KYNERA/KYLE.git
cd KYLE
cargo build --release --bin ky
sudo cp target/release/ky /usr/local/bin/ky
```

---

## Development

```bash
# Run all Rust tests (9 crates, 157 tests)
cargo test --workspace

# Build all crates
cargo build --workspace

# TypeScript type-check (VS Code extension)
cd vscode-ky && npx -p typescript tsc --noEmit

# Package VS Code extension
cd vscode-ky && npx @vscode/vsce package
```

---

## License

[MIT](LICENSE) — Copyright (c) 2026 Kynera
## v0.5.0
Thu Jul  2 11:43:40 -04 2026
