# Kyle Programming Language

**Readable like Python. Typed like Rust. Fast like C.**

Kyle is a compiled, statically-typed language with Python-like indentation syntax, Rust-grade type safety, and native performance via LLVM. No GC, no borrow checker — just fast, safe code.

---

## Install (macOS ARM)

**No dependencies required.** Just download and run.

### Option 1 — GitHub CLI

```bash
gh release download v0.2.0 -R IT-KYNERA/KYLE
tar xzf klc-v0.2.0-macos-arm64.tar.gz
cd klc && sudo ./install.sh --local .
```

### Option 2 — Manual

Download `klc-v0.2.0-macos-arm64.tar.gz` from [releases](https://github.com/IT-KYNERA/KYLE/releases), then:

```bash
tar xzf klc-v0.2.0-macos-arm64.tar.gz
cd klc && sudo ./install.sh --local .
```

### Option 3 — With a token

```bash
export GITHUB_TOKEN=ghp_...
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh | sh
```

### Verify

```bash
klc --version   # → klc v0.2.0
```

---

## Hello, World!

```kl
fn main():
    println("Hello, World!")
```

```bash
klc run hello.kl
```

---

## Language Tour

### Variables

```kl
name = "Kyle"         # immutable by default
mut count = 0         # mutable with `mut`
PI = 3.1416           # constants are UPPERCASE
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

for item in list:
    println(item)
```

### Structs / Classes

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
    println(c.increment())  # 11
```

### Enums & Match

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
    x = first([10, 20, 30])    # → 10
    y = first(["a", "b", "c"]) # → "a"
```

### Dict / Map

```kl
fn main():
    user = { "name": "Anna", "age": 30 }
    println(user["name"])      # Anna
    println(str(user.len()))   # 2
```

### Async

```kl
fn main():
    task = async compute(21)
    result = await task
    println("Result: " + str(result))

fn compute(x: i32) -> i32:
    return x * 2
```

### Closures

```kl
fn main():
    double = (x) => x * 2
    println(str(double(21)))  # 42
```

### Error Handling

```kl
fn div(a: i32, b: i32) -> i32!:
    if b == 0:
        return Error("division by zero")
    return a / b

fn main():
    result = div(10, 2)?
    println(str(result))
```

### Defer

```kl
fn main():
    f = open("file.txt")
    defer close(f)
    # f is automatically closed when main() returns
```

### Pattern Matching (expression)

```kl
fn describe(n: i32) -> str:
    return match n:
        0: "zero"
        1: "one"
        _: "many"
```

### Ternary

```kl
age_desc = age >= 18 ? "adult" : "minor"
```

---

## CLI

| Command | Description |
|---------|-------------|
| `klc build <file>` | Compile to binary |
| `klc run <file>` | Compile and run |
| `klc check <file>` | Type-check only |
| `klc fmt <file>` | Format code |
| `klc new <name>` | Create a new project |
| `klc --version` | Show version |

---

## Feature Status

| Feature | Status |
|---------|--------|
| Functions, variables, control flow | ✅ |
| Structs / classes with methods | ✅ |
| Enums + pattern matching | ✅ |
| Generics (structs + functions) | ✅ |
| Closures | ✅ |
| Async / await | ✅ |
| Dict / Map literals | ✅ |
| Error handling (`!` / `?`) | ✅ |
| Defer, guard | ✅ |
| Match as expression | ✅ |
| Ternary operator | ✅ |
| Spread operator, range slicing | ✅ |
| Type aliases | ✅ |
| Package manager | ✅ |
| Code formatter | ✅ |
| Language server (LSP) | ✅ |
| VS Code extension | ✅ |
| String interpolation | ✅ |
| RAII memory management | ✅ |

---

## Examples

See [`examples/`](examples/) for 50+ working programs:

```bash
klc run examples/fibonacci.kl
klc run examples/enum_test.kl
klc run examples/async_test.kl
klc run examples/generic_struct.kl
klc run examples/dict_test.kl
```

---

## Documentation

Full language docs in [`docs/`](docs/):

- [Language Specification](docs/01-language-specification.md)
- [Syntax Reference](docs/17-syntax-reference.md)
- [Getting Started](docs/18-getting-started.md)
- [Standard Library](docs/07-standard-library.md)
- [Roadmap](docs/13-roadmap.md)

---

## License

MIT
