# Kyle Vision

Kyle is a compiled, statically-typed programming language that combines the
readability of Python, the type safety of Rust, the simplicity of Go, and the
performance of native code via LLVM.

---

## Why Kyle?

Existing languages force trade-offs:

- **Python:** readable but slow, runtime type errors
- **Rust:** safe and fast but complex, steep learning curve
- **Go:** simple but limited type system, no user-defined generics
- **TypeScript:** typed but transpiled, erases types at runtime
- **C#/.NET:** verbose, heavy runtime, legacy baggage

Kyle eliminates these trade-offs.

---

## Core Philosophy

```
Readable like Python
Typed like Rust
Simple like Go
Fast like C
```

---

## Design Principles

1. **Readability first** — no `self`, no `let`/`var`, no semicolons, no braces, no exceptions
2. **Explicit over implicit** — errors are values, types checked at compile time, null is explicit, async is explicit
3. **Predictable performance** — compiled to native code, LLVM backend, deterministic RAII (no GC), zero-cost abstractions
4. **Modern tooling built-in** — package manager, formatter, test runner, language server, build system
5. **Memory safety without GC** — RAII + Compiler-Inferred Ownership (the compiler inserts `free` for you)

---

## Language Highlights

- Indentation-based syntax (4 spaces)
- Strong static type system with Hindley-Milner inference
- Optional types (`Option<T>`) and error types (`T!`)
- Generics with monomorphization (structs + functions)
- Classes (with inheritance), structs (pass-by-reference), enums (tagged unions), contracts
- Async/await (thread-based runtime)
- Pattern matching with guards
- Defer, guard, unsafe blocks
- Ternary, match-expression, spread, range slicing
- Dict/Map literals with indexing
- Compiles to native machine code via LLVM 18
- RAII memory model — no garbage collector, no manual `free`
- Constants by UPPERCASE convention (`PI = 3.14`)

---

## What Kyle Eliminates

```
self / let / var keyword
public / private / protected
virtual / override
try / catch / finally
semicolons
braces for blocks
pass keyword
exceptions
hidden control flow
NULL pointers (use Option<T>)
wildcard imports
```

---

## Comparison Table

| Feature | Kyle | Python | Rust | Go | TypeScript |
|---------|------|--------|------|-----|------------|
| Compiled to native | Yes | No | Yes | Yes | No |
| Strong typed | Yes | No | Yes | Weak | Weak |
| Type inference | Yes | N/A | Yes | No | Yes |
| Generics | Yes | No | Yes | No | Yes |
| No exceptions | Yes | No | Yes | No | No |
| No GC (RAII) | Yes | No | Yes | No | No |
| Async built-in | Yes | No | No | Yes | No |
| Package manager | Built-in | 3rd-party | Built-in | Built-in | 3rd-party |
| Indentation syntax | Yes | Yes | No | No | No |
| LLVM backend | Yes | No | Yes | No | No |
| Entry point args | `[str]` | `sys.argv` | `Vec<String>` | `os.Args` | `process.argv` |

---

## Kyle in One Example

```kl
import io

contract Greeter:
    fn greet() -> str

class Person(name: str):
    fn greet() -> str:
        return "Hello, " + name + "!"

fn main(args: [str]) -> i32:
    person = Person("Anna")
    io.println(person.greet())
    return 0
```

---

## Target Audience

- Backend developers building APIs and microservices
- Systems programmers who want safety without complexity
- Teams that want Python readability with C-level performance
- Developers tired of Rust's learning curve or Go's limitations

---

## Target Platforms

- macOS (Apple Silicon + Intel)
- Linux (x64 + ARM)
- Windows (x64)

Future: WebAssembly, embedded systems

---

## Project Status

See `docs/05-roadmap-status.md` for the complete phase breakdown and verified
implementation status.

---

## Version

```
Kyle Vision v2.0
Last updated: 2026-06-26
```