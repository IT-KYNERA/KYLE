# Types, Errors & Memory

> How Kyle's type system, error handling, and memory model work under the hood.

---

## 1. Type System

Kyle is **statically typed** with **type inference** by default. The compiler
resolves all types at compile time — there is no runtime type tagging, no
`dyn` dispatch, no reflection.

### 1.1 Primitive Types

| Type | Size | Layout | Notes |
|---|---|---|---|
| `i8` | 1 byte | `i8` | Signed integer, ASCII byte |
| `i16` | 2 bytes | `i16` | Signed integer |
| `i32` | 4 bytes | `i32` | Default integer |
| `i64` | 8 bytes | `i64` | 64-bit integer |
| `u8` | 1 byte | `u8` | Unsigned byte |
| `u16` | 2 bytes | `u16` | Unsigned |
| `u32` | 4 bytes | `u32` | Unsigned |
| `u64` | 8 bytes | `u64` | Unsigned |
| `f32` | 4 bytes | `f32` | Single-precision float |
| `f64` | 8 bytes | `f64` | Double-precision float (default) |
| `bool` | 1 byte | `i8` | `0` = false, `1` = true |
| `str` | 8 bytes | `*const u8` | Null-terminated UTF-8 |
| `char` | 1 byte | `i8` | Single ASCII byte |
| `void` | 0 bytes | n/a | Return type only, not a value |
| `any` | 8 bytes | `*const u8` | Top type (opaque pointer) |

**String encoding:** Strings are null-terminated C strings (`*const u8`). The
length is computed at runtime via `kl_strlen`. There is no length field stored
alongside the pointer at the value level.

**Char encoding:** A `char` is a single byte. Full Unicode is not supported —
Unicode codepoints above 127 cannot be represented in a `char` value.

### 1.2 Inference Rules

The type checker uses **forward-flow inference** — it walks each expression
in source order, inferring types from:

1. **Literal values** — `42` is `i32`, `3.14` is `f64`, `"hi"` is `str`, etc.
2. **Variable declarations** — `x = expr` infers the type from `expr`
3. **Function return types** — propagated to call sites
4. **Generic specialization** — type parameters inferred from arguments

This is **not** a full Hindley-Milner unification. The implementation is
deliberately simpler and faster to compile.

### 1.3 Explicit Annotations

```kl
x: i32 = 42
# immutable — no mut keyword needed
name: str = "Kyle"

# mutable — use := walrus operator
name := "Kyle"
name = "Kyle2"         # allowed: mutable

items: [i32] = [1, 2, 3]
```

Annotations are checked against the inferred type. If they conflict, a
compile error is reported.

### 1.4 Composite Types

| Syntax | Type | Example |
|---|---|---|
| `[T]` | List of T | `[1, 2, 3]` |
| `{K: V}` | Dict with K keys, V values | `{"a": 1, "b": 2}` |
| `(T, U, ...)` | Tuple | `(1, "hi", 3.14)` |
| `final class Name:` | User-defined value class | `Point(1, 2)` |
| `abstract class Name:` | User-defined abstract class | `Animal` |
| `enum Name { ... }` | User-defined enum | `Result.Ok(42)` |
| `Name<T>` | Generic instantiation | `Box<i32>` |
| `T?` | Optional T | `i32?` |
| `T!` | Error-returning T | `i32!` |

Note: `final class` replaces `struct` (which remains as a deprecated alias
for migration purposes and will be removed before v1.0).

### 1.5 Generics

Generics are **monomorphized** — the compiler generates one specialized copy
of the function or struct for each type combination used at a call site.

```kl
fn identity<T>(x: T) T:
    return x

# Compiler generates:
#   identity_i32(x: i32) i32
#   identity_str(x: str) str
```

**Trade-off:** monomorphization produces faster code (everything is concrete)
but increases binary size. The compiler currently does not deduplicate
monomorphizations.

### 1.6 Type Widening

When an expression mixes two integer types, the compiler **automatically
widens** to the wider type:

```kl
x: i32 = 42
y: i64 = 9999999999
z = x + y            # i64 (widened from i32)
```

The rule is: `i8` < `i16` < `i32` < `i64`, and unsigned to signed at the
same width. Float types follow the same rule: `f32` < `f64`.

Mixing integers and floats (e.g. `i32 + f64`) is a compile error.

### 1.7 Casting

```kl
x = 42
y = x as i64          # explicit cast from i32 to i64
z = 3.14 as i32       # float to int (truncates)
c = 65 as char        # int to char
```

The `as` keyword performs explicit type conversion. The compiler must be
able to statically verify the conversion is valid (no runtime type checks).

---

## 2. Error Handling

Kyle has **no exceptions**. Errors are values. There are two complementary
mechanisms: `T!` return types for fallible functions, and `T?` for optional
values.

### 2.1 The `T!` Return Type

A function with `T!` returns either `T` (success) or an error. Internally
this is represented as `Option<T>` in the runtime, where `Some(v)` is success
and `None` is the error.

```kl
fn parse(s: str) i32!:
    if s == "":
        return error("empty string")
    n = int(s)?
    if n < 0:
        return error("negative")
    return n
```

**`error("msg")`** is a built-in that constructs an error value. The string
is preserved for runtime error messages and debug output.

### 2.2 The `?` Operator

`expr?` extracts the value from a `T?` or propagates the error from a `T!`.
If the value is `None`/error, the current function returns immediately with
that value.

```kl
fn read_int(path: str) i32!:
    fd = open(path, 0)?           # propagate error if open fails
    line = read_str(fd, 4096)?    # propagate error if read fails
    close(fd)
    return int(line)?            # propagate error if parse fails
```

`?` can only be used in a function with a `T!` return type. Using it in a
function that returns `T` (not `T!`) is a compile error.

### 2.3 The `T?` Optional Type

`T?` is the syntactic sugar for optional values. It is the only public
syntax — the internal representation is `Option<T>` but developers never
write it directly.

```kl
name: str? = find_user("alice")    # may or may not exist
```

Matching an optional:

```kl
match name:
    Some(n): process(n)
    None:    handle_missing()
```

Chained access with `?.`:

```kl
name = user?.name        # returns None if user is None
```

The match arms use `:` (not `=>`).

### 2.4 Errors vs. Options — When to Use Which

| Use | When |
|---|---|
| `T!` return type | The function can fail for a reason (file not found, parse error) |
| `T?` | The value is fundamentally optional (lookup miss, missing field) |
| `T` (no return type) | The function always succeeds |

In practice, `T!` and `T?` are the same type internally. The `!` is a marker
that says "this function may fail and you must handle it".

### 2.5 What is NOT an Error

Kyle's error model does not include:

- **Panics** — use `assert(condition)` if you want a runtime check that aborts
- **Exceptions** — there is no `try`/`catch`/`finally`
- **Stack unwinding** — there is no destructor-on-throw
- **Error codes** — there is no `errno` or C-style return code

---

## 3. Memory Model: Move Semantics (Planned)

Kyle is moving from a **reference-counted** model to **move semantics by
default** (Phase 6). This section documents the *target* memory model. The
current implementation uses move semantics; the transition
is underway.

### 3.1 Core Principle: Move by Default, Copy on Intent

| Category | Behavior | Examples |
|---|---|---|
| **Copy types** | Implicitly copied on assignment | `i8`–`i64`, `u8`–`u64`, `f32`, `f64`, `bool`, `char`, `void` |
| **Move types** | Ownership transfers on assignment; source is invalidated | `final class`, `abstract class`, `[T]`, `{K: V}`, `str`, `T?`, `T!` |
| **Clone** | Explicit deep copy via `.clone()` | `b = a.clone()` |

Primitives are `Copy` — they live on the stack and assignment duplicates
the bits. Classes and collections are `Move` — they own heap resources and
assignment transfers ownership.

```kl
# Copy — stack value, bits are duplicated
x: i32 = 42
y = x              # both x and y are valid, independent copies

# Move — ownership transfers
a = [1, 2, 3]
b = a              # a is now invalid (move)
# print(a)        # compile error: a was moved

# Explicit clone
c = b.clone()      # deep copy, both b and c are valid
```

### 3.2 What the Compiler Does

The compiler's **move checker** (Phase 6) tracks:

1. **Live variables** — which names hold valid values at each program point
2. **Move paths** — when a value is assigned to another name, the source is
   marked as moved (invalidated)
3. **Re-initialization** — moved-from variables can be re-assigned:
   ```kl
   a = [1, 2, 3]
   b = a           # a is moved
   a = [4, 5, 6]   # re-initialization is fine
   ```

### 3.3 What Happens at Scope Exit

When a variable goes out of scope:

- **Copy types:** nothing — the value is on the stack, no destructor needed
- **Move types:** the compiler inserts a `kl_release` call to free associated
  heap memory, but **only if** the value was not moved out (ownership was not
  transferred). Once moved, the destructor is suppressed — the new owner
  is responsible.

This is analogous to Rust's `drop` but managed by the compiler rather than
via a trait. The compiler knows statically whether a variable is the
current owner.

### 3.4 Final Class ABI: Pass-by-Reference

Final classes are passed by reference (pointer) in function calls. There
is no copy overhead for class parameters:

```kl
final class Point:
    x: i32
    y: i32

fn distance(a: Point, b: Point) f64:
    # `a` and `b` are pointers in the LLVM IR
    dx = a.x - b.x
    dy = a.y - b.y
    return sqrt(dx * dx + dy * dy)
```

In the generated LLVM IR, `Point` parameters are `ptr` (8 bytes), not the
full struct layout. This matches C ABI conventions.

If you need a stack copy (for mutation without affecting the caller), use
`.clone()`.

### 3.5 List and Dict Storage

Lists and dicts are **heap-allocated** via `kl_list_new` / `kl_dict_new`.
With move semantics, assignment transfers ownership:

```kl
a = [1, 2, 3]      # kl_list_new, a owns the allocation
b = a              # move: a is invalidated, b owns the allocation
# ... at end of scope ...
# kl_release(b) -> refcount=0, frees the list
# kl_release(a) is NOT emitted — a was moved out
```

### 3.6 String Literals

String literals (`"hello"`) are **not** owned by any variable. They are
stored as static constant data in the binary's `.rodata` section. You cannot
free them, and you don't need to.

When a string literal is assigned to a variable, the compiler treats it as
a static string reference — no allocation, no move. Concatenation results
(e.g. `"a" + "b"`) produce owned strings via `kl_concat`.

### 3.7 Refcounting: Legacy Mechanism

The current compiler (pre-Phase 6) uses **reference counting** as the
default memory management strategy. Every heap allocation starts with
refcount = 1; `kl_retain` and `kl_release` are inserted automatically.

This is being replaced by move semantics because:

- **Performance:** refcounting adds atomic operations on every copy, even
  in single-threaded code
- **Predictability:** refcounting cannot handle cyclic references without
  an explicit weak-reference mechanism
- **Ergonomics:** move semantics makes ownership explicit at the syntax level

After Phase 6, reference counting will remain only in the stdlib types
`Rc<T>` and `Arc<T>` for shared ownership scenarios where single-owner
semantics do not apply.

### 3.8 Summary of Memory Operations

| Operation | Copy types | Move types |
|---|---|---|
| Assignment (`=`) | Bitwise copy | Ownership transfer, source invalidated |
| Function parameter | Bitwise copy | Reference passed (ABI) |
| Return value | Bitwise copy | Ownership transferred to caller |
| `.clone()` | N/A (free) | Deep copy, both valid |
| Scope exit | Nothing | `kl_release` if owned |
| Borrow (`&`) | Reference | Reference (no move) |

---

## 4. ABI & FFI (Planned)

Every Kyle type is laid out in a way that is **C-compatible**. You will be
able to call C libraries directly from Kyle (Phase 9).

### 4.1 Runtime ABI Table

| C signature | Used for |
|---|---|
| `void kl_print(const u8* s, i32 len)` | `print(s)` |
| `void kl_println(const u8* s, i32 len)` | `println(s)` |

| `u8* kl_i64_to_str(i64 v)` | `str(v)` |
| `i64 kl_str_to_i64(const u8* s)` | `int(s)` (planned) |
| `i32 kl_strlen(const u8* s)` | `len(s)` for strings |
| `u8* kl_concat(const u8* a, i32 a_len, const u8* b, i32 b_len)` | string `+` |
| `i32 kl_eq_str(const u8* a, i32 a_len, const u8* b, i32 b_len)` | string `==` |
| `i32 kl_input()` | `input()` |
| `i32 kl_input_with_prompt(const u8* prompt, i32 prompt_len)` | `input(p)` |
| `i32 kl_open(const u8* path, const u8* mode)` | `open(path, mode)` |
| `i32 kl_close(i32 fd)` | `close(fd)` |
| `u8* kl_read_str(i32 fd, i32 count)` | `read_str(fd, n)` |
| `i32 kl_write_str(i32 fd, const u8* s)` | `write_str(fd, s)` |
| `i32 kl_sleep(i32 ms)` | `sleep(ms)` |
| `i32 kl_now()` | `now()` |
| `void* kl_alloc(i64 size)` | heap allocation |
| `void kl_free(void* ptr)` | heap free |
| `void kl_retain(void* ptr)` | refcount++ (legacy) |
| `void kl_release(void* ptr)` | refcount-- / free (legacy) |
| `i32 kl_str_to_upper(const u8* s, i32 len)` | `to_upper(s)` |
| `i32 kl_str_to_lower(const u8* s, i32 len)` | `to_lower(s)` |
| `i32 kl_str_trim(const u8* s, i32 len)` | `trim(s)` |
| `i32 kl_str_replace(const u8* s, i32 len, const u8* old, const u8* new)` | `replace(s, o, n)` |
| `i32 kl_char_at(const u8* s, i32 idx)` | `char_at(s, i)` |
| `i32 kl_is_digit(char c)` | `is_digit(c)` |
| `i32 kl_is_alpha(char c)` | `is_alpha(c)` |
| `i32 kl_is_alnum(char c)` | `is_alnum(c)` |
| `i32 kl_is_whitespace(char c)` | `is_whitespace(c)` |
| `i32 kl_is_upper(char c)` | `is_upper(c)` |
| `i32 kl_is_lower(char c)` | `is_lower(c)` |
| `i32 kl_ord(char c)` | `ord(c)` |
| `i32 kl_substr(const u8* s, i32 start, i32 count)` | `substr(s, st, n)` |
| `void* kl_list_new()` | new list |
| `void kl_list_push(void* list, i64 value)` | list append |
| `i64 kl_list_pop(void* list)` | list pop |
| `i64 kl_list_get(void* list, i32 idx)` | list index |
| `void kl_list_set(void* list, i32 idx, i64 value)` | list assign |
| `i32 kl_list_len(void* list)` | list length |
| `void* kl_list_slice(void* list, i32 start, i32 end)` | list slice |
| `void kl_list_extend(void* dst, void* src)` | list extend |
| `void* kl_dict_new()` | new dict |
| `void kl_dict_set(void* dict, const u8* key, i32 key_len, i64 value)` | dict set |
| `i64 kl_dict_get(void* dict, const u8* key, i32 key_len)` | dict get |
| `i32 kl_dict_len(void* dict)` | dict length |
| `i32 kl_range(i64 start, i64 end)` | `range()` iterator |
| `i64 kl_spawn_thread(void* fn, i64 arg)` | `async` |
| `i64 kl_join_thread(i64 handle)` | `await` |
| `void* kl_init_args(i32 argc, const u8** argv)` | CLI arg init |

### 4.2 FFI (Planned — Phase 9)

```kl
extern "C":
    fn PQconnectdb(conninfo: str) *PGconn
    fn PQexec(conn: *PGconn, query: str) *PGresult
    fn PQresultStatus(res: *PGresult) i32
```

FFI declarations are parsed but not yet lowered to function calls. The
unsafe block (`unsafe:`) is the only context where external calls are
allowed.

---

## 5. Concurrency Model

### 5.1 Async / Await (Current)

Async is implemented as **thread spawning**:

```kl
task = async expensive_computation()  # spawns a new thread
result = await task                    # joins the thread
```

Internally:

1. `async <expr>` lowers to `kl_spawn_thread(<closure>, 0)`
2. Returns an `i64` thread handle
3. `await <handle>` lowers to `kl_join_thread(<handle>)`, which blocks

This is simple and correct, but spawns a real OS thread per async expression.
Work-stealing coroutines (similar to Tokio) are planned for Phase 11.

### 5.2 Channels (Planned)

```kl
ch = Channel<i32>(10)      # buffered channel
ch.send(42)                 # send (blocks if full)
val = ch.recv()            # receive (blocks if empty)
ch.close()                  # close the channel
```

Channels exist in the runtime (`klc_runtime/src/channel.rs`) but are not yet
exposed to the Kyle language. This is planned for Phase 9.

---

*Version: v0.3.0 · Last updated: 2026-06-28*
