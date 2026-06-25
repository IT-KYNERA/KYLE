# Kyle Vision Document v1.0

## Introduction

Kyle is a compiled programming language designed for the modern software era. It combines the readability of Python with the type safety of Rust and the performance of native compilation via LLVM.

---

## Why Kyle?

Existing languages force trade-offs:

- Python: readable but slow, runtime errors
- Rust: safe and fast but complex, steep learning curve
- Go: simple but limited type system, no generics
- TypeScript: typed but transpiled, runtime types
- C#/Java: verbose, heavy runtime, legacy baggage

Kyle aims to eliminate these trade-offs.

---

## Core Philosophy

```text
Readable like Python
Typed like Rust
Fast like C
Simple like Go
```

---

## Design Principles

### 1. Readability First

```text
No self
No let
No var

No semicolons
No braces
No exceptions
```

### 2. Explicit Over Implicit

```text
Errors are values
Types are checked at compile time
Null is explicit
Async is explicit
Imports are explicit
```

### 3. Predictable Performance

```text
Compiled to native code
LLVM backend
No hidden allocations
Deterministic RAII
Zero-cost abstractions
```

### 4. Modern Tooling

```text
Built-in package manager
Built-in formatter
Built-in test runner
Built-in language server
Built-in build system
```

---

## Target Audience

```text
Backend developers
Systems programmers
Enterprise teams
Startups building products
Developers tired of complexity
```

---

## Target Platforms

```text
macOS
Linux
Windows
```

Future:

```text
WebAssembly
iOS
Android
Embedded systems
```

---

## Language Highlights

- Indentation-based syntax (4 spaces)
- Strong static type system with inference
- Optional types (`Option<T>`) and error types (`T!`)
- Generics with monomorphization
- Classes, structs, enums, contracts
- Async/await with work-stealing scheduler
- Package manager with registry
- Compiles to native machine code via LLVM
- No exceptions — errors are values
- RAII memory model with compiler-inferred ownership (no GC)
- Constants by UPPERCASE convention (`PI = 3.14`)
- Object literals (`{ name: "Juan" }`) with dot access

---

## What Kyle Eliminates

```text
self
let / var
public / private / protected keywords
virtual / override
try / catch / finally
semicolons
braces for blocks
pass keyword
continue keyword
spawn keyword
exceptions
hidden control flow
NULL pointers
wildcard imports
circular dependencies
```

---

## Kyle in One Example

```kl
import io
import json

contract Serializable:
    fn serialize() -> str

class User : Serializable:
    name: str
    age: i32

    User(name: str, age: i32):
        name = name
        age = age

    fn serialize() -> str:
        return json.stringify(this)

fn main():
    user = User("Anna", 30)
    io.println(user.serialize())
```

---

## Comparison Table

| Feature | Kyle | Python | Rust | Go | TypeScript |
|---------|----|--------|------|-----|------------|
| Compiled | Yes | No | Yes | Yes | No |
| Type Safety | Strong | Weak | Strong | Weak | Weak |
| Type Inference | Yes | N/A | Yes | No | Yes |
| Generics | Yes | No | Yes | No | Yes |
| No Exceptions | Yes | No | Yes | No | Yes |
| GC | No (RAII) | Yes | No | Yes | Yes |
| Async Built-in | Yes | No | No | Yes | No |
| Package Manager | Built-in | Third-party | Built-in | Built-in | Third-party |
| Object Literals | Yes | No | No | No | Yes |
| Indentation Syntax | Yes | Yes | No | No | No |
| LLVM Backend | Yes | No | Yes | No | No |

---

## Market Position

Kyle occupies the space between:

```text
Python (readability)
    ↓
Kyle
    ↓
Rust (safety, performance)
```

It is designed for teams that want:

```text
Productivity of Python
Safety of Rust
Simplicity of Go
Performance of C
```

---

## Project Status

Current phase: **Phase 6 — Language Completion** (prácticamente terminado)

Phase 6 está casi completo: todas las features de sintaxis generan código
funcionando (for, generics, error handling, optional chaining, string
interpolation, defer, guard, type aliases, dict/map, spread, range slicing,
ternary, match-expression, for-else/while-else).

Próximas fases:
- **Phase 7 — Cross-Platform Support**: portar a Linux y Windows
- **Phase 8 — Distribution & Tooling**: instalador curl, web oficial,
  autocompletado LSP, empaquetado VS Code, CI/CD
- **Phase 9 — Self-Hosting**: compilador escrito en Kyle
- **Phase 10 — Production Ecosystem**: registry, WASM, etc.

See `docs/13-roadmap.md` for the phase breakdown and `docs/16-status.md`
for the verified feature matrix (what really generates code vs. what is
still a placeholder).

---

## Version

```text
Kyle Vision Document v1.2
Last updated: 2026-06-22
```
