# Vision

> Why Kyle exists, what it is, and what it is not.

---

## Why Kyle?

Most languages ask you to pick two of three:

- **Readability** (Python, Ruby)
- **Type safety** (Rust, Java, C#)
- **Raw performance** (C, C++)

Kyle is an attempt to give you **all three at once**, plus the simplicity of Go
and the modern ergonomics of TypeScript, without the historical baggage of any
of them.

It is a single compiled language, with one toolchain, one binary, and one way
to do things — but that one way is fast, safe, and pleasant to read.

---

## Core Philosophy

> **Readable like Python · Typed like Rust · Simple like Go · Fast like C**

These are the four north-stars. Every design decision is checked against them.

| North-star | What it means in practice |
|---|---|
| Readable | Indentation, no semicolons, no braces, no `self`, no `pass` |
| Typed | Static types, inferred by default, checked at compile time |
| Simple | One way to do each thing, minimal keywords, no hidden control flow |
| Fast | Compiles to native machine code via LLVM 18, no GC, no runtime overhead |

---

## Design Principles

1. **No garbage collector, no manual `free()`.** Memory is reclaimed at scope
   exit (RAII) using compiler-inferred ownership, with runtime refcounting as
   the safety net. The developer never writes `malloc` or `delete` and never
   sees a GC pause.

2. **No exceptions, no null.** Errors are values (`T!` return type) propagated
   with `?`. Absence is represented as `Option<T>`, accessed with `?.`. The
   program either succeeds with a value, fails with a typed error, or has no
   value — never with a null pointer exception.

3. **No implicit conversions.** `i32 + i64` is a compile-time error, not a
   silent narrowing. The compiler widens only where it can prove no information
   is lost, and it always warns on conversions between incompatible units.

4. **One way to do each thing.** No five kinds of loop, no ten kinds of
   string, no four ways to declare a variable. There is exactly one syntax for
   each construct, and it is the simplest one that works.

5. **Compiler-driven ergonomics.** Wherever the language can do work for the
   developer without sacrificing the four north-stars, it does: type inference,
   auto-declared variables, ownership tracking, monomorphization, error message
   linking, dead-code elimination.

---

## Language Highlights

- **Indentation-based blocks** — no braces, no semicolons, no `end`
- **Static typing with inference** — annotate when you want, infer when you don't
- **Generics** — `<T>` on functions, structs, and enums, monomorphized
- **Pattern matching** — exhaustive, expression-form, with bindings
- **Algebraic data types** — enums with payloads, matching by structure
- **First-class closures** — `(x) => x * 2`, captured by reference (RAII)
- **Async / await** — built on threads, no color, no callback hell
- **Error values** — `T!` return type, `?` propagation, never `try/catch`
- **Optionals** — `Option<T>` with `?.` chaining and `?:` default
- **Classes with inheritance, polymorphism, and visibility** — `_` protected,
  `__` private, no prefix public
- **Structs by reference** — passed by `ptr` to LLVM, no copy overhead
- **RAII memory** — `kl_alloc` / `kl_retain` / `kl_release` at scope exit
- **C-compatible ABI** — every type is `#[repr(C)]`, ready for FFI

---

## What Kyle Eliminates

| Removed | Why |
|---|---|
| `null` / `nil` | Replaced by `Option<T>` and `T!` |
| `self` / `this` parameter | Use `this` as a keyword, not a parameter |
| `try` / `catch` / `finally` | Errors are values, propagate with `?` |
| `new` keyword | `Point { x: 1, y: 2 }` is the constructor call |
| `let` / `var` / `val` / `const` keywords | `mut x = 1` for mutable, `x = 1` for immutable, `UPPER = 1` for constant |
| `pass` | Empty bodies are just empty indentation |
| `interface` / `trait` | `class X implements Y` for the same purpose |
| `impl` block | Methods live inside the class/struct |
| Garbage collector | RAII + refcounting, fully deterministic |
| `void` as a value | `void` is a return type, not a value |
| Header files | One source file per module, period |

---

## Comparison

| | Kyle | Python | Rust | Go | TypeScript |
|---|---|---|---|---|---|
| Compiles to native | ✅ | ❌ | ✅ | ✅ | ❌ |
| Static typing | ✅ inferred | ❌ | ✅ explicit | ✅ partial | ✅ partial |
| Generics | ✅ monomorphized | ✅ dynamic | ✅ monomorphized | ✅ erased | ✅ erased |
| Pattern matching | ✅ exhaustive | ❌ | ✅ | partial | ❌ |
| No GC | ✅ RAII | ❌ | ✅ | ❌ | ❌ |
| No exceptions | ✅ values | ❌ | ✅ | partial | ❌ |
| No null | ✅ | ❌ | ✅ | ❌ | ❌ |
| Indentation-based | ✅ | ✅ | ❌ | ❌ | ❌ |
| One way to do things | ✅ | ❌ | ❌ | ✅ | ❌ |
| Single binary, zero deps | ✅ | ❌ | ❌ | ✅ | ❌ |
| Compiles in seconds | ✅ | n/a | ❌ | ✅ | ❌ |

---

## Kyle in One Example

```kl
import io

contract Greeter:
    fn greet(name: str) -> str

class Person: Greeter
    name: str

    Person(name: str):
        this.name = name

    fn greet(name: str) -> str:
        return "Hello, " + name + "! I'm " + this.name + "."

fn main(args: [str]) -> i32:
    p = Person("Kyle")
    io.println(p.greet("World"))
    return 0
```

This file uses: import, contract, class with contract implementation, single
inheritance, instance method, public visibility by default, typed parameter,
return value, string interpolation via `+`, the `Person` constructor, and the
entry point. The whole program compiles to a single native binary.

---

## Target Audience

Kyle is for:

- **Backend engineers** writing HTTP services, CLI tools, and daemons who want
  Python's ergonomics with C's performance
- **Systems programmers** who want Rust's safety without the borrow-checker
  learning curve
- **Educators and students** who need a small, clear, well-typed language to
  teach the fundamentals of compilation and type systems
- **Tool authors** who need a fast, statically-linked, single-file binary for
  distribution

Kyle is **not** for:

- Browser frontends (no JS codegen — use TypeScript)
- Mobile apps (no iOS/Android targets planned)
- Embedded with hard real-time constraints (no GC pauses, but no
  pre-allocated memory model either)
- Replacing every language — Kyle is a backend and systems language

---

## Target Platforms

| Platform | Status |
|---|---|
| macOS ARM (Apple Silicon) | ✅ Supported |
| Linux ARM (aarch64) | ✅ Supported |
| Linux x64 (x86_64) | 📅 Planned |
| macOS Intel (x86_64) | 📅 Planned |
| Windows x64 | 📅 Planned |

The install script auto-detects your OS and architecture, and the release
pipeline is ready to ship additional platform binaries as they are built.

---

## Project Status

See [`docs/05-roadmap-status.md`](05-roadmap-status.md) for the full breakdown
of phases, feature matrix, and release checklist. Phases 0–6 are complete;
Phase 7 (Language Completion) is the current focus — finishing every syntax
construct, fixing all known bugs, and turning all 🔶/❌ into ✅.

---

*Version: v0.2.2 · Last updated: 2026-06-28*
