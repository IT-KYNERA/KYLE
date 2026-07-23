<div align="center">

# Kyle

**A compiled, statically-typed language for backend systems and CLI tools.**

Readable like Python · Typed like Rust · Simple like Go · Fast like C

[![License: MIT](https://img.shields.io/badge/license-MIT-6C3FC5?style=for-the-badge)](LICENSE)
[![Release](https://img.shields.io/badge/release-v0.8.4-6C3FC5?style=for-the-badge)](https://github.com/IT-KYNERA/KYLE/releases/latest)
[![Platform](https://img.shields.io/badge/platform-macOS%20ARM/x64%20%7C%20Linux%20ARM/x64%20%7C%20Windows%20x64-6C3FC5?style=for-the-badge)](#install)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-6C3FC5?style=for-the-badge)](https://www.rust-lang.org)

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
| **macOS** | ARM64 | [ky-macos-arm64.tar.gz](https://github.com/IT-KYNERA/KYLE/releases/download/v0.8.4/ky-macos-arm64.tar.gz) |
| **Linux** | ARM64 | [ky-linux-arm64.tar.gz](https://github.com/IT-KYNERA/KYLE/releases/download/v0.8.4/ky-linux-arm64.tar.gz) |
| **Linux** | x64 | [ky-linux-x64.tar.gz](https://github.com/IT-KYNERA/KYLE/releases/download/v0.8.4/ky-linux-x64.tar.gz) |
| **Windows** | x64 | [ky-windows-x64.zip](https://github.com/IT-KYNERA/KYLE/releases/download/v0.8.4/ky-windows-x64.zip) |

> **Note**: macOS Intel (x64) is no longer supported. Use Apple Silicon (ARM64) on all modern Macs.

---

## Quick Start

```bash
ky new myapp && cd myapp
ky run
```

Or run a single file:

```bash
echo 'print("Hello from Kyle!")' > hello.ky
ky run hello.ky
```

---

## Hello World

```kyle
fn main():
    print("Hello, World!")
```

## Variables

```kyle
name = "Kyle"          # immutable (default)
count: ^i32 = 0        # mutable with ^
count += 1

items: ^[str] = []     # mutable list
```

## Collections

```kyle
items = [1, 2, 3]                    # list [i32]
items: ^[str] = ["a", "b"]           # mutable list
nums = set{1, 2, 3}                  # set set<i32>
dict = {"name": "kyle", "ver": 1}    # dict {str: i32}
q = queue{1, 2, 3}                   # queue queue<i32>
s = stack{"a", "b"}                  # stack stack<str>
```

## Imports

```kyle
use std.io                       # module
use std.io.{print, read}         # selective
use ~utils.helpers               # relative
```

## Functions

```kyle
fn add(a: i32, b: i32) i32:
    a + b

fn greet(name: &str):
    print("Hello, " + name)
```

## Error Handling

```kyle
fn parse(s: &str) i32!:
    n = int(s)?
    if n < 0: return error("negative")
    n

x: i32! = parse("42")
y = x!   # propagate on error
```

## Types

```kyle
x: i32            # primitive
x: i32?           # optional (Option)
x: i32!           # fallible (Result)
x: ^i32           # mutable
x: &str           # borrow
x: ^&[i32]!       # mutable borrow of list, may error
x: ^&[str]?       # mutable borrow of list, optional
x: ^set<i32>!     # mutable set with error
```

---

## Commands

```bash
ky new <project>      # create new project
ky run <file.ky>      # compile and run
ky build <file.ky>    # compile to binary
ky check <file.ky>    # type-check only
ky parse <file.ky>    # dump AST
ky mir <file.ky>      # dump MIR
ky fmt <file.ky>      # format source
ky test               # run project tests
```

---

## Documentation

| Resource | Location |
| :------- | :------- |
| Language Syntax | `docs/03-language/syntax/` |
| Type System | `docs/09-specification/type-system.md` |
| Collections | `docs/03-language/syntax/collections.md` |
| Modules & Imports | `docs/03-language/syntax/modules.md` |
| Syntax Reference | `docs/15-kyle-syntax-reference.md` |
| Roadmap | `docs/11-project/roadmap.md` |

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
$env:LLVM_SYS_181_PREFIX = "C:\Program Files\LLVM"

# Build
git clone https://github.com/IT-KYNERA/KYLE.git
cd KYLE
cargo build --release --bin ky
```

---

## Development

```bash
cargo test --workspace
cargo build --workspace
```

---

## License

[MIT](LICENSE) — Copyright (c) 2026 [Kynera](https://github.com/IT-KYNERA)
