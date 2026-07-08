# Kyle — Type Inventory

> Complete type audit. Every item is marked `[ ]` — must be tested and verified
> before marking as complete. See `TEST_CHECKLIST.md` for test procedures.

**Legend:**
- `[ ]` = Not tested / Needs verification
- `[x]` = Tested and working
- `[b]` = Bug known
- `[-]` = Does not exist / Not planned

---

## Language Conventions

### Ownership (v0.6)

```ky
x = 1           # immutable, COPY (i32)
x: ^T = v       # mutable, OWNED
y = x           # MOVE (non-Copy types)
y = x.clone()   # explicit COPY
f(&x)           # BORROW (immutable reference)
f(^&x)          # MUT BORROW (mutable reference)
f(x)            # MOVE (default params)
fn f(x: &T)     # BORROW param
fn f(x: ^&T)    # MUT BORROW param
```

Copy types (`y = x` does NOT move): `i8-u64`, `f32-f64`, `bool`, `char`, `ptr`

### Naming (snake_case)

| Rule | Examples |
|------|----------|
| Single-word types | `str`, `regex`, `url`, `uuid`, `bytes`, `mutex`, `future` |
| Multi-word types | `str_builder`, `atomic_i64`, `atomic_bool`, `big_int`, `date_time` |
| Functions | `spawn_thread`, `join_thread`, `parallel_for`, `fetch_add`, `to_str` |
| Type constructors | `regex("[0-9]+")`, `box(42)`, `channel<i32>(16)` |
| Constants (UPPER) | `MAX_SIZE := 1024` |

---

## 1. Primitive Types

| # | Type | Copy? | Status | Usage | Notes |
|---|------|-------|--------|-------|-------|
| 1 | `bool` | Copy | [ ] | `true`/`false` | 1 byte, `i1` in LLVM |
| 2 | `char` | Copy | [b] | `c = 'a'` | Bug: infers as i32. `char.to_i32()` = 97 |
| 3 | `byte` | Copy | [-] | — | Does not exist. Use `u8` |
| 4 | `str` | Move | [ ] | `s = "hello"` | Heap-allocated, null-terminated |
| 5 | `i8` | Copy | [ ] | `x: i8 = 127` | Signed 8-bit |
| 6 | `i16` | Copy | [ ] | `x: i16 = 32767` | Signed 16-bit |
| 7 | `i32` | Copy | [ ] | `x = 42` | Default integer literal |
| 8 | `i64` | Copy | [ ] | `x: i64 = 42` | Signed 64-bit |
| 9 | `u8` | Copy | [ ] | — | **No MirType/codegen.** Exists in Type enum only |
| 10 | `u16` | Copy | [ ] | — | Same |
| 11 | `u32` | Copy | [ ] | — | Same |
| 12 | `u64` | Copy | [ ] | — | Same |
| 13 | `f32` | Copy | [ ] | `x: f32 = 3.14` | 32-bit float |
| 14 | `f64` | Copy | [ ] | `x = 3.14` | Default float literal |
| 15 | `void` | — | [ ] | `fn foo() void:` | Return type only. Cannot instantiate |
| 16 | `never` | — | [-] | — | `!` type for diverging functions |
| 17 | `ptr` | Copy | [ ] | `p = 0 as ptr` | Raw pointer, FFI/unsafe |

---

## 2. Compound Types

| # | Type | Copy? | Status | Usage | Notes |
|---|------|-------|--------|-------|-------|
| 18 | `final class` | Move | [ ] | `final class Point:` | Non-inheritable struct |
| 19 | `class` | Move | [ ] | `class Dog :: Animal:` | Inheritance via `::` |
| 20 | `abstract class` | Move | [ ] | `abstract class Shape:` | Cannot instantiate |
| 21 | `contract` | Move | [ ] | `contract Drawable:` | Trait/interface |
| 22 | `enum` | Move | [ ] | `enum Color:` | Tagged union with payload |
| 23 | `tuple` | Move | [ ] | `p = (1, "a")` | **No MirType/codegen.** Parser-only |
| 24 | `[T; N]` | Move | [ ] | `[1, 2, 3]` / `[0; 100]` | Stack array, GEP direct |
| 25 | `{T}` | Move | [ ] | `v = {1, 2, 3}` | Heap list, dynamic |
| 26 | `{K: V}` | Move | [ ] | `d = {"k": 1}` | Heap dict |
| 27 | `set<T>` | Move | [-] | — | Does not exist |
| 28 | `slice` | Move | [-] | — | Does not exist |
| 29 | `queue<T>` | Move | [-] | — | Use `{T}` with `push()`/`pop_first()` |
| 30 | `stack<T>` | Move | [-] | — | Use `{T}` with `push()`/`pop()` |

---

## 3. Optional / Fallible

| # | Type | Status | Usage | Notes |
|---|------|--------|-------|-------|
| 31 | `T?` / `option<T>` | [b] | `name: str? = none` | Bug: `str?` causes type mismatch |
| 32 | `T!` / `result<T>` | [b] | `fn div(a,b): i32!` | Bug: `ok()`/`error()` patterns work |

---

## 4. Ownership / References

| # | Type | Copy? | Status | Usage | Notes |
|---|------|-------|--------|-------|-------|
| 33 | `^T` (mutable) | — | [ ] | `x: ^str = "hola"` | Compile-time marker, zero overhead |
| 34 | `&T` (borrow) | Copy | [ ] | `fn f(x: &str)` | Immutable borrow |
| 35 | `^&T` (mut borrow) | Copy | [ ] | `fn f(x: ^&str)` | Mutable borrow |
| 36 | `box<T>` | Move | [-] | — | Heap pointer |
| 37 | `rc<T>` | Copy | [-] | — | Single-thread refcount |
| 38 | `arc<T>` | Copy | [-] | — | Multi-thread refcount |
| 39 | `weak<T>` | Copy | [-] | — | Weak reference |

---

## 5. Concurrency / Async

| # | Type | Status | Usage | Notes |
|---|------|--------|-------|-------|
| 40 | `async fn` | [ ] | `async fn f(p: T) R:` | Thread pool |
| 41 | `await` | [ ] | `await task` | Wait for result |
| 42 | `async:` block | [ ] | `t = async: ...` | Inline async |
| 43 | `future<T>` | [-] | `t: future<str> = async: ...` | Does not exist yet |
| 44 | `channel<T>` | [-] | `ch = channel<i32>(16)` | Runtime exists, no Kyle type |
| 45 | `select` | [-] | `select: &msg -> ch: ...` | Does not exist |
| 46 | `mutex<T>` | [-] | `mutex<i32>(0); lock(m): ...` | Does not exist |
| 47 | `atomic_i64` | [-] | `atomic_i64(0).fetch_add(1)` | Does not exist |
| 48 | `atomic_bool` | [-] | `atomic_bool(false).store(true)` | Does not exist |
| 49 | `iterator` | [-] | `list.iter().map(fn).filter(fn).collect()` | Runtime KlIter exists |

---

## 6. Specialized (Must be NATIVE)

> All these types must be native Kyle types (NOT packages). Only HTTP/SQLite/Postgres are packages.

| # | Type | Now | Must be | Runtime status |
|---|------|-----|---------|----------------|
| 50 | `date_time` | [ ] | Native | `chrono` crate |
| 51 | `duration` | [ ] | Native | `chrono` |
| 52 | `date` | [ ] | Native | |
| 53 | `time` | [ ] | Native | |
| 54 | `bytes` | [ ] | Native | |
| 55 | `decimal` | [ ] | Native | |
| 56 | `uuid` | [ ] | Native | |
| 57 | `url` | [ ] | Native | |
| 58 | `regex` | [ ] | Native | |
| 59 | `env` | [ ] | Native | |
| 60 | `file` | [ ] | Native | |
| 61 | `socket` | [ ] | Native | |
| 62 | `path` | [ ] | Native | |
| 63 | `json` | [ ] | Native | |
| 64 | `big_int` | [-] | — | |

---

## 7. Packages (ONLY these)

| Package | Status | Files |
|---------|--------|-------|
| `http` | [ ] | `packages/http/` |
| `sqlite` | [ ] | `packages/sqlite/` |
| `postgres` | [-] | Pending |

---

## 8. Functions / Callables

| # | Type | Status | Usage |
|---|------|--------|-------|
| 65 | `fn(...) T` | [ ] | `fn add(a: i32, b: i32) i32:` |
| 66 | `async fn(...) T` | [ ] | `async fn fetch(url: &str) str:` |
| 67 | Closure | [ ] | `(x: i32): x * 2` |
| 68 | `static fn` | [b] | `static fn name` — parser bug |
| 69 | `str_builder` | [ ] | `str_builder(50000).append("x")` |

---

## Summary

| Status | Count |
|--------|-------|
| `[ ]` Not tested | ~50 |
| `[b]` Known bugs | 3 |
| `[-]` Not implemented | ~20 |

## Priority (to test)

| Priority | Area | Items |
|----------|------|-------|
| **P0** | Ownership v0.6 | `^T`, `&T`, `^&T`, move by default, borrow checker |
| **P1** | Primitives | u8-u64 codegen, char bug, tuple codegen |
| **P2** | Collections | `{T}` push/pop/reserve, `{K:V}` get/set, `[T;N]` access |
| **P3** | Concurrency | async fn, await, parallel_for, threads |
| **P4** | Classes | class, final class, contract, enum, generics |
| **P5** | Options/Results | `T?`, `T!`, pattern matching |
