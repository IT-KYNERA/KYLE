<div align="center">

<img src="vscode-kl/icons/kl_128.png" width="128" alt="Kyle">

# Kyle

**A compiled, statically-typed language for backend systems and CLI tools.**

Readable like Python · Typed like Rust · Simple like Go · Fast like C

[![License: MIT](https://img.shields.io/badge/license-MIT-6C3FC5?style=for-the-badge)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-157%20passing-6C3FC5?style=for-the-badge)](#development)
[![CI](https://github.com/IT-KYNERA/KYLE/actions/workflows/ci.yml/badge.svg)](https://github.com/IT-KYNERA/KYLE/actions/workflows/ci.yml)
[![Release](https://img.shields.io/badge/release-v0.4.0-6C3FC5?style=for-the-badge)](https://github.com/IT-KYNERA/KYLE/releases/latest)
[![Platform](https://img.shields.io/badge/platform-Linux%20ARM%20%7C%20Linux%20x64%20%7C%20macOS%20ARM-6C3FC5?style=for-the-badge)](#download)
[![VS Code](https://img.shields.io/badge/VS%20Code-extension-6C3FC5?style=for-the-badge)](vscode-kl/)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-6C3FC5?style=for-the-badge)](https://www.rust-lang.org)

</div>

---

## Download

| Platform | Arch | Link |
| :--- | :--- | :--- |
| **Linux** | ARM64 | [kl-v0.4.0-linux-arm64.tar.gz](https://github.com/IT-KYNERA/KYLE/releases/download/v0.4.0/kl-v0.4.0-linux-arm64.tar.gz) |
| **Linux** | x64 | [kl-v0.4.0-linux-x64.tar.gz](https://github.com/IT-KYNERA/KYLE/releases/download/v0.4.0/kl-v0.4.0-linux-x64.tar.gz) |
| **macOS** | ARM64 | [kl-v0.4.0-macos-arm64.tar.gz](https://github.com/IT-KYNERA/KYLE/releases/download/v0.4.0/kl-v0.4.0-macos-arm64.tar.gz) |
| **Windows** | x64 | [Build from source](#build-from-source) |

```bash
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh | sh
```

The script auto-detects your OS and architecture, downloads the correct binary,
and installs it to `/usr/local/bin/kl` (or `~/.kl/bin/kl` if not writable).

### VS Code Extension

![VS Code Extension](vscode-kl/icons/kl.png)

**Syntax highlighting, LSP diagnostics, snippets, debugging, testing UI, and more.**

#### Install from VSIX (current)

1. Download the latest `.vsix` from the [Releases page](https://github.com/IT-KYNERA/KYLE/releases)
2. In VS Code, open the Command Palette (`Cmd+Shift+P`) → `Extensions: Install from VSIX...`
3. Select the downloaded `.kl-*.vsix` file
4. Reload VS Code

#### Install from GitHub Actions (bleeding edge)

1. Go to the [Actions tab](https://github.com/IT-KYNERA/KYLE/actions) → latest CI run
2. Scroll to **Artifacts** → download `kl-vscode-extension`
3. Unzip → `Extensions: Install from VSIX...` → select the `.vsix`

#### Build from source

```bash
git clone https://github.com/IT-KYNERA/KYLE.git
cd KYLE/vscode-kl
npm install
npx @vscode/vsce package
# → kl-0.3.0.vsix created — install via Extensions: Install from VSIX...
```

**Requirements:** VS Code ^1.85, Node.js 20+, and the `kl` binary in PATH.

---

## Quick Start

```bash
# Run a script (no main() needed — works like Python)
cat > hello.kl << 'EOF'
println("Hello from Kyle!")
EOF
kl run hello.kl
# → Hello from Kyle!

# Or create a project
kl new myapp
cd myapp
cat > src/main.kl << 'EOF'
fn main() i32:
    println("Hello, World!")
    0
EOF
kl build --release
./target/release/myapp
# → Hello, World!
```

---

## Language Syntax

```kyle
# Variables — no `let`, no `mut`, no `const`
name := "Kyle"          # mutable (walrus operator)
version = "1.0"         # immutable
PI ::= 3.14159          # compile-time constant

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
| `=` immutable, `:=` mutable, `::=` constant | Labeled loops | Destructuring |
| First-class closures | Function pointer types `fn(T) U` | Imports with aliases |
| List methods (add/pop/insert/contains/etc) | String methods (upper/lower/trim/etc) | Dict literals |
| Defer, guard, unsafe | String interpolation | Variadic functions |
| Stdlib: core, io, math, str, time, json, collections, testing | LSP: diagnostics, completions, go-to-def, hover, inlay hints, code lens | VS Code: snippets (35+), tasks, testing UI, debug adapter, color theme |
| `kl fmt` formatter with `--check` | `kl test` test runner | Shell completions (bash/zsh/fish/powershell) |
| Package manager: `kl add/remove/update/outdated/publish` | Semantic version resolution with lock file | Local package cache (`~/.kl/cache/`) |

---

## CLI

### Project Commands (from a project directory with `kl.toml`)

| Command | Description |
| :--- | :--- |
| `kl build [--release]` | Compile project to native binary |
| `kl run [--release]` | Compile and execute project |
| `kl test` | Run all `#[test]` functions |
| `kl info` | Show project info |
| `kl add <dep>[@<ver>]` | Add dependency |
| `kl remove <dep>` | Remove dependency |
| `kl update` | Update lock file to latest compatible versions |
| `kl outdated` | List outdated dependencies |
| `kl publish` | Publish package to registry |
| `kl login` | Login to package registry |

### File Commands

| Command | Description |
| :--- | :--- |
| `kl build <file.kl>` | Compile single file |
| `kl run <file.kl>` | Compile and run single file |
| `kl check <file.kl>` | Type-check without codegen |
| `kl parse <file.kl>` | Parse and dump AST |
| `kl mir <file.kl>` | Parse and dump MIR |
| `kl test <file.kl>` | Run tests in single file |
| `kl fmt [file/dir]` | Format sources (project, file, or directory) |
| `kl fmt --check [file]` | Check formatting (CI mode) |

### Tools

| Command | Description |
| :--- | :--- |
| `kl new <project>` | Create new KL project |
| `kl lsp` | Start LSP server (stdio) |
| `kl completions <shell>` | Generate shell completions (bash, zsh, fish, powershell) |
| `kl help` | Show help |

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
cargo build --release --bin kl
sudo cp target/release/kl /usr/local/bin/kl
```

---

## Development

```bash
# Run all Rust tests (9 crates, 157 tests)
cargo test --workspace

# Build all crates
cargo build --workspace

# TypeScript type-check (VS Code extension)
cd vscode-kl && npx -p typescript tsc --noEmit

# Package VS Code extension
cd vscode-kl && npx @vscode/vsce package
```

---

## License

[MIT](LICENSE) — Copyright (c) 2026 Kynera
