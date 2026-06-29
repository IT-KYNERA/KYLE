# Migration Guide

> How to think about Kyle if you're coming from Python, Rust, or Go. Focuses on
> the **key differences** and common pitfalls.

---

## 1. From Python

Python is Kyle's closest relative in terms of readability. Both use indentation,
no semicolons, and value readability. But Kyle is compiled and statically typed.

### Syntax Differences

| Python | Kyle | Reason |
| :--- | :--- | :--- |
| `x = 5` | `x = 5` (immutable) or `x := 5` (mutable) | Mutability is explicit |
| `x: int = 5` | `x: i32 = 5` | Strong typing, explicit sizes |
| `def f(x):` | `fn f(x):` | `fn` is shorter, more common in compiled langs |
| `None` | `None` | Same (but typed as `T?`) |
| `Optional[int]` | `i32?` | Postfix `?` is more readable |
| `self` | `this` | Shorter, less typing |
| `raise Exception` | `return error("msg")` | No exceptions, errors are values |
| `try/except` | `guard` / `?` | Error propagation, fail-fast |
| `list.append(x)` | `items.add(x)` | Shorter method name |
| `len(list)` | `items.len()` | Method-call style (also `len(items)` works) |
| `str.upper()` | `s.upper()` | Same |
| `for x in range(10):` | `for x in 0..10:` | Range syntax is more concise |
| `if x is None:` | `if x is None:` | Same |
| `class X:` | `class X:` | Same (but Kyle fields are declared) |
| `def __init__`: | `X(args):` | Constructor is the class name |

### What to Watch Out For

1. **No `let`/`const`/`var`** — the operator itself declares the binding.
   `x = 5` is a declaration (immutable), not reassignment.

2. **Mutable by choice** — `x := 5` declares a mutable variable. You must
   opt into mutability with `:=`.

3. **No exceptions** — every error is a value. Use `fn f() T!` and `?` to
   propagate. No `try`/`catch`/`finally`.

4. **Typed** — all types are known at compile time. `42` is `i32`, not a
   dynamically-typed integer. You'll need explicit conversions.

5. **No duck typing** — if it looks like a duck and quacks like a duck, it
   needs a `contract` (interface) declaration.

6. **Fields are declared** — classes need explicit field declarations.
   No `self.x = value` creating fields on the fly.

---

## 2. From Rust

Kyle takes Rust's type safety but drastically simplifies the syntax. No
lifetime annotations, no borrow checker (at v1.0), no macros.

### Syntax Differences

| Rust | Kyle | Reason |
| :--- | :--- | :--- |
| `let x = 5;` | `x = 5` | No `let`, no `;` |
| `let mut x = 5;` | `x := 5` | Walrus operator is cleaner |
| `const X: i32 = 5;` | `X ::= 5` | `::=` is visually distinct |
| `fn f(x: i32) i32 { }` | `fn f(x: i32) i32:` | Indentation, no `;` |
| `Option<T>` | `T?` | Postfix `?` everywhere |
| `Result<T, E>` | `T!` | Only error type (no custom error types yet) |
| `x.ok_or("err")?` | `x?` | Implicit error propagation |
| `self` | `this` | Shorter, more common in OOP langs |
| `struct Point { x: i32 }` | `final class Point: x: i32` | Unified under `class` |
| `impl Point { fn f() {} }` | Inside `class Point:` body | Methods defined inline |
| `trait Greeter { }` | `contract Greeter:` | Contracts, not traits |
| `x.unwrap_or(default)` | `unwrap_or(x, default)` | Standalone function |
| `vec![1, 2, 3]` | `[1, 2, 3]` | List literals are built-in syntax |
| `HashMap::new()` | `{}` or `{"k": "v"}` | Dict literals are built-in syntax |
| `x.clone()` | `x.clone()` | Same (for str/list/dict) |

### What to Watch Out For

1. **No borrow checker** (yet) — Kyle v1.0 uses move semantics with a dataflow
   analysis instead of a full borrow checker. References `&T`/`&mut T` are
   planned post-v1.0.

2. **No lifetime annotations** — Kyle's dataflow analysis tracks moves
   intraprocedurally. No `'a`, no `'static`.

3. **No `impl` blocks** — methods are defined inside the class body directly.
   No separation of data and behavior.

4. **No `;`** — statements are terminated by newlines. Be careful with
   multi-line expressions — they need parentheses or continuation.

5. **No `match` as powerful** — Kyle's `match` is simpler. No or-patterns
   (yet), no guards (yet). Pattern matching will grow.

6. **No `derive` macros** — no `#[derive(Debug, Clone)]`. Common traits are
   built-in for built-in types.

7. **No custom error types** — `T!` is always `Result<T, Error>` where
   `Error` is an opaque string. Custom error types are planned.

---

## 3. From Go

Go and Kyle share simplicity, fast compilation, and a focus on tooling. But
Kyle has a richer type system and different syntax.

### Syntax Differences

| Go | Kyle | Reason |
| :--- | :--- | :--- |
| `var x int = 5` | `x: i32 = 5` or `x = 5` | Type inference, no `var` |
| `x := 5` | `x := 5` | Same walrus, but also means mutable |
| `const X = 5` | `X ::= 5` | Different syntax |
| `func f(x int) int { }` | `fn f(x: i32) i32:` | `fn`, colon, indentation |
| `x, err := f()` | `x = f()?` | No tuple errors, `?` propagates |
| `if err != nil { return }` | `guard ok else: return` | Guard pattern, fail-fast |
| `struct { X int }` | `final class: x: i32` | `final class` syntax |
| `interface { F() }` | `contract: fn f()` | Contracts instead of interfaces |
| `slice` | `[T]` | List (built-in syntax) |
| `map[string]int` | `{str: i32}` | Dict (built-in syntax) |
| `len(slice)` | `items.len()` or `len(items)` | Both work |
| `for i, v := range items { }` | `for v in items:` | Simpler range (index optional) |
| `defer f()` | `defer f()` | Same keyword, same semantics |
| `panic("msg")` | `panic("msg")` | Same (planned) |
| `error.New("msg")` | `error("msg")` | Simpler error construction |

### What to Watch Out For

1. **No goroutines** — Kyle's async is `async`/`await` with a planned
   work-stealing scheduler. No `go f()` syntax.

2. **No channels** — Kyle uses async/await and shared-nothing message passing.
   Channels are planned for the stdlib.

3. **No `nil` slice/map** — in Kyle, `items: [i32] = []` creates an empty list.
   There is no `nil` list value.

4. **No implicit `else` in `defer`** — Go's `defer` runs at function return.
   Kyle's `defer` also runs at scope exit (same semantics).

5. **No goroutine leaks** — `async` tasks are joined via `await`. Unjoined
   tasks are not garbage collected.

6. **No `select`** — not planned. Use `match` with async operations instead.

7. **No `fallthrough`** — `match` arms are exclusive. No implicit or explicit
   fallthrough.
