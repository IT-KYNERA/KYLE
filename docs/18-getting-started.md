# Kyle Programming Language — Getting Started v2.0

---

## Quick Install (once available)

```bash
curl -fsSL https://kl-lang.org/install.sh | sh
```

This installs the `klc` compiler, package manager, formatter, and LSP.
No dependencies required — the binary is self-contained.

Verify installation:

```bash
klc --version
```

---

## Quick Start (once installed)

```bash
klc new hello_kyle       # Create a new project
cd hello_kyle
klc run src/main.kl      # Compile and run → "Hello, World!"
```

Or run a single file:

```bash
klc run my_file.kl       # Compile and run
klc build my_file.kl     # Compile to native binary only
./my_file                # Run the binary directly
```

---

## Development (from source)

### Prerequisites

- **Rust** 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **LLVM 18.1** (see per-platform instructions below)

### Platform Setup

#### macOS (Apple Silicon)

```bash
brew install llvm@18 zstd
# Add to ~/.zshrc:
export LIBCLANG_PATH="/opt/homebrew/opt/llvm@18/lib"
export PATH="/opt/homebrew/opt/llvm@18/bin:$PATH"
```

#### Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install llvm-18-dev libpolly-18-dev libzstd-dev
```

#### Windows

```bash
# Using chocolatey:
choco install llvm
```

### Clone & Build

```bash
git clone https://github.com/kynera/kl
cd kl
cargo build --workspace
```

### Commands (development mode)

```bash
# Compile and run a Kyle file:
cargo run --bin klc -- run hello.kl

# Once built, use the binary directly:
./target/debug/klc run hello.kl

# Or install globally:
cargo install --path crates/klc_cli
klc --version
```

---

## CLI Reference

| Command | Description |
|---------|-------------|
| `klc run <file.kl>` | Compile and run |
| `klc build <file.kl>` | Compile to native binary |
| `klc check <file.kl>` | Type-check without codegen |
| `klc parse <file.kl>` | Parse and dump AST |
| `klc mir <file.kl>` | Parse and dump MIR |
| `klc fmt <file.kl>` | Format source code |
| `klc new <project>` | Create new project |
| `klc add <dep>` | Add dependency |
| `klc remove <dep>` | Remove dependency |
| `klc test` | Run tests |
| `klc lsp` | Start language server |
| `klc help` | Show help |

---

## Project Structure

```
my_project/
├── kl.toml             # Project manifest
├── src/
│   └── main.kl         # Entry point
└── tests/
    └── test_main.kl    # Tests
```

### kl.toml

```toml
name = "my_project"
version = "0.1.0"
description = "My Kyle project"

[dependencies]
# std = "*"
```

---

## Examples

### Hello World

```kl
fn main():
    println("Hello, World!")
```

### Fibonacci

```kl
fn fibonacci(n: i32) -> i32:
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

fn main():
    result = fibonacci(10)
    println(result)     # → 55
```

### FizzBuzz

```kl
fn main():
    for i in 1..16:
        if i % 15 == 0:
            println("FizzBuzz")
        elif i % 3 == 0:
            println("Fizz")
        elif i % 5 == 0:
            println("Buzz")
        else:
            println(i)
```

### Using Option / Error Propagation

```kl
enum Option<T>:
    None
    Some(T)

fn safe_divide(a: i32, b: i32) -> Option<i32>:
    if b == 0:
        return Option.None
    return Option.Some(a / b)

fn main():
    result = safe_divide(10, 2)?
    println(result)     # → 5
```

---

## VS Code Extension

### Installation

```bash
# From VS Code Marketplace (once published):
# Search "KL Language Support" in Extensions panel

# Or from .vsix:
code --install-extension vscode-kl.vsix
```

### Features

- Syntax highlighting
- Language server (diagnostics, symbols, signature help)
- Commands: Run, Build, Type-check (Cmd+Shift+P → "KL:")
- Icon theme for `.kl` files

---

## Next Steps

- Read the [Language Specification](01-language-specification.md)
- Browse the [Examples Gallery](../../examples/)
- See the [Roadmap](13-roadmap.md) for upcoming features

---

## Version

```text
Getting Started v2.0
Last updated: 2026-06-25
```
