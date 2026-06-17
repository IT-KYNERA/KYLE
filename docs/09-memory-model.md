# KL Memory Model Specification v1.0

---

## Philosophy

KL manages memory automatically. The developer never manually allocates or frees memory.

```text
No malloc
No free
No manual memory management
No pointer arithmetic
Default memory safety
```

---

## Design Goals

```text
Safe by default
Predictable performance
No dangling pointers
No use-after-free
No double-free
No buffer overflows
Minimal runtime overhead
```

---

## Memory Regions

KL uses two memory regions:

```text
Stack
Heap
```

---

## Stack Allocation

Stack is used for:

```text
Local variables
Function parameters
Return addresses
Small fixed-size values
```

Stack allocation is:

```text
Fast (increment pointer)
Deterministic (LIFO order)
No fragmentation
No garbage collection needed
```

### Stack-Allocated Types

```kl
i8, i16, i32, i64
u8, u16, u32, u64
f32, f64
bool
char
```

### Example

```kl
fn compute():
    x: i32 = 10
    y: i32 = 20
    result = x + y
```

All variables live on the stack.

---

## Heap Allocation

Heap is used for:

```text
Dynamic objects
Lists
Dictionaries
Strings longer than 16 bytes
Closures capturing variables
Class instances
```

### Example

```kl
fn create_user() -> User:
    user = User("Anna", 30)
    return user
```

`User` is allocated on the heap.

---

## Garbage Collector

KL uses a **generational garbage collector**.

### Generation 0 (Young)

```text
Recently allocated objects
Collected frequently
Small pause times
```

### Generation 1 (Old)

```text
Objects surviving Gen 0
Collected infrequently
Larger pause times
```

### Collection Trigger

```text
Allocation threshold exceeded
Manual request (gc.collect())
Memory pressure detected
```

### GC Algorithm

```text
Phase 1: Mark
    Traverse from roots (stack, registers, globals)
    Mark all reachable objects

Phase 2: Sweep
    Scan heap
    Free unmarked objects
    Compact if needed
```

### GC Safety

```text
Objects are never moved while referenced
Pause times are bounded
GC is thread-safe
```

---

## Escape Analysis

The compiler performs escape analysis.

### Stack Allocation Optimization

If an object does not escape the function scope:

```kl
fn compute():
    buffer = [1, 2, 3]
    # buffer never leaves this function
    return len(buffer)
```

Compiler allocates `buffer` on the stack.

### Heap Allocation Required

If an object escapes:

```kl
fn create_list() -> list<i32>:
    buffer = [1, 2, 3]
    return buffer  # escapes! heap allocated
```

---

## Reference Counting

Future optimization:

```text
Selective reference counting
For small objects with clear ownership
Avoids GC pause for short-lived objects
```

Not in v1.0.

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
greeting = name + " World"   # concatenation
first = greeting[0]          # indexing
length = len(greeting)       # length
```

All operations are bounds-checked at runtime (debug mode).

---

## Value Types vs Reference Types

### Value Types (Stack)

```kl
i8, i16, i32, i64
u8, u16, u32, u64
f32, f64
bool
char
struct (by default)
tuple (by default)
```

### Reference Types (Heap)

```kl
class
list
dict
str (heap path)
```

---

## Copy Semantics

### Implicit Copy

Value types are copied on assignment:

```kl
a: i32 = 10
b = a  # a is copied
```

### Reference Copy

Reference types share the reference:

```kl
a: list<i32> = [1, 2, 3]
b = a  # both point to same list
b.add(4)  # a is also modified
```

### Explicit Clone

```kl
b = a.clone()  # deep copy
```

---

## Immutability

### Variables Are Mutable By Default

```kl
name = "John"
name = "Jane"  # allowed, variable is mutable
```

All lowercase or camelCase variables are mutable by default.

### Constants (UPPERCASE)

Names in ALL_CAPS are constants and cannot be reassigned:

```kl
PI = 3.141592
PI = 10  # Error: cannot modify constant
```

---

## Ownership Model

KL does not have a Rust-style ownership system in v1.0.

```text
No borrow checker
No lifetimes
No move semantics
```

The Garbage Collector handles all memory safety.

### Future: Optional Ownership

```text
Phase 2: Opt-in ownership annotations
Phase 3: Borrow checker for performance-critical code
Phase 4: Full ownership integration
```

---

## Memory Safety Guarantees

### Compile-Time

```text
Type safety
Bounds checking (static where possible)
Null safety via Option<T>
Use-after-move detection (future)
```

### Runtime

```text
Bounds checking (debug mode)
Garbage collection
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

## Thread Safety

### Default

```text
Immutable by default
Thread-safe GC
Atomic operations for primitives
```

### Shared State

```kl
counter = AtomicI32(0)
counter.add(1)
value = counter.load()
```

### Mutex

```kl
data = Mutex<list<i32>>()
lock = data.lock()
lock.add(10)
```

---

## Memory Model Version

```text
KL Memory Model Specification v1.0
```
