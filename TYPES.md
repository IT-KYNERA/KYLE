# Kyle — Quick Reference

> Quick index. Full details in `docs/03-language/` and `docs/04-standard-library/`.

## Ownership (v0.6)

| Syntax | Meaning |
|--------|---------|
| `x = v` | Immutable, OWNED (Copy types: i32, bool, etc.) |
| `x: ^T = v` | Mutable, OWNED |
| `y = x` | MOVE (non-Copy types: str, list, dict, class) |
| `y = x.clone()` | Explicit COPY (both alive) |
| `f(&x)` | BORROW (caller retains ownership) |
| `f(^&x)` | MUT BORROW (caller retains, mutable) |
| `fn f(x: T)` | MOVE param (default) |
| `fn f(x: &T)` | BORROW param |
| `fn f(x: ^&T)` | MUT BORROW param |

## Naming (snake_case)

| Rule | Example |
|------|---------|
| Single-word types | `str`, `regex`, `url`, `uuid`, `bytes`, `mutex` |
| Multi-word types | `str_builder`, `atomic_i64`, `big_int`, `date_time` |
| Functions | `spawn_thread`, `join_thread`, `parallel_for`, `to_str` |
| Type constructors | `box(42)`, `channel<i32>(16)`, `regex("[0-9]+")` |
| Constants | `MAX_SIZE := 1024` (UPPER) |

## Primitive Types

| Type | Copy? | Usage |
|------|-------|-------|
| `i8..u64`, `f32..f64` | Copy | `x: i32 = 42` |
| `bool`, `char`, `ptr` | Copy | `true`, `'a'`, `0 as ptr` |
| `str` | Move | `s = "hello"` |
| `void` | — | `fn f() void:` |

## Compound Types

| Type | Mutability | Usage |
|------|-----------|-------|
| `[T; N]` | Move | `[1, 2, 3]`, `[0; 100]` |
| `{T}` | Move | `{1, 2, 3}`, `.push()`, `.pop()` |
| `{K: V}` | Move | `{"k": 42}` |
| `(T1, T2)` | Move | `(1, "a")` |
| `class` / `final class` | Move | `class Dog :: Animal:` |
| `contract` | Move | `contract Drawable:` |
| `enum` | Move | `enum Color: RED GREEN` |

## Ownership Types

| Syntax | Meaning | Status |
|--------|---------|--------|
| `^T` | Mutable marker | [x] |
| `&T` | Immutable borrow | [x] |
| `^&T` | Mutable borrow | [x] |
| `box<T>` | Heap pointer | [-] |
| `rc<T>` | Ref count | [-] |
| `arc<T>` | Atomic refcount | [-] |

## Concurrency

| Feature | Status | Usage |
|---------|--------|-------|
| `async fn` | [x] | `async fn f(p: T) R:` |
| `await` | [x] | `await task` |
| `async:` block | [x] | `t = async: ...` |
| `parallel_for` | [x] | `parallel.for(fn, 0, N)` |
| `spawn_thread` | [x] | `thread.spawn(fn, arg)` |
| `future<T>`, `channel<T>`, `mutex<T>` | [-] | Not yet |

## Standard Library (native)

| Module | Import | Description |
|--------|--------|-------------|
| `core` | `option`, `result` | `T?`, `T!`, `is_some()`, `unwrap()` |
| `collections` | `list`, `set`, `iter` | push/pop/map/filter |
| `strings` | `str`, `str_builder` | trim, to_upper, contains |
| `io` | `print`, `println` | Console I/O |
| `fs` | `file` | File operations |
| `path` | `path` | Path manipulation |
| `net` | `tcp` | TCP networking |
| `http` | `client`, `server` | HTTP client/server (package) |
| `json` | `json` | JSON parse/stringify |
| `math` | `math` | max, min, sqrt, pi |
| `random` | `random` | int, float, shuffle |
| `time` | `date_time`, `duration` | Date, time, duration |
| `process` | `process` | OS processes |
| `thread` | `thread` | OS threads |
| `sync` | `mutex`, `atomic`, `channel` | Sync primitives |
| `crypto` | `crypto` | sha1, base64 |
| `regex` | `regex` | is_match, find, replace |
| `testing` | `assert` | assert.eq, assert.ne |

## See also

- `TEST_CHECKLIST.md` — Test suite (95 tests)
- `docs/03-language/` — Full language reference
- `docs/04-standard-library/` — Full API docs
- `docs/` — Complete documentation index
