<div align="center">

<img src="vscode-kl/icons/kl_128.png" width="128" alt="Kyle">

# Kyle

**A compiled, statically-typed language for backend systems and CLI tools.**

Readable like Python · Typed like Rust · Simple like Go · Fast like C

[![License: MIT](https://img.shields.io/badge/license-MIT-6C3FC5?style=for-the-badge)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-123%20passing-6C3FC5?style=for-the-badge)](#development)
[![CI](https://github.com/IT-KYNERA/KYLE/actions/workflows/ci.yml/badge.svg)](https://github.com/IT-KYNERA/KYLE/actions/workflows/ci.yml)
[![Release](https://img.shields.io/badge/release-v0.4.0-6C3FC5?style=for-the-badge)](https://github.com/IT-KYNERA/KYLE/releases/latest)
[![Platform](https://img.shields.io/badge/platform-Linux%20ARM%20%7C%20Linux%20x64%20%7C%20macOS%20ARM-6C3FC5?style=for-the-badge)](#download)
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

Syntax highlighting, autocomplete, and LSP support (coming soon).

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
mkdir myapp && cd myapp
cat > main.kl << 'EOF'
fn main() i32:
    println("Hello, World!")
    0
EOF
kl build --release main.kl
./target/release/main
# → Hello, World!
```

---

## Language Syntax

```kyle
# Variables
name := "Kyle"          # mutable (walrus)
version = "1.0"          # immutable
PI ::= 3.14159           # compile-time constant

# Function — no ->, just return type
fn add(a: i32, b: i32) i32:
    a + b

# Class with constructor, property, and inheritance
class Animal:
    fn speak() str: "..."

class Dog: Animal:
    name: str
    Dog(name: str):
        this.name = name
    fn speak() str:
        "Woof! I'm " + this.name

# Property with get/set
class Counter:
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

# Async/await
async fn fetch(n: i32) i32:
    n * n

t := async fetch(42)
result := await t

# Lists with methods
items := [1, 2, 3]
items.add(4)
items.pop()
items.len()
items.contains(2)
items.insert(1, 99)
items.reverse()
items.clear()

# String methods
s := "Hello"
s.upper()     # "HELLO"
s.lower()     # "hello"
s.contains("ll")  # 1 (true)

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
```

---

## Benchmarks

Kyle compiles to native code via **LLVM 18**. Results compared to Rust, Java 21, and Python 3:

| Benchmark | Rust | **Kyle** | Java 21 | Python 3 |
| :--- | :--- | :--- | :--- | :--- |
| Primes (100K) | 0.003s | **0.006s** | 0.019s | 0.080s |
| Mandelbrot | 0.002s | **0.005s** | 0.059s | 0.030s |
| Arithmetic (10M) | 0.001s | **0.029s** | 0.026s | 0.547s |
| **vs Rust** | **1x** | **~7x** | ~15x | ~150x |

Kyle is **~2x faster than Java 21** in CPU-intensive workloads and **~20x faster than Python**. The gap with Rust (~7x) is due to the absence of SSA Form in Kyle's MIR — planned for a future optimization phase.

---

## Features

| | | |
| :--- | :--- | :--- |
| Compiled to native via LLVM 18 | Move semantics (Copy types vs Move types) | Indentation-based blocks (no braces) |
| Static typing with inference | No GC, no manual free | Generics (monomorphized) |
| Pattern matching with or-patterns | Classes, inheritance, polymorphism | Properties (get/set) |
| Contracts (interfaces) | Enums with payload | Match guards |
| Error values with `?` propagation | `T?` optionals, `T!` error types | Async/await (thread pool) |
| `=` immutable, `:=` mutable, `::=` constant | Labeled loops | Destructuring |
| First-class closures | Function pointer types `fn(T) U` | Imports with aliases |
| List methods (add/pop/insert/contains/etc) | String methods (upper/lower/trim/etc) | Dict literals |
| Defer, guard, unsafe | String interpolation | Variadic functions |
| Stdlib: math, io, json, time, testing | 123 unit tests, all passing | 0 warnings, clean build |

---

## CLI

| Command | Description |
| :--- | :--- |
| `kl run file.kl` | Compile and run (works without `main()`) |
| `kl build file.kl` | Compile to native binary |
| `kl build --release` | Optimized binary (`-O2`/`-O3`) |
| `kl check file.kl` | Type-check only (fast) |
| `kl mir file.kl` | Print MIR intermediate representation |

---

## Documentation

| Doc | Description |
| :--- | :--- |
| [Language Reference](docs/01-language-reference.md) | Complete syntax reference |
| [Types, Errors & Memory](docs/02-types-errors-memory.md) | Type system, move semantics |
| [Compiler Architecture](docs/04-compiler-architecture.md) | Pipeline, crates, LLVM codegen |
| [Roadmap](docs/05-roadmap-status.md) | Phases, benchmarks, optimization plan |
| [Stdlib Reference](docs/06-stdlib.md) | Standard library modules |
| [Migration Guide](docs/07-migration-guide.md) | From Python, Rust, or Go |

---

## Build from Source

Requires **LLVM 18** and Rust.

```bash
# Linux
sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev

# macOS
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
cargo test --workspace   # 123 tests
cargo build --release    # 0 warnings
```

---

## License

[MIT](LICENSE) — Copyright (c) 2026 Kynera
