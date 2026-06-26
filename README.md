# Kyle Programming Language

**Readable like Python. Typed like Rust. Fast like C.**

Kyle is a compiled, statically-typed language combining Python's indentation-based readability, Rust's type safety, Go's simplicity, and LLVM's native performance.

```
fn main():
    println("Hello, World!")
```

---

## Quick Start

### Requirements

- **macOS ARM (Apple Silicon)** — the only currently supported platform
- No need to install Rust, LLVM, or any dependencies

### Install

The repo is private, so you need **one** of these:

**Option 1 — GitHub CLI (recommended):**
```bash
gh release download v0.1.1 -R IT-KYNERA/KYLE
tar xzf klc-v0.1.1-macos-arm64.tar.gz
cd klc && sudo ./install.sh --local .
```

**Option 2 — Manual download from GitHub:**
1. Go to https://github.com/IT-KYNERA/KYLE/releases
2. Download `klc-v0.1.1-macos-arm64.tar.gz`
3. Extract and install:
```bash
tar xzf klc-v0.1.1-macos-arm64.tar.gz
cd klc && sudo ./install.sh --local .
```

**Option 3 — With a GitHub token:**
```bash
export GITHUB_TOKEN=ghp_...
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh | sh
```

**Option 4 — Build from source (requires Rust):**
```bash
git clone https://github.com/IT-KYNERA/KYLE.git
cd kl
cargo build --bin klc --release
cargo build -p klc_runtime --release
# Then copy target/release/klc and target/release/libklc_runtime.a somewhere
```

### Verify
```bash
klc --version
```

### Create and run a project
```bash
klc new my_project
cd my_project
klc run
```

---

## Hello, World!

```kl
fn main():
    println("Hello, World!")
```

Save as `hello.kl`, then:
```bash
klc build hello.kl   # → produces ./hello
./hello               # → Hello, World!
# Or directly:
klc run hello.kl
```

---

## More Examples

### Fibonacci

```kl
fn fibonacci(n: i32) -> i32:
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

fn main() -> i32:
    result = fibonacci(10)
    println("fibonacci(10) = " + str(result))
    return 0
```

### Structs and Methods

```kl
class Counter:
    count: i32

    Counter(start: i32):
        this.count = start

    fn increment() -> i32:
        this.count = this.count + 1
        return this.count

    fn add(n: i32) -> i32:
        this.count = this.count + n
        return this.count

fn main():
    c = Counter(10)
    println(c.increment())   # 11
    println(c.increment())   # 12
    println(c.add(5))        # 17
```

### Match and Enums

```kl
enum Option:
    Some(i32)
    None

fn main():
    val = Option.Some(42)
    match val:
        Option.Some(v):
            println("Value: " + str(v))
        Option.None:
            println("No value")
```

### Async

```kl
fn main():
    task = async compute(42)
    result = await task
    println("Result: " + str(result))

fn compute(x: i32) -> i32:
    return x * 2
```

---

## CLI Commands

| Command | Description |
|---------|-------------|
| `klc build <file>` | Compile to native binary |
| `klc run <file>` | Compile and execute |
| `klc check <file>` | Type-check only |
| `klc parse <file>` | Show AST |
| `klc mir <file>` | Show MIR |
| `klc fmt <file>` | Format code |
| `klc new <name>` | Create new project |
| `klc --version` | Show version |
| `klc --help` | Show help |

---

## Features

- Indentation-based syntax (4 spaces)
- Strong static typing with inference
- Generics (structs + functions, monomorphized)
- Pattern matching with enums
- Closures (first-class functions)
- Async/await (thread-based)
- RAII memory management (no GC, no borrow checker)
- Classes with constructors and methods
- Dict/Map literals
- Spread operator, range slicing
- Ternary (`cond ? a : b`), match-expression
- Defer, guard, type aliases
- Package manager (`klc new`, `klc add`)
- Language server (LSP)
- Code formatter
- VS Code extension

---

## Project Structure

```
kl/
├── crates/          # 9 Rust crates (compiler)
│   ├── klc_core/    #   AST, types, spans, diagnostics
│   ├── klc_frontend/#   Lexer + parser
│   ├── klc_semantic/#   Type checker + symbol table
│   ├── klc_mir/     #   MIR lowering + optimizer + ownership
│   ├── klc_backend/ #   LLVM codegen + linker
│   ├── klc_driver/  #   Pipeline orchestration
│   ├── klc_cli/     #   CLI binary
│   ├── klc_runtime/ #   Runtime (RAII, I/O, strings, async)
│   └── klc_tools/   #   LSP, formatter, package manager
├── std/             # Standard library (.kl sources)
├── examples/        # Example programs
├── docs/            # 20 specification documents
└── vscode-kl/       # VS Code extension
```

---

## Documentation

Full documentation is in `docs/`:

| Doc | Description |
|-----|-------------|
| [Language Spec](docs/01-language-specification.md) | Complete syntax reference |
| [Formal Grammar](docs/02-formal-grammar.md) | EBNF grammar |
| [Type System](docs/04-type-system.md) | Types and type rules |
| [Standard Library](docs/07-standard-library.md) | std API reference |
| [Memory Model](docs/09-memory-model.md) | RAII + ownership |
| [Roadmap](docs/13-roadmap.md) | Current and future phases |
| [Getting Started](docs/18-getting-started.md) | Beginner's guide |
| [Syntax Reference](docs/17-syntax-reference.md) | Quick syntax lookup |

---

## Building from Source

**Prerequisites:** Rust toolchain + LLVM 18

```bash
# macOS
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)

# Build
cargo build --workspace

# Test
cargo test -p klc_core -p klc_frontend -p klc_semantic -p klc_mir -p klc_runtime -p klc_tools
```

---

## Status

- **Phase 6 — Language Completion** (current)
- 86 unit tests passing
- 52 example programs working
- macOS ARM only

See [docs/13-roadmap.md](docs/13-roadmap.md) for the full roadmap.

---

## License

MIT
