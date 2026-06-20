# Kyle Memory Model Specification v2.0 — RAII + Compiler-Inferred Ownership

---

## Philosophy

Kyle manages memory automatically using RAII (Resource Acquisition Is Initialization) with compiler-inferred ownership. The developer never manually allocates or frees memory. The compiler determines at compile time whether a value can be moved (zero-cost) or needs reference counting.

```text
No malloc
No free
No GC pauses
No manual memory management
No pointer arithmetic
Default memory safety
Deterministic destruction
```

---

## Design Goals

```text
Safe by default
Zero-cost moves (like Rust)
Deterministic destruction on scope exit
No dangling pointers
No use-after-free
No double-free
No buffer overflows
No garbage collector pauses
```

---

## Memory Regions

Kyle uses two memory regions:

```text
Stack
Heap
```

---

## Stack Allocation

Stack is used for:

```text
Local variables (primitives)
Function parameters
Return addresses
Small fixed-size values (structs, tuples)
Arrays proven not to escape the function
```

Stack allocation is:

```text
Fast (increment / decrement pointer)
Deterministic (LIFO order)
No fragmentation
Zero overhead — no reference counting needed
```

### Stack-Allocated Types

```kl
i8, i16, i32, i64
u8, u16, u32, u64
f32, f64
bool
char
struct (by default)
tuple (by default)
```

### Example

```kl
fn compute():
    x: i32 = 10
    y: i32 = 20
    result = x + y
```

All variables live on the stack. Freed instantly when the function returns.

---

## Heap Allocation

Heap is used for:

```text
Dynamic objects (class instances)
Lists (arrays)
Dictionaries / maps
Strings longer than 16 bytes
Closures capturing variables
Values that escape the current function
```

### Example

```kl
fn create_user() -> User:
    user = User("Anna", 30)
    return user
```

`User` is allocated on the heap. Ownership is transferred to the caller (zero-cost move).

---

## Ownership Model — Compiler-Inferred

Kyle does NOT require the programmer to write ownership annotations. The compiler infers the ownership model automatically.

### Move Inference (Zero-Cost)

When a value is used only once, the compiler treats it as a **move** — the reference is passed directly with zero overhead:

```kl
fn main():
    datos = [1, 2, 3, 4, 5]     # heap allocation
    resultado = procesar(datos)  # COMPILER: "datos" used only once
    # → MOVE: passes reference, zero refcount overhead
    print(resultado)
    # FIN → datos and resultado freed instantly
```

### Shared Reference Counting (Automatic)

When a value is used multiple times, the compiler automatically inserts retain/release:

```kl
fn main():
    config = LoadConfig()
    process(config)              # first use
    print_summary(config)        # second use
    # COMPILER: cannot be move, needs refcounting
    # → inserts retain/release automatically
    # FIN → refcount = 0 → freed
```

The programmer never writes `Rc::new()`, `Arc::new()`, `Box::new()`, `&`, `&mut`, or lifetime annotations. The compiler handles everything.

### Stack Optimization (Escape Analysis)

If an object does not escape the function scope, the compiler allocates it on the stack:

```kl
fn calcular() -> i32:
    items = [1, 2, 3]           # COMPILER: does not escape
    mut total = 0
    for n in items:
        total += n
    return total
    # → COMPILER: items goes on the STACK, not heap. Zero overhead.
```

---

## RAII — Deterministic Destruction

When a variable goes out of scope, its destructor runs immediately. This is the core of RAII.

### Scope Exit

```kl
fn ejemplo():
    archivo = File.Open("data.txt")     # resource acquired
    datos = archivo.read()
    # FIN → archivo goes out of scope
    # → File's destructor runs: closes the file, flushes buffers
    # → datos freed
    # All deterministic, all immediate. No GC pause.
```

### Defer Not Needed

Because RAII handles cleanup automatically, `defer` statements are rarely needed:

```kl
# With RAII, this is handled automatically:
fn escribir():
    archivo = File.Open("out.txt")
    archivo.write("Hello")
    # ← File closes automatically at the end of the function
```

### Compound Assignment

Compound assignments like `x += 1`, `x *= 2` require `mut`:

```kl
fn ejemplo():
    total = 0       # immutable
    total += 5      # ERROR: cannot modify immutable variable

fn ejemplo():
    mut total = 0   # mutable
    total += 5      # OK
```

---

## Reference Counting

Reference counting is used automatically when the compiler detects that a value is shared (used more than once). The programmer never interacts with it directly.

### Thread Safety

```text
Single-thread: lightweight (non-atomic) reference counting
Multi-thread: automatic atomic reference counting when needed
```

### Cycle Detection

The compiler detects reference cycles automatically and inserts weak references as needed:

```kl
class Nodo:
    valor: i32
    siguiente: Nodo?

fn main():
    a = Nodo(1)
    b = Nodo(2)
    a.siguiente = b
    b.siguiente = a     # cycle → compiler inserts weak ref automatically

    # Both freed correctly when scope exits
```

---

## String Model

### Small String Optimization

```text
Strings <= 15 bytes: stack allocated
Strings > 15 bytes: heap allocated
```

### String Operations

```kl
name = "Hello"
greeting = name + " World"   # concatenation (new allocation)
first = greeting[0]          # indexing (bounds-checked)
length = len(greeting)       # length
```

All operations are bounds-checked at runtime (debug mode).

---

## Value Types vs Reference Types

### Value Types (Stack — Copy Semantics)

```kl
i8, i16, i32, i64
u8, u16, u32, u64
f32, f64
bool
char
struct (by default)
tuple (by default)
```

Value types are copied on assignment:

```kl
a: i32 = 10
b = a       # a is copied (two independent values)
```

### Reference Types (Heap — Move/Share Semantics)

```kl
class
list
dict
str (heap path, > 15 bytes)
```

Reference types share the reference on assignment:

```kl
a: list<i32> = [1, 2, 3]
b = a       # both point to the same list
b.add(4)    # a is also modified
```

### Explicit Clone

```kl
b = a.clone()  # deep copy
```

---

## Immutability

### Variables Are Immutable By Default

```kl
name = "John"
name = "Jane"  # ERROR: cannot modify immutable variable
```

### Mutable Variables (mut keyword)

```kl
mut name = "John"
name = "Jane"  # OK: variable is mutable
```

### Constants (UPPERCASE — Always Immutable)

Names in ALL_CAPS are compile-time constants. They cannot be modified and cannot be declared with `mut`:

```kl
PI = 3.141592
PI = 10          # Error: cannot modify constant

mut PI = 3.14    # Error: constants cannot be mutable
```

### Struct Field Mutability

```kl
class Punto:
    x: i32
    y: i32

fn main():
    p = Punto(10, 20)
    p.x = 5     # ERROR: p is immutable (default)

    mut p = Punto(10, 20)
    p.x = 5     # OK: mut p means all fields are mutable
```

Struct fields do not have individual mutability annotations. If the variable is `mut`, all fields are mutable. If not, all fields are immutable.

---

## Resource Management (RAII)

### File Example

```kl
fn leer_config() -> str:
    archivo = File.Open("config.kl")
    contenido = archivo.read()
    return contenido
    # ← archivo closes automatically (RAII destructor)
    # ← contenido is moved to caller (zero-cost)
```

### Lock Example

```kl
mut datos = [1, 2, 3]
bloqueo = Mutex.lock(datos)
bloqueo.add(4)
# ← bloqueo releases automatically when scope exits
```

### Network Connection Example

```kl
fn fetch_url(url: str) -> str:
    conn = TcpConnection(url)
    respuesta = conn.request()
    return respuesta
    # ← conn closes automatically (RAII destructor)
```

---

## Comparison: Kyle vs Rust

| Concept | Rust | Kyle |
|---------|------|----|
| Default mutability | Immutable (`let`) | Immutable |
| Mutable syntax | `let mut x` | `mut x` |
| Ownership annotations | Required (`&`, `&mut`, `Box`, `Rc`) | None (compiler infers) |
| Borrow checker | Yes (strict rules) | No (refcounting for shared) |
| Lifetimes | `'a`, `'b` | Not needed |
| Move semantics | Explicit (`x` vs `&x`) | Automatic (single-use = move) |
| Shared ownership | `Rc::new()`, `Arc::new()` | Automatic refcounting |
| Destructors | `Drop` trait | Automatic RAII |
| Thread safety | Borrow checker + `Send`/`Sync` | Automatic atomic refcounting |
| Learning curve | Months | Days |

---

## Memory Safety Guarantees

### Compile-Time

```text
Type safety
Bounds checking (static where possible)
Null safety via Option<T>
Immutability enforcement
```

### Runtime

```text
Bounds checking (debug mode)
Reference counting safety
Type-safe downcasting
```

---

## Alignment

### Default Alignment

```text
i8:   1 byte
i16:  2 bytes
i32:  4 bytes
i64:  8 bytes
f32:  4 bytes
f64:  8 bytes
```

### Struct Alignment

```kl
struct Point:
    x: f64   # offset 0
    y: f64   # offset 8
    z: f64   # offset 16
```

Compiler inserts padding as needed for alignment.

---

## Memory Model Version

```text
Kyle Memory Model Specification v2.0 — RAII + Compiler-Inferred Ownership
```
