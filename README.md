<p align="center">
  <img src="https://via.placeholder.com/120x120/6C3FC5/FFFFFF?text=K" width="120" alt="Kyle logo">
</p>

<h1 align="center">Kyle Programming Language</h1>

<p align="center">
  <b>Readable like Python. Typed like Rust. Fast like C.</b>
</p>

<p align="center">
  <a href="https://github.com/IT-KYNERA/KYLE/releases"><img src="https://img.shields.io/github/v/release/IT-KYNERA/KYLE?color=6C3FC5&label=version&style=flat-square" alt="Version"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-6C3FC5?style=flat-square" alt="License"></a>
  <a href="https://github.com/IT-KYNERA/KYLE"><img src="https://img.shields.io/badge/platform-macOS%20ARM-6C3FC5?style=flat-square" alt="Platform"></a>
</p>

---

<p align="center">
  <b>One command to install:</b><br>
  <code>curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh | sh</code>
</p>

<p align="center">
  <i>No dependencies. No Rust. No LLVM. Just a native binary.</i>
</p>

---

## Hello, World

```kl
fn main():
    println("Hello, World!")
```

```console
$ klc run hello.kl
Hello, World!
```

---

## Quick Start

```console
# Install (macOS ARM)
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh | sh

# Verify
klc --version

# Create a new project
klc new my_project

# Run it
cd my_project && klc run

# Or compile a single file
klc build hello.kl && ./hello
```

---

## Why Kyle?

| Problem | Solution |
|---------|----------|
| Python is too slow | Compiled to native code via LLVM |
| Rust is too complex | No borrow checker, no lifetimes, no `self` |
| Go is too limited | Full generics, enums, pattern matching |
| TypeScript isn't native | True native compilation, zero runtime |

Kyle gives you the **readability of Python**, the **type safety of Rust**, the **simplicity of Go**, and the **performance of C** — all in one language.

---

## Language Tour

### Variables

```kl
name = "Kyle"         # immutable by default
mut count = 0         # mutable with `mut`
PI = 3.14159          # constants (UPPERCASE)
```

### Functions

```kl
fn greet(name: str):
    println("Hello, " + name + "!")

fn add(a: i32, b: i32) -> i32:
    return a + b
```

### Control Flow

```kl
if x > 0:
    println("positive")
elif x < 0:
    println("negative")
else:
    println("zero")

while i < 10:
    println(str(i))
    i = i + 1

for i in 0..5:
    println(str(i))

for item in items:
    println(item)
```

### Structs & Methods

```kl
class Counter:
    count: i32

    Counter(start: i32):
        this.count = start

    fn increment() -> i32:
        this.count = this.count + 1
        return this.count

fn main():
    c = Counter(10)
    println(c.increment())   # 11
    println(c.increment())   # 12
```

### Enums & Pattern Matching

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

### Generics

```kl
fn first<T>(list: [T]) -> T:
    return list[0]

fn main():
    x = first([10, 20, 30])    # 10
    y = first(["a", "b", "c"]) # "a"
```

### Closures

```kl
fn main():
    double = (x) => x * 2
    println(str(double(21)))  # 42
```

### Async / Await

```kl
fn main():
    task = async compute(21)
    result = await task
    println(str(result))  # 42

fn compute(x: i32) -> i32:
    return x * 2
```

### Dict / Map

```kl
fn main():
    user = {"name": "Anna", "age": 30}
    println(user["name"])       # Anna
    println(str(user.len()))    # 2
```

### Error Handling

```kl
fn div(a: i32, b: i32) -> i32!:
    if b == 0:
        return Error("division by zero")
    return a / b

fn main():
    result = div(10, 2)?       # 5 (or panics on error)
    println(str(result))
```

### Defer

```kl
fn main():
    file = open("data.txt")
    defer close(file)
    # file is automatically closed on return
```

### Match Expression

```kl
fn classify(n: i32) -> str:
    return match n:
        0: "zero"
        1: "one"
        _: "many"
```

### Ternary

```kl
status = age >= 18 ? "adult" : "minor"
```

### Spread & Slicing

```kl
a = [1, 2, 3]
b = [...a, 4, 5]       # [1, 2, 3, 4, 5]
c = a[0..2]            # [1, 2]
```

---

## CLI

| Command | Description |
|---------|-------------|
| `klc build <file>` | Compile to native binary |
| `klc run <file>` | Compile and execute |
| `klc check <file>` | Type-check without compiling |
| `klc parse <file>` | Show AST |
| `klc mir <file>` | Show MIR |
| `klc fmt <file>` | Format source code |
| `klc new <project>` | Create a new project |
| `klc --version` | Show version |
| `klc --help` | Show help |

---

## Feature Matrix

| Feature | Status |
|---------|--------|
| Variables, functions, control flow | ✅ |
| Structs / classes with methods | ✅ |
| Enums + pattern matching | ✅ |
| Generics (structs + functions) | ✅ |
| Closures (first-class functions) | ✅ |
| Async / await (thread-based) | ✅ |
| Dict / Map literals | ✅ |
| Error types (`!` / `?`) | ✅ |
| Defer, guard | ✅ |
| Match as expression | ✅ |
| Ternary operator | ✅ |
| Spread operator | ✅ |
| Range slicing | ✅ |
| Type aliases | ✅ |
| String interpolation | ✅ |
| RAII memory management (no GC) | ✅ |
| Package manager | ✅ |
| Code formatter | ✅ |
| Language server (LSP) | ✅ |
| VS Code extension | ✅ |

---

## Examples

See the [`examples/`](examples/) directory for 50+ working programs:

```console
klc run examples/fibonacci.kl
klc run examples/enum_test.kl
klc run examples/async_test.kl
klc run examples/generic_struct.kl
klc run examples/dict_test.kl
klc run examples/closure_test.kl
```

---

## Documentation

Comprehensive documentation is available in [`docs/`](docs/):

| Document | Description |
|----------|-------------|
| [Getting Started](docs/18-getting-started.md) | Step-by-step guide |
| [Language Specification](docs/01-language-specification.md) | Full language reference |
| [Syntax Reference](docs/17-syntax-reference.md) | Quick syntax lookup |
| [Standard Library](docs/07-standard-library.md) | API reference |
| [Type System](docs/04-type-system.md) | Types and type rules |
| [Roadmap](docs/13-roadmap.md) | Current and future plans |

---

## Building from Source

> You only need this if you want to contribute to the compiler itself.
> For everyday use, use the installer above.

**Prerequisites:** Rust toolchain, LLVM 18

```bash
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)
cargo build --workspace
cargo test -p klc_core -p klc_frontend -p klc_semantic -p klc_mir -p klc_runtime -p klc_tools
```

---

## Platform Support

| Platform | Status |
|----------|--------|
| macOS ARM (Apple Silicon) | ✅ Currently supported |
| macOS Intel | 🔜 Planned |
| Linux x64 | 🔜 Planned |
| Linux ARM | 🔜 Planned |
| Windows x64 | 🔜 Planned |

---

## License

MIT
