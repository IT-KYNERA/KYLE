# Kyle Type System, Errors & Memory Model

> Type system, error handling, RAII memory management, and ABI.

---

## Type System

### Primitive Types

| Type | Size | Values | Default |
|------|------|--------|---------|
| `i32` | 4 bytes | -2^31 .. 2^31-1 | 0 |
| `i64` | 8 bytes | -2^63 .. 2^63-1 | 0 |
| `f32` | 4 bytes | IEEE 754 single | 0.0 |
| `f64` | 8 bytes | IEEE 754 double | 0.0 |
| `bool` | 1 byte | true / false | false |
| `str` | ptr + len | UTF-8 string | "" |
| `char` | 1 byte | ASCII / UTF-8 byte | '\0' |
| `void` | 0 | Unit type | — | 

### Type Inference

Kyle uses Hindley-Milner type inference. Types are inferred from literals and
expressions:

```kl
x = 42           # i32
y = 3.14         # f64
name = "Anna"    # str
nums = [1, 2, 3] # [i32]
```

### Explicit Types

```kl
mut count: i32 = 0
items: [str] = ["a", "b", "c"]
```

### Type Aliases

```kl
type UserId = i32
type Username = str

id: UserId = 42
```

Aliases resolve recursively (chained aliases work):

```kl
type Age = i32
type Years = Age
# Years → Age → i32
```

### Composite Types

| Syntax | Meaning |
|--------|---------|
| `[T]` | List of T |
| `{K: V}` | Dict with key K, value V |
| `Option<T>` | Optional (Some(value) or None) |
| `T!` | Error type (T or error) |
| `struct Name` | User-defined struct |
| `class Name` | User-defined class |
| `enum Name` | Tagged union |

### Generics

Generics are monomorphized at compile time — each concrete instantiation
generates a separate function/struct:

```kl
struct Pair<A, B>:
    first: A
    second: B

fn first<T>(items: [T]) -> Option<T>:
    if len(items) > 0:
        return Some(items[0])
    return None
```

### Casting

```kl
x: i32 = 42
y: i64 = x as i64    # explicit cast
s: str = str(x)      # builtin str() converts to string
n: i32 = len(s)      # builtin len() returns i32
```

The compiler automatically inserts widening casts (i32 → i64) when needed for
binary operations and function calls.

---

## Error Handling

Kyle has no exceptions. Errors are values.

### Error Type (`T!`)

A function with return type `T!` can return either `T` or an error:

```kl
fn parse_int(s: str) -> i32!:
    if is_digit(s):
        return 42
    return Error("not a number")
```

### Error Propagation (`?`)

The `?` operator propagates errors to the caller:

```kl
fn process() -> i32!:
    val = parse_int("123")?    # if parse_int fails, process returns the error
    return val * 2
```

### Optional Type (`Option<T>`)

```kl
fn find_user(id: i32) -> Option<User>:
    if id == 1:
        return Some(User(name: "Ana"))
    return None

# Pattern matching
match find_user(1):
    Some(u) => println(u.name)
    None => println("not found")
```

### Optional Chaining (`?.`)

Safely access fields on optional values:

```kl
name = user?.name    # if user is None, name is None
age = user?.age ?: 0  # provide default with ?:
```

### Error vs Optional

| Feature | Syntax | Use case |
|---------|--------|----------|
| Error type | `T!` + `?` | Operations that can fail (I/O, parsing, DB) |
| Optional | `Option<T>` | Values that might not exist |

---

## Memory Model: RAII + Compiler-Inferred Ownership

### No Garbage Collector

Kyle uses RAII (Resource Acquisition Is Initialization) with Compiler-Inferred
Ownership. The compiler automatically determines when to allocate and free
memory — the developer never writes `free` or `drop`.

### How It Works

1. **Allocation:** When a value is created (string, list, dict, struct), the
   compiler emits `kl_alloc(size)` to allocate heap memory.

2. **Ownership inference:** The Ownership Pass (`klc_mir/src/ownership.rs`)
   analyzes each function's control flow and inserts `kl_release(ptr)` calls at
   block exits (function return, scope end, early return, error propagation).

3. **Release:** `kl_release` decrements the reference count. If it reaches zero,
   `kl_free` is called, deallocating the memory.

4. **Retain:** When a value is stored in multiple locations (e.g., returned from
   a function and also used locally), `kl_retain` increments the count.

### What the Developer Sees

```kl
fn greet(name: str):
    msg = "Hello, " + name    # kl_alloc for the concatenated string
    println(msg)
    # compiler inserts kl_release(msg) here automatically

fn get_name() -> str:
    return "Anna"             # kl_alloc, returned (retained), caller owns it
```

No `free`, no `drop`, no `defer free(ptr)`. The compiler handles it.

### Struct ABI: Pass-by-Reference

Structs are passed by pointer (not by value) to all functions. This means:

- `fn modify(p: Point)` receives a pointer to the original struct
- Modifications to `p.x` affect the original
- No struct copying overhead

The codegen (`klc_backend/src/codegen.rs`) treats struct parameters as `ptr`
in LLVM and generates `FieldPtr` (GEP) instructions to access fields.

---

## ABI & FFI

### Runtime ABI Functions

| Function | Signature | Purpose |
|----------|-----------|---------|
| `kl_alloc` | `kl_alloc(size: i64) -> *void` | Heap allocation |
| `kl_free` | `kl_free(ptr: *void)` | Heap deallocation |
| `kl_retain` | `kl_retain(ptr: *void)` | Increment ref count |
| `kl_release` | `kl_release(ptr: *void)` | Decrement + free if zero |
| `kl_print` | `kl_print(str_ptr, len)` | Print string |
| `kl_println` | `kl_println(str_ptr, len)` | Print string + newline |
| `kl_str_concat` | `kl_str_concat(a, a_len, b, b_len) -> str` | String concatenation |
| `kl_list_new` | `kl_list_new() -> *list` | Create new list |
| `kl_list_add` | `kl_list_add(list, value)` | Append to list |
| `kl_dict_new` | `kl_dict_new() -> *dict` | Create new dict |
| `kl_dict_set` | `kl_dict_set(dict, key, key_len, value)` | Set dict entry |
| `kl_spawn_thread` | `kl_spawn_thread(fn_ptr, arg) -> i64` | Spawn async task |
| `kl_join_thread` | `kl_join_thread(handle) -> i64` | Await async task |

### FFI (Phase 9 — Planned)

FFI allows calling C library functions (libpq, sqlite3, system APIs):

```kl
extern "C":
    fn PQconnectdb(conninfo: str) -> *PGconn

unsafe:
    conn = PQconnectdb("postgresql://localhost/app")
```

**Current status:** The `unsafe` keyword is parsed and type-checked, but FFI
declarations (`extern "C"` blocks) and raw pointer lowering are not yet
implemented. This is the **top priority for Phase 9**.

---

## Concurrency Model

### Async/Await (Current Implementation)

Kyle's async/await is thread-based:

```kl
task = async 42           # spawns a new OS thread
result = await task       # joins the thread, returns 42
```

The runtime uses `std::thread::spawn` (via `kl_spawn_thread`) and
`std::thread::join` (via `kl_join_thread`). Each `async` creates a real OS
thread; `await` blocks until it completes.

**Target design (future Phase 10):** Work-stealing thread pool with coroutines
for efficient concurrent I/O. The current thread-per-task model is simple and
correct but not efficient for thousands of concurrent operations.

### Channels

```kl
ch = Channel<i32>(buffer_size)
ch.send(42)          # send value
val = ch.recv()      # receive value (blocks if empty)
```

---

## Version

```
Type System, Errors & Memory Model v1.0
Last updated: 2026-06-26
```