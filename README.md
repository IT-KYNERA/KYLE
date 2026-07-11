<div align="center">

<img src="vscode-ky/icons/ky_128.png" width="128" alt="Kyle">

# Kyle

**A compiled, statically-typed language for backend systems and CLI tools.**

Readable like Python · Typed like Rust · Simple like Go · Fast like C

[![License: MIT](https://img.shields.io/badge/license-MIT-6C3FC5?style=for-the-badge)](LICENSE)
[![CI](https://github.com/IT-KYNERA/KYLE/actions/workflows/ci.yml/badge.svg)](https://github.com/IT-KYNERA/KYLE/actions/workflows/ci.yml)
[![Release](https://img.shields.io/badge/release-v0.7.0-6C3FC5?style=for-the-badge)](https://github.com/IT-KYNERA/KYLE/releases/latest)
[![Platform](https://img.shields.io/badge/platform-macOS%20ARM/x64%20%7C%20Linux%20ARM/x64%20%7C%20Windows%20x64-6C3FC5?style=for-the-badge)](#install)
[![VS Code](https://img.shields.io/badge/VS%20Code-extension-6C3FC5?style=for-the-badge)](vscode-ky/)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-6C3FC5?style=for-the-badge)](https://www.rust-lang.org)
[![Docs](https://img.shields.io/badge/docs-kyle.kynera.lol-6C3FC5?style=for-the-badge)](https://kyle.kynera.lol)

</div>

---

## Install

### Compiler (`ky`)

**macOS / Linux** (one command):

```bash
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh | sh
```

**Windows** (PowerShell):

```powershell
iwr -Uri "https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.ps1" | iex
```

| Platform | Arch | Direct link |
| :--- | :--- | :--- |
| **macOS** | ARM64 | [ky-macos-arm64.tar.gz](https://github.com/IT-KYNERA/KYLE/releases/download/v0.7.0/ky-macos-arm64.tar.gz) |
| **Linux** | ARM64 | [ky-linux-arm64.tar.gz](https://github.com/IT-KYNERA/KYLE/releases/download/v0.7.0/ky-linux-arm64.tar.gz) |
| **Linux** | x64 | [ky-linux-x64.tar.gz](https://github.com/IT-KYNERA/KYLE/releases/download/v0.7.0/ky-linux-x64.tar.gz) |
| **Windows** | x64 | [ky-windows-x64.zip](https://github.com/IT-KYNERA/KYLE/releases/download/v0.7.0/ky-windows-x64.zip) |

### VS Code Extension

**macOS / Linux**:
```bash
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/vscode-ky/install-extension.sh | sh
```

**Windows** (PowerShell):
```powershell
iwr -Uri "https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/vscode-ky/install-extension.ps1" | iex
```

> **Platform note**: macOS Intel (x64) is no longer supported. Apple stopped shipping Intel Macs.
> Use Apple Silicon (ARM64) on all modern Macs.

---

## Quick Start

```bash
ky new myapp && cd myapp
ky run
# → Hello, World!
```

Or run a single file like a script:

```bash
echo 'println("Hello from Kyle!")' > hello.ky
ky run hello.ky
# → Hello from Kyle!
```

---

## Hello World

```kyle
fn main() i32:
    println("Hello, World!")
    0
```

## Variables

```kyle
name = "Kyle"          # immutable (default)
age: &i32 = 30         # mutable (& in type)
PI := 3.14159          # compile-time constant
```

## Functions

```kyle
fn add(a: i32, b: i32) i32:
    a + b
```

## Classes

```kyle
class Greeter:
    name: str
    Greeter(name: str):
        this.name = name
    fn greet() str:
        "Hello, " + this.name + "!"
```

## Error Handling

```kyle
fn parse(s: str) i32!:
    n := int(s)?
    if n < 0: return error("negative")
    n
```

---

## Performance

Kyle compila a código nativo via **LLVM 18** con optimizaciones SSA-form.
Benchmarks reales en **Apple M1/macOS**. 3 warmup + 5 mediciones.

Ejecutar: `bash benchmarks/run_benchmarks.sh`

| Benchmark | C | C++ | Rust | C# | Java | Go | Python | **Kyle** |
|-----------|:--:|:---:|:----:|:--:|:----:|:--:|:------:|:--------:|
| Prime Sieve (3M) | 9ms | 9ms | 9ms | 26ms | 31ms | 9ms | 193ms | **23ms** |
| Fibonacci (500M) | 121ms | 121ms | 123ms | 260ms | 139ms | 124ms | TO | **243ms** |
| String Concat (500k) | 9ms | 8ms | 3ms | 25ms | 30ms | 5ms | 22ms | **80ms** |
| MatMul (100x100x10) | 7ms | 7ms | 8ms | 33ms | 38ms | 14ms | 1206ms | **7ms** |

> **Kyle compite con C/Rust** en cómputo numérico (Prime Sieve, MatMul).
> En Fibonacci es ~2x más lento que C (runtime overhead).
> En string concat es ~10x más lento (overhead de `str_builder`).
> Python es 8-170x más lento que Kyle.

---

## Documentation

| Resource | Link |
| :------- | :--- |
| Website & docs | [kyle.kynera.lol](https://kyle.kynera.lol) |
| Language Reference | [docs/01-language-reference.md](docs/01-language-reference.md) |
| Types, Errors & Memory | [docs/02-types-errors-memory.md](docs/02-types-errors-memory.md) |
| Modules, Packages & Tooling | [docs/03-modules-packages-tooling.md](docs/03-modules-packages-tooling.md) |
| Compiler Architecture | [docs/04-compiler-architecture.md](docs/04-compiler-architecture.md) |
| Roadmap & Status | [docs/05-roadmap-status.md](docs/05-roadmap-status.md) |

---

## Build from Source

Requires **LLVM 18** and **Rust 1.81+**.

```bash
# Linux (Debian/Ubuntu)
sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev

# macOS
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)

# Windows (PowerShell as Admin)
choco install llvm --version=18.1.8
# Or download from: https://github.com/llvm/llvm-project/releases/tag/llvmorg-18.1.8
$env:LLVM_SYS_181_PREFIX = "C:\Program Files\LLVM"

# Build (all platforms)
git clone https://github.com/IT-KYNERA/KYLE.git
cd KYLE
cargo build --release --bin ky

# Binary at: target/release/ky (or ky.exe on Windows)
```

---

## Development

```bash
# Run all tests (9 crates, 157+ tests)
cargo test --workspace

# Build all crates
cargo build --workspace
```

---

## License

[MIT](LICENSE) — Copyright (c) 2026 [Kynera](https://github.com/IT-KYNERA)
