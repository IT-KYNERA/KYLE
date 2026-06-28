# Language Reference

> Complete syntax, semantics, and status of every construct in Kyle.

This document is the canonical reference for what the language looks like and
how each piece behaves. Every construct includes:

- **A short description** of what it does
- **The exact syntax** with a runnable example
- **Semantics** — what is true when the construct executes
- **Status** — whether the compiler accepts it today (✅ working, 🔶 partial,
  ❌ not yet implemented)

> **All checkboxes below are intentionally left unchecked.** They are the
> test matrix for the language. Tick each box as the corresponding construct
> is verified to compile, type-check, and run end-to-end through `kl run`.

---

## Legend

| Marker | Meaning |
|---|---|
| ✅ | Implemented, tested, working end-to-end |
| 🔶 | Parsed or partially implemented, behavior may be limited |
| ❌ | Not yet implemented (planned in a future phase) |
| `-` | Not applicable |

---

# 1. Program Structure

## 1.1 Entry Point

Every Kyle program has exactly one entry point, `main`, in `src/main.kl` for
project mode or the top of the file for single-file mode.

- [ ] `fn main(args: [str]) -> i32:` declaration recognized as entry point
- [ ] `args: [str]` is the list of command-line arguments
- [ ] Return value is the process exit code (`0` = success, non-zero = failure)
- [ ] `fn main() -> i32:` (no args) is also accepted
- [ ] `fn main(args: list<str>)` (generic-syntax list) is also accepted
- [ ] `klc-backend` generates a C-style `main` wrapper that calls the Kyle `main`

```kl
fn main(args: [str]) -> i32:
    println("Hello, World!")
    return 0
```

**Semantics:** The compiler wraps the Kyle `main` in a C `int main(int argc,
const char** argv)` function, marshals `argv` into a `list<str>`, calls the
Kyle function, and returns its `i32` result to the OS.

---

## 1.2 Comments

- [ ] `#` starts a line comment to end of line
- [ ] No block comments (planned: `-# ... #-`)

```kl
# This is a comment
x = 42   # trailing comment
```

---

# 2. Variables & Mutability

Kyle has three forms of binding: **immutable variable**, **mutable variable**,
and **constant**. There is no `let`, `var`, or `val` keyword — the convention
is:

| Form | Syntax | Mutability | Rebindable |
|---|---|---|---|
| Immutable | `name = value` | ❌ | ❌ |
| Mutable | `mut name = value` | ✅ | ✅ |
| Constant | `NAME = value` | ❌ | ❌ |

## 2.1 Immutable Variables

- [ ] `name = expr` declares an immutable variable
- [ ] Re-assignment is a compile error
- [ ] Type is inferred from the right-hand side

```kl
name = "Kyle"           # str, immutable
x = 42                  # i32, immutable
items = [1, 2, 3]       # [i32], immutable
```

## 2.2 Mutable Variables

- [ ] `mut name = expr` declares a mutable variable
- [ ] Re-assignment is allowed
- [ ] Type is fixed at declaration

```kl
mut count = 0
count = count + 1       # OK
count = "hello"         # ❌ compile error: type changed
```

## 2.3 Constants

- [ ] `UPPER_NAME = expr` declares a compile-time constant
- [ ] The name must be all uppercase (letters, digits, underscores)
- [ ] Value must be a literal (no function calls, no variables)

```kl
PI = 3.14159
MAX_RETRIES = 3
GREETING = "Hello"
```

## 2.4 Explicit Type Annotations

- [ ] `name: T = expr` annotates the type explicitly
- [ ] Works for both `mut` and immutable forms
- [ ] The annotation must match the inferred type, or a wider compatible one

```kl
x: i32 = 42
mut name: str = "Kyle"
items: [i32] = [1, 2, 3]
```

## 2.5 Type Inference

- [ ] Local variables are inferred from the initializer
- [ ] Function parameters are inferred only if not annotated
- [ ] Function return type is inferred if not annotated
- [ ] Generics are inferred from call-site argument types

```kl
x = 42                 # i32
items = [1, 2, 3]      # [i32]
pi = 3.14              # f64
```

---

# 3. Primitive Types

| Type | Size | Literal | Notes |
|---|---|---|---|
| `i8` | 1 byte signed | `127` | Integer, signed |
| `i16` | 2 bytes signed | `32767` | Integer, signed |
| `i32` | 4 bytes signed | `42` | Default integer |
| `i64` | 8 bytes signed | `9999999999` | 64-bit integer |
| `u8` | 1 byte unsigned | `255` | Byte, unsigned |
| `u16` | 2 bytes unsigned | `65535` | Unsigned |
| `u32` | 4 bytes unsigned | `4294967295` | Unsigned |
| `u64` | 8 bytes unsigned | `0xFFFFFFFFFFFFFFFF` | Unsigned |
| `f32` | 4 bytes float | `3.14` | Single-precision |
| `f64` | 8 bytes float | `3.14159265358979` | Double-precision |
| `bool` | 1 byte | `true` / `false` | Boolean |
| `str` | pointer + length | `"hello"` | Null-terminated UTF-8 bytes |
| `char` | 1 byte | `'a'` | Single byte (ASCII subset) |
| `void` | n/a | (no value) | Return type only |

## 3.1 Integer Literals

- [ ] Decimal: `42`, `1_000_000`
- [ ] Hexadecimal: `0xFF`, `0xDEAD_BEEF`
- [ ] Binary: `0b1010`, `0b1100_0011`
- [ ] Underscores as digit separators
- [ ] Suffix annotation: `42i8`, `42u32`, `42i64`

```kl
x = 42           # i32
y = 0xFF         # i32
z = 0b1010       # i32
big = 1_000_000  # i32
b: u8 = 255      # u8
```

## 3.2 Float Literals

- [ ] Decimal: `3.14`, `0.5`, `1.0`
- [ ] Scientific: `1e10`, `2.5e-3`
- [ ] Suffix annotation: `3.14f32`, `2.5f64`

```kl
pi = 3.14159          # f64
small: f32 = 0.5      # f32
huge = 1.0e100        # f64
```

## 3.3 String Literals

- [ ] Double-quoted: `"hello"`, `"multi\nline"`
- [ ] Escape sequences: `\n`, `\t`, `\r`, `\\`, `\"`, `\0`
- [ ] String interpolation: `"Hello, {name}!"` (any expression in `{}`)

```kl
name = "World"
greeting = "Hello, {name}!"    # "Hello, World!"
escape = "Line1\nLine2"
quote = "She said \"hi\""
```

**Semantics:** Strings are null-terminated UTF-8 byte arrays, stored as
`*const u8` in the LLVM IR. The length is computed by `kl_strlen` at
runtime; no length field is stored alongside the pointer.

## 3.4 Character Literals

- [ ] Single-quoted: `'a'`, `'Z'`, `'0'`
- [ ] Escape sequences: `'\n'`, `'\t'`, `'\\'`, `'\''`
- [ ] ASCII only (one byte)

```kl
c = 'a'
newline = '\n'
```

## 3.5 Boolean Literals

- [ ] `true` and `false` are keywords

```kl
ok = true
done = false
```

## 3.6 None Literal

- [ ] `None` is the value of the absent case in `Option<T>`

```kl
result: Option<i32> = None
```

---

# 4. Operators

## 4.1 Arithmetic Operators

| Op | Meaning | Example | Status |
|---|---|---|---|
| `+` | addition | `a + b` | ✅ |
| `-` | subtraction | `a - b` | ✅ |
| `*` | multiplication | `a * b` | ✅ |
| `/` | division | `a / b` | ✅ |
| `%` | remainder | `a % b` | ✅ |
| `**` | power | `a ** b` | 🔶 Parsed, lowered as multiplication (incorrect) |
| `+%` | `a + (a * b / 100)` | `x +% 10` | 🔶 Parsed, no semantic meaning |
| `-%` | `a - (a * b / 100)` | `x -% 10` | 🔶 Parsed, no semantic meaning |
| `*%` | `a * b / 100` | `x *% 10` | 🔶 Parsed, no semantic meaning |

- [ ] `+` works for `i32`/`i64`/`f32`/`f64` and `str + str` (concatenation)
- [ ] `-`, `*`, `/`, `%` work for `i32`/`i64`/`f32`/`f64`
- [ ] Integer division by zero panics
- [ ] Float division by zero produces inf/nan
- [ ] String concatenation with `+` allocates a new buffer

## 4.2 Comparison Operators

| Op | Meaning | Example | Status |
|---|---|---|---|
| `==` | equal | `a == b` | ✅ |
| `!=` | not equal | `a != b` | ✅ |
| `<` | less than | `a < b` | ✅ |
| `>` | greater than | `a > b` | ✅ |
| `<=` | less or equal | `a <= b` | ✅ |
| `>=` | greater or equal | `a >= b` | ✅ |

- [ ] `==` and `!=` work for integers, floats, booleans, strings
- [ ] `<`, `>`, `<=`, `>=` work for integers, floats, strings (lexicographic)
- [ ] Returns a `bool`

```kl
x = 5 == 5         # true
y = "a" < "b"      # true (lexicographic)
```

## 4.3 Logical Operators

| Op | Meaning | Example | Status |
|---|---|---|---|
| `and` | logical and | `a and b` | ✅ |
| `or` | logical or | `a or b` | ✅ |
| `not` | logical not | `not a` | ✅ |

- [ ] `and`, `or` are short-circuit (right side not evaluated if result determined)
- [ ] `not` has higher precedence than `and`/`or`

```kl
ok = x > 0 and x < 10
ready = not done
```

## 4.4 Bitwise Operators

| Op | Meaning | Example | Status |
|---|---|---|---|
| `&` | bitwise and | `a & b` | ✅ (i32/i64) |
| `\|` | bitwise or | `a \| b` | ✅ (i32/i64) |
| `^` | bitwise xor | `a ^ b` | ✅ (i32/i64) |
| `<<` | left shift | `a << 3` | ✅ (i32/i64) |
| `>>` | right shift | `a >> 3` | ✅ (i32/i64) |
| `~` | bitwise not | `~a` | ✅ (i32/i64) |

- [ ] All bitwise operators work for `i32`, `i64`
- [ ] All bitwise operators work for `u8`, `u16`, `u32`, `u64`
- [ ] Bitwise operators on `f32`/`f64` are a compile error

```kl
mask = 0xFF & 0x0F       # 0x0F
shifted = 1 << 4         # 16
flipped = ~0             # all bits set
```

## 4.5 Assignment Operators

| Op | Meaning | Example | Status |
|---|---|---|---|
| `=` | assign | `x = 5` | ✅ |
| `+=` | add-assign | `x += 1` | ✅ |
| `-=` | sub-assign | `x -= 1` | ✅ |
| `*=` | mul-assign | `x *= 2` | ✅ |
| `/=` | div-assign | `x /= 2` | ✅ |
| `%=` | mod-assign | `x %= 3` | ✅ |
| `&=` | and-assign | `x &= 0xFF` | ✅ |
| `\|=` | or-assign | `x \|= 0x10` | ✅ |
| `^=` | xor-assign | `x ^= 0xFF` | ✅ |
| `<<=` | shl-assign | `x <<= 2` | ✅ |
| `>>=` | shr-assign | `x >>= 2` | ✅ |

- [ ] All compound assignments work for the corresponding scalar types
- [ ] Compound assignment on immutable is a compile error
- [ ] `x op= y` is equivalent to `x = x op y`

## 4.6 Range Operator

- [ ] `start..end` creates a range (used inside `for`)
- [ ] `0..5` iterates 0, 1, 2, 3, 4
- [ ] `start..` is an open-ended range (end omitted) — 🔶 partial
- [ ] `..end` is a start-open range — 🔶 partial
- [ ] `..` alone is a full range — 🔶 partial

```kl
for i in 0..5:
    println(i)         # prints 0, 1, 2, 3, 4
```

## 4.7 Spread Operator

- [ ] `...expr` inside a list literal spreads its elements
- [ ] `...expr` outside a list literal is a no-op (parsed but inert)
- [ ] Works for both lists and dicts in literal context

```kl
a = [1, 2, 3]
b = [...a, 4, 5]       # [1, 2, 3, 4, 5]
```

## 4.8 Ternary Operator

- [ ] `cond ? a : b` returns `a` if `cond` is true, else `b`
- [ ] Right-associative for chained ternaries
- [ ] Both branches must have the same type

```kl
status = age >= 18 ? "adult" : "minor"
```

## 4.9 Optional Chaining

- [ ] `obj?.field` returns `None` if `obj` is `None`, else the field
- [ ] `obj?.method()` returns `None` if `obj` is `None`, else the result
- [ ] `?.` chains left-to-right

```kl
name = user?.name
greeting = user?.greet()?.upper()
```

- [ ] `?:` default operator (e.g. `user?.age ?: 0`) — ❌ not implemented

## 4.10 Error Propagation

- [ ] `expr?` extracts the value from an `Option<T>` or returns the function
- [ ] Only valid in functions with `T!` return type
- [ ] Propagates the `None` case as the function's return

```kl
fn parse_num(s: str) -> i32!:
    n = int(s)?
    if n < 0:
        return error("negative")
    return n
```

## 4.11 Member Access

- [ ] `obj.field` accesses a field
- [ ] `obj.method()` calls a method
- [ ] `obj.field = value` assigns a field
- [ ] `obj[index]` indexes a list or dict
- [ ] `obj[start..end]` slices a list

```kl
p = Point { x: 1, y: 2 }
p.x = 10
items = [1, 2, 3, 4, 5]
first = items[0]
slice = items[1..3]      # [2, 3]
```

## 4.12 Precedence Table

From highest to lowest:

| Precedence | Operators | Associativity |
|---|---|---|
| 1 (highest) | `()` `[]` `.` `?.` | left |
| 2 | unary `-` `~` `not` | right |
| 3 | `**` | right |
| 4 | `*` `/` `%` `*%` | left |
| 5 | `+` `-` `+%` `-%` | left |
| 6 | `..` `...` | left |
| 7 | `<<` `>>` | left |
| 8 | `&` | left |
| 9 | `^` | left |
| 10 | `\|` | left |
| 11 | `==` `!=` `<` `>` `<=` `>=` `is` | left |
| 12 | `and` | left |
| 13 | `or` | left |
| 14 | `?:` (planned) | right |
| 15 | `?` | right |
| 16 | `=`, `+=`, `-=`, ... | right |
| 17 (lowest) | `,` | left |

---

# 5. Functions

## 5.1 Function Declaration

- [ ] `fn name(params) -> RetType:` declares a function
- [ ] `fn name(params):` (no return type) returns `void`
- [ ] `fn name<T>(params) -> T:` declares a generic function
- [ ] Function body is an indented block

```kl
fn add(a: i32, b: i32) -> i32:
    return a + b

fn greet(name: str):
    println("Hello, " + name)
```

## 5.2 Parameters

- [ ] `name: type` declares a typed parameter
- [ ] `name` (untyped) infers from the body
- [ ] Default values: `name: type = default` — ❌ not implemented
- [ ] Variadic: `...names: T` — ❌ not implemented

```kl
fn add(a: i32, b: i32) -> i32:
    return a + b
```

## 5.3 Return Values

- [ ] `return expr` exits the function with a value
- [ ] The last expression of a block is implicitly returned (if no `return`)
- [ ] Reaching the end of a non-void function without `return` is a compile error

```kl
fn add(a: i32, b: i32) -> i32:
    return a + b

# equivalent
fn add(a: i32, b: i32) -> i32:
    a + b
```

## 5.4 Generic Functions

- [ ] `<T>` declares a type parameter
- [ ] `<T, U>` declares multiple type parameters
- [ ] Generics are monomorphized (one specialized function per type combination)

```kl
fn identity<T>(x: T) -> T:
    return x

fn pair<T, U>(a: T, b: U) -> (T, U):
    return (a, b)
```

## 5.5 Error-Returning Functions

- [ ] `-> T!` declares a function that can return an error
- [ ] Internally represented as `Option<T>` (None = error)
- [ ] `return error(msg)` returns an error
- [ ] `?` propagates `None` as the function's return

```kl
fn read_int() -> i32!:
    line = input("enter number: ")
    n = int(line)?
    return n
```

## 5.6 Async Functions (Expression-Form)

- [ ] `async expr` spawns `expr` on a new thread
- [ ] Returns a task handle (i64)
- [ ] `await task` joins the thread and returns its result
- [ ] `async fn name():` form — ❌ not implemented (use `async <expr>` instead)

```kl
task = async compute_something()
result = await task
```

## 5.7 Const Functions

- [ ] `const fn name():` declares a function callable at compile time
- [ ] Only allowed in constant expressions
- [ ] Body must use only const-allowed operations (literals, other const fns)
- [ ] Real compile-time evaluation — 🔶 partial (type-checks only, not evaluated)

```kl
const fn double(x: i32) -> i32:
    return x * 2
```

## 5.8 Abstract Functions

- [ ] `abs fn name():` declares an abstract function — ❌ not implemented
- [ ] Only `abs class` exists; `abs fn` is not yet a syntax

## 5.9 Function Visibility

- [ ] `fn name():` — public (default, callable from anywhere)
- [ ] `fn _name():` — protected (callable from same class and subclasses)
- [ ] `fn __name():` — private (callable only from inside the declaring class)
- [ ] The leading underscores are stripped from the stored name; you call
  `this._name()` and `this.__name()` without the prefixes
- [ ] Calling a private method from outside the class is a compile error

```kl
class Counter:
    fn __reset():
        this.count = 0

    fn increment():
        this.reset()         # OK: inside the class

c = Counter()
# c.reset()                  # ❌ compile error: private method
```

---

# 6. Control Flow

## 6.1 If / Elif / Else

- [ ] `if cond:` opens a conditional
- [ ] `elif cond:` continues the chain
- [ ] `else:` is the final branch
- [ ] `if` is a **statement only**, not an expression
- [ ] For inline conditionals, use the ternary operator: `cond ? a : b`

```kl
if x > 0:
    println("positive")
elif x < 0:
    println("negative")
else:
    println("zero")
```

**Note:** Kyle has only **one way** to do conditionals. Use `if` for blocks,
ternary (`?:`) for inline values. There is no "if as expression" syntax.

## 6.2 While

- [ ] `while cond:` loops while the condition is true
- [ ] `while-else:` runs the `else` block if the loop completes without `break`

```kl
mut i = 0
while i < 10:
    println(i)
    i = i + 1
```

## 6.3 For

- [ ] `for var in iterable:` iterates over a list
- [ ] `for var in start..end:` iterates a numeric range
- [ ] `for-else:` runs the `else` block if the loop completes without `break`

```kl
for item in items:
    process(item)

for i in 0..10:
    println(i)
```

## 6.4 Loop

- [ ] `loop:` is an infinite loop
- [ ] Exit with `break`
- [ ] Skip iteration with `continue`

```kl
mut i = 0
loop:
    i = i + 1
    if i > 10:
        break
```

## 6.5 Break

- [ ] `break` exits the innermost loop
- [ ] `break value` exits with a value (used in for/while-else)

```kl
for x in items:
    if x == target:
        break
```

## 6.6 Continue

- [ ] `continue` skips the rest of the current loop iteration and jumps to
  the next one
- [ ] Useful for filtering out items without nested `if` blocks

```kl
# Without continue (nested ifs):
for user in users:
    if user.is_active:
        if not user.is_banned:
            send_welcome_email(user)

# With continue (flat, easier to read):
for user in users:
    if not user.is_active:
        continue                  # skip inactive users
    if user.is_banned:
        continue                  # skip banned users
    send_welcome_email(user)      # only active, non-banned users
```

## 6.7 Match

- [ ] `match value:` opens a pattern match
- [ ] Arms are `pattern => body`
- [ ] Patterns: literal, identifier binding, wildcard `_`, enum variant
- [ ] `1 | 2 =>` or-patterns — ❌ not implemented
- [ ] `if cond` guard — 🔶 partial (parsed, not used for filtering)
- [ ] `is type` is-type pattern — ❌ not implemented in lowering
- [ ] Match as expression returns a value

```kl
match status:
    200 => "ok"
    404 => "not found"
    500 => "error"
    _   => "unknown"

# Match as expression
label = match x:
    0 => "zero"
    1 => "one"
    _ => "many"
```

## 6.8 Defer

- [ ] `defer expr` schedules the expression to run when the current scope exits
- [ ] Multiple defers run in LIFO order (last-in, first-out)
- [ ] Used for guaranteed cleanup (closing files, releasing locks, etc.)

**Why:** Without `defer`, cleanup code at the end of a function might not run
if an early return happens. `defer` guarantees cleanup runs no matter how the
function exits.

```kl
# Without defer — file leak if parse fails:
fn read_file(path: str) -> str!:
    fd = open(path, 0)?
    data = read_str(fd, 4096)?        # if this fails...
    close(fd)                          # ...this never runs! LEAK.
    return data

# With defer — cleanup always runs:
fn read_file(path: str) -> str!:
    fd = open(path, 0)?
    defer close(fd)                    # runs no matter what
    data = read_str(fd, 4096)?
    return data                        # close(fd) runs here
                                       # close(fd) also runs if read fails
```

**LIFO order example:**

```kl
fn example():
    defer println("first defer")      # runs 3rd
    defer println("second defer")     # runs 2nd
    defer println("third defer")      # runs 1st
    println("body runs")
# Output:
# body runs
# third defer
# second defer
# first defer
```

## 6.9 Guard

- [ ] `guard cond else: body` — if `cond` is true, continue; if false, run `body`
- [ ] The `body` must return (or `break`/`continue` in a loop)
- [ ] Used for "fail fast" validation at the top of a function

**Why:** Instead of writing `if not condition: return ...` everywhere, you
write a single `guard` at the top. The rest of the function can assume the
condition holds.

```kl
# Without guard — nested ifs:
fn process_order(order: Order):
    if not order.is_valid():
        return error("invalid order")
    if order.user == null:
        return error("no user")
    if order.items.len() == 0:
        return error("empty order")
    charge(order)

# With guard — flat, one validation per line:
fn process_order(order: Order) -> i32!:
    guard order.is_valid() else:
        return error("invalid order")
    guard order.user != null else:
        return error("no user")
    guard order.items.len() > 0 else:
        return error("empty order")
    # here we KNOW all three conditions hold
    return charge(order)
```

## 6.10 Unsafe

- [ ] `unsafe:` marks a block as containing unsafe operations
- [ ] Used for FFI calls, raw pointers, and other low-level work
- [ ] The type-checker is more permissive inside the block
- [ ] FFI lowering inside `unsafe:` — ❌ not implemented (planned Phase 9)
- [ ] `alloc`/`free` outside `unsafe:` — ❌ not implemented

**Why:** `unsafe` makes dangerous code **explicit and searchable**. When a
security auditor searches for `unsafe` in your codebase, they find every place
that needs careful review.

**Planned syntax (Phase 9):**

```kl
extern "C":
    fn malloc(size: i32) -> ptr
    fn free(p: ptr)

fn my_alloc(size: i32) -> ptr:
    unsafe:
        p = malloc(size)
        if p == null:
            return null
        # raw pointer operations go here
        return p
```

Until FFI is implemented, the `unsafe` keyword is parsed but has no
operational effect.

---

# 7. Data Structures

## 7.1 Structs

- [ ] `struct Name:` declares a struct
- [ ] `struct Name<T>:` declares a generic struct
- [ ] Fields are `name: type` declarations
- [ ] `Name { x: 1, y: 2 }` creates an instance
- [ ] `.field` accesses a field
- [ ] `.field = value` assigns a field
- [ ] Structs are passed by reference (no copy)

```kl
struct Point:
    x: i32
    y: i32

p = Point { x: 10, y: 20 }
println(p.x)            # 10
p.y = 30
```

## 7.2 Generic Structs

- [ ] `struct Name<T>:` declares a generic struct
- [ ] `Name<i32> { x: 0 }` instantiates with a concrete type
- [ ] Each instantiation is monomorphized

```kl
struct Box<T>:
    value: T

b = Box<i32> { value: 42 }
b = Box<str> { value: "hi" }
```

## 7.3 Enums

- [ ] `enum Name:` declares an enum
- [ ] Variants are `Name` (no payload) or `Name(T1, T2, ...)` (with payload)
- [ ] `Name.Variant` constructs a value
- [ ] Pattern-matched in `match`

```kl
enum Option<T>:
    Some(T)
    None

v: Option<i32> = Option.Some(42)

match v:
    Option.Some(n) => println("got " + str(n))
    Option.None    => println("nothing")
```

## 7.4 Classes

- [ ] `class Name:` declares a class
- [ ] `class Name(args):` declares a class with constructor parameters
- [ ] `class Name: Parent` declares inheritance from `Parent`
- [ ] `class Name: Contract` declares implementation of `Contract`
- [ ] `class Name: Parent implements Contract` does both
- [ ] Fields are declared inside the class
- [ ] Methods are `fn name():` inside the class
- [ ] `Name(args)` invokes the constructor
- [ ] `instance.field` and `instance.method()` work
- [ ] `this` refers to the current instance

```kl
class Counter:
    count: i32

    Counter(start: i32):
        this.count = start

    fn increment() -> i32:
        this.count = this.count + 1
        return this.count

c = Counter(10)
c.increment()
```

## 7.5 Inheritance & Polymorphism

- [ ] `class Child: Parent` inherits from `Parent`
- [ ] Child inherits all parent fields
- [ ] Child inherits all parent methods
- [ ] Child can override a parent method by re-declaring it
- [ ] Method dispatch follows the inheritance chain at call time

```kl
class Animal:
    fn speak():
        println("generic sound")

class Dog: Animal
    fn speak():
        println("Woof!")

a = Animal()
a.speak()        # "generic sound"
d = Dog()
d.speak()        # "Woof!"
```

## 7.6 Visibility on Class Members

- [ ] `name: type` (no prefix) — public field/method
- [ ] `_name: type` — protected field/method (same class + subclasses)
- [ ] `__name: type` — private field/method (only inside the class)
- [ ] The leading underscores are stripped from the stored name
- [ ] You call `this._name` and `this.__name` without the prefixes
- [ ] Private access from outside the class is a compile error
- [ ] Protected access from outside the class hierarchy is a compile error

```kl
class Bank:
    __balance: i32

    Bank(initial: i32):
        this.__balance = initial

    fn __recompute():
        this.__balance = this.__balance * 2

    fn deposit(amount: i32):
        this.recompute()              # OK: inside the class
        this.__balance = this.__balance + amount
```

## 7.7 Abstract Classes

- [ ] `abs class Name:` declares an abstract class
- [ ] Cannot be instantiated directly
- [ ] Subclasses must inherit and provide all methods
- [ ] Abstract method enforcement — 🔶 partial (class can be abstract, but no
  abstract method marker is enforced on subclasses)

```kl
abs class Shape:
    fn area() -> f64

class Circle: Shape
    radius: f64

    Circle(r: f64):
        this.radius = r

    fn area() -> f64:
        return 3.14159 * this.radius * this.radius
```

## 7.8 Contracts

- [ ] `contract Name:` declares a contract (interface)
- [ ] Contracts list method signatures (no bodies)
- [ ] `class X: Contract` declares that `X` implements the contract
- [ ] Generic contracts `contract Name<T>:` — ❌ not implemented
- [ ] `impl` keyword — ❌ not used; `class X: Contract` does the impl

```kl
contract Greeter:
    fn greet(name: str) -> str

class Person: Greeter
    name: str

    Person(name: str):
        this.name = name

    fn greet(name: str) -> str:
        return "Hello, " + name + ", I'm " + this.name
```

## 7.9 Type Aliases

- [ ] `type Alias = T` declares a type alias
- [ ] `type Alias<T> = T<T>` declares a generic type alias

```kl
type IntList = [i32]
type StringMap = dict<str, str>
type Callback<T> = (T) -> void
```

## 7.10 Properties

Properties are fields with custom **getter** and/or **setter** logic. The
caller uses normal field-access syntax (`obj.prop`), but the compiler inserts
calls to the getter or setter.

- [ ] `get:` defines a read accessor — ❌ not implemented
- [ ] `set:` defines a write accessor — ❌ not implemented
- [ ] `name: type` with no get/set is a normal field

**Planned syntax:**

```kl
class Account:
    __balance: i32

    Account(initial: i32):
        this.__balance = initial

    # Read-only computed property
    get is_overdrawn() -> bool:
        return this.__balance < 0

    # Read-write property with validation
    get balance() -> i32:
        return this.__balance

    set balance(value: i32):
        if value < 0:
            return                  # silently reject negative values
        this.__balance = value
```

**Usage** (when implemented):

```kl
a = Account(100)
println(a.balance)              # calls get → 100
a.balance = 50                  # calls set with value=50
a.balance = -1                  # set rejects, balance stays 50
println(a.is_overdrawn)         # calls get is_overdrawn → false
```

---

# 8. Collections

## 8.1 Lists

- [ ] `[1, 2, 3]` creates a list literal
- [ ] `[1, 2, ...rest]` spreads another list
- [ ] `items[i]` indexes
- [ ] `items[i..j]` slices
- [ ] `items[i] = val` assigns
- [ ] `items.add(val)` appends (method)
- [ ] `items.pop()` removes and returns the last
- [ ] `items.len()` returns the length

```kl
items = [1, 2, 3]
items.add(4)              # [1, 2, 3, 4]
items.pop()               # 4; items is [1, 2, 3]
first = items[0]          # 1
slice = items[0..2]        # [1, 2]
```

## 8.2 Dicts

- [ ] `{"key": value}` creates a dict literal
- [ ] Keys must be strings
- [ ] `dict["key"]` looks up
- [ ] `dict["key"] = val` sets
- [ ] `dict.len()` returns the number of entries

```kl
ages = {"alice": 30, "bob": 25}
ages["charlie"] = 35
alice_age = ages["alice"]
n = ages.len()            # 3
```

- [ ] Non-string keys — ❌ not supported (planned)

## 8.3 Tuples

- [ ] `(a, b, c)` creates a tuple
- [ ] Element access via `.0`, `.1`, `.2` — 🔶 partial (parsed, not fully tested)

```kl
t = (1, "hello", 3.14)
x = t.0                  # 1
```

---

# 9. Closures

- [ ] `(params) => expr` is a closure
- [ ] `(params) =>\n  body` is a block-bodied closure
- [ ] Captures variables by reference
- [ ] First-class: can be passed, returned, stored
- [ ] Type annotations on parameters: `(x: i32) => x * 2` — 🔶 partial

```kl
double = (x: i32) => x * 2
result = double(5)        # 10

# Block body
process = (item: str):
    return item.to_upper()

# As argument
items.map((x) => x * 2)
```

---

# 10. Async / Await

- [ ] `async <expr>` spawns the expression on a new thread
- [ ] `await task` joins and returns the result
- [ ] Tasks are returned as `i64` handles
- [ ] `async fn name():` — ❌ not implemented (use `async <expr>`)

```kl
task = async expensive_computation()
result = await task
```

---

# 11. Error Handling

Kyle has **no exceptions**. Errors are values.

## 11.1 The `T!` Return Type

- [ ] `-> T!` declares a function that can return an error
- [ ] Internally, this is `Option<T>` (None = error)
- [ ] `return error("msg")` returns an error
- [ ] `?` propagates the error from the calling function

```kl
fn parse(s: str) -> i32!:
    if s == "":
        return error("empty string")
    return int(s)?         # int() can fail; propagate

fn caller() -> i32!:
    n = parse("42")?       # propagate error if parse fails
    return n * 2
```

## 11.2 The `?` Operator

- [ ] `expr?` extracts the value from `Option<T>`
- [ ] If `None`, returns from the enclosing function with the same error
- [ ] Only valid in functions with `T!` return type

```kl
fn read_file(path: str) -> str!:
    fd = open(path, 0)?
    content = read_str(fd, 4096)?
    close(fd)
    return content
```

## 11.3 The `Option<T>` Type

- [ ] `Option<T>` is `enum Option { Some(T), None }`
- [ ] `Some(value)` constructs the present case
- [ ] `None` constructs the absent case
- [ ] Pattern-matched or accessed with `?.`

```kl
x: Option<i32> = Some(42)
y: Option<i32> = None

match x:
    Some(n) => println("got " + str(n))
    None    => println("nothing")
```

## 11.4 Optional Chaining

- [ ] `obj?.field` returns `None` if `obj` is `None`
- [ ] `obj?.method()` returns `None` if `obj` is `None`

```kl
name = user?.name        # str or None
age = user?.age          # i32 or None
```

---

# 12. Imports & Modules

## 12.1 Import Forms

- [ ] `import x` — imports the `x` module from the current project
- [ ] `import x from y` — imports `x` from the `y` package (alias)
- [ ] `from x import y` — imports `y` from module `x`
- [ ] `from x import y as z` — imports `y` as `z`
- [ ] `import ~x` — relative import from current file

```kl
import io
import math
from str import capitalize as cap
```

## 12.2 Module Resolution

- [ ] Module name maps to a file `x.kl` in:
  1. The current file's directory
  2. The project's `src/` directory
  3. `cwd/std/`
  4. The compiler's bundled `std/`
- [ ] `~` prefix is replaced with the current file's directory
- [ ] No nested module paths (single-segment names only)

## 12.3 Visibility (Module-Level)

- [ ] Module-level names are public by default
- [ ] `_name` is protected (not re-exported)
- [ ] `__name` is private (not importable)

---

# 13. String Operations

## 13.1 Concatenation

- [ ] `s1 + s2` concatenates strings
- [ ] `"prefix" + var` is a common pattern

```kl
greeting = "Hello, " + name + "!"
```

## 13.2 Interpolation

- [ ] `"text {expr} text"` interpolates the expression
- [ ] The expression is converted to string via `str()`

```kl
name = "World"
greeting = "Hello, {name}!"    # "Hello, World!"
x = 42
msg = "value is {x}"           # "value is 42"
```

## 13.3 Built-in String Functions

- [ ] `len(s)` — character count
- [ ] `to_upper(s)` — uppercase
- [ ] `to_lower(s)` — lowercase
- [ ] `trim(s)` — strip whitespace
- [ ] `replace(s, old, new)` — replace all occurrences
- [ ] `substr(s, start, count)` — substring
- [ ] `char_at(s, i)` — character at index
- [ ] `contains(s, sub)` — check if substring is present
- [ ] `starts_with_str(s, prefix)` — std lib
- [ ] `ends_with_str(s, suffix)` — std lib
- [ ] `capitalize(s)` — std lib
- [ ] `repeat_str(s, n)` — std lib

```kl
s = "  Hello, World!  "
up = to_upper(s)              # "  HELLO, WORLD!  "
trimmed = trim(s)             # "Hello, World!"
sub = substr(s, 2, 5)         # "Hello"
```

---

# 14. Built-in Functions

| Function | Signature | Description | Status |
|---|---|---|---|
| `print(s)` | `(str) -> void` | Print to stdout | ✅ |
| `println(s)` | `(str) -> void` | Print with newline | ✅ |
| `print_int(n)` | `(i32) -> void` | Print integer | ✅ |
| `println_int(n)` | `(i32) -> void` | Print integer with newline | ✅ |
| `print_err(s)` | `(str) -> void` | Print to stderr | 🔶 registered, no `kl_print_err` |
| `len(x)` | `([T]) -> i32` or `(str) -> i32` | Length of list or string | ✅ |
| `str(x)` | `(any) -> str` | Convert to string | ✅ (i64 only) |
| `int(s)` | `(str) -> i32!` | Parse string to integer | 🔶 registered, no runtime impl |
| `float(s)` | `(str) -> f64!` | Parse string to float | 🔶 registered, no runtime impl |
| `bool(x)` | `(any) -> bool` | Convert to boolean | 🔶 registered, no runtime impl |
| `input()` | `() -> str` | Read line from stdin | ✅ |
| `input(prompt)` | `(str) -> str` | Print prompt, read line | ✅ |
| `range(n)` | `(i32) -> [i32]` | Create range `[0, n)` | ✅ |
| `range(start, end)` | `(i32, i32) -> [i32]` | Create range `[start, end)` | 🔶 partial |
| `open(path, mode)` | `(str, i32) -> i32` | Open file, return fd | ✅ |
| `close(fd)` | `(i32) -> void` | Close file descriptor | ✅ |
| `read_str(fd, count)` | `(i32, i32) -> str` | Read bytes from fd | ✅ |
| `write_str(fd, s)` | `(i32, str) -> i32` | Write string to fd | ✅ |
| `sleep(ms)` | `(i32) -> void` | Sleep for ms milliseconds | ✅ |
| `now()` | `() -> i32` | Current unix timestamp (seconds) | ✅ |
| `assert(cond)` | `(bool) -> void` | Panic if false | ✅ |
| `assert_eq(a, b)` | `(any, any) -> void` | Panic if not equal | ✅ |
| `assert_ne(a, b)` | `(any, any) -> void` | Panic if equal | 🔶 registered, no runtime |
| `assert_str(a, b)` | `(str, str) -> void` | Panic if strings differ | ✅ |
| `to_upper(s)` | `(str) -> str` | Uppercase | ✅ |
| `to_lower(s)` | `(str) -> str` | Lowercase | ✅ |
| `trim(s)` | `(str) -> str` | Strip whitespace | ✅ |
| `replace(s, old, new)` | `(str, str, str) -> str` | Replace all | ✅ |
| `substr(s, start, count)` | `(str, i32, i32) -> str` | Substring | ✅ |
| `char_at(s, i)` | `(str, i32) -> char` | Char at index | ✅ |
| `contains(s, sub)` | `(str, str) -> bool` | Contains substring | ✅ |
| `ord(c)` | `(char) -> i32` | Char to ASCII code | ✅ |
| `is_digit(c)` | `(char) -> bool` | Is digit | ✅ |
| `is_alpha(c)` | `(char) -> bool` | Is letter | ✅ |
| `is_alnum(c)` | `(char) -> bool` | Is alphanumeric | ✅ |
| `is_whitespace(c)` | `(char) -> bool` | Is whitespace | ✅ |
| `is_upper(c)` | `(char) -> bool` | Is uppercase | ✅ |
| `is_lower(c)` | `(char) -> bool` | Is lowercase | ✅ |
| `ceil(f)` | `(f64) -> f64` | Round up | 🔶 registered, no runtime |
| `floor(f)` | `(f64) -> f64` | Round down | 🔶 registered, no runtime |
| `round(f)` | `(f64) -> f64` | Round to nearest | 🔶 registered, no runtime |
| `json_parse(s)` | `(str) -> dict<str, i64>` | Parse JSON object | ✅ (objects only) |
| `json_stringify(d)` | `(dict<str, i64>) -> str` | Stringify JSON object | ✅ (objects only) |

---

# 15. Standard Library (`std/`)

| Module | Functions | Status |
|---|---|---|
| `core` | `Option<T>`, `Some`, `None`, `unwrap_or`, `is_some`, `is_none` | ✅ |
| `math` | `absolute`, `absolute64`, `pow`, `sqrt`, `gcd`, `min`, `max`, `clamp` | ✅ |
| `io` | `read_file`, `write_file` | ✅ |
| `str` | `starts_with_str`, `ends_with_str`, `capitalize`, `repeat_str` | ✅ |
| `testing` | `assert`, `assert_eq`, `assert_str`, `assert_ne` | ✅ |
| `collections` | `list_sum`, `list_product`, `list_max`, `list_min`, `list_range` | ✅ |
| `json` | `parse`, `stringify` | ✅ |
| `time` | `timestamp`, `sleep_ms`, `seconds_since` | ✅ |

---

# 16. EBNF Grammar (Reference)

```ebnf
program            = { declaration } ;
declaration        = import_decl | function_decl | class_decl | struct_decl
                   | enum_decl | contract_decl | type_alias | variable_decl ;

import_decl        = "import" identifier [ "from" identifier ]
                   | "from" identifier "import" identifier [ "as" identifier ]
                   | "import" "~" identifier ;

function_decl      = [ "abs" ] "fn" identifier [ "<" type_params ">" ]
                     "(" [ parameters ] ")" [ "->" type ] ":" block ;

class_decl         = [ "abs" ] "class" identifier [ "<" type_params ">" ]
                     [ "(" parameters ")" ] [ ":" identifier ] ":" block ;

struct_decl        = "struct" identifier [ "<" type_params ">" ] ":" block ;

enum_decl          = "enum" identifier [ "<" type_params ">" ] ":" block ;

contract_decl      = "contract" identifier ":" block ;

type_alias         = "type" identifier [ "<" type_params ">" ] "=" type ;

variable_decl      = [ "mut" ] identifier [ ":" type ] "=" expression ;

block              = NEWLINE INDENT { statement } DEDENT ;

statement          = variable_decl | const_decl | expression_stmt | if_stmt
                   | while_stmt | for_stmt | match_stmt | return_stmt
                   | break_stmt | continue_stmt | defer_stmt | guard_stmt
                   | unsafe_block | loop_block ;

if_stmt            = "if" expression ":" block
                     { "elif" expression ":" block }
                     [ "else" ":" block ] ;

while_stmt         = "while" expression ":" block [ "else" ":" block ] ;
for_stmt           = "for" identifier "in" expression ":" block
                     [ "else" ":" block ] ;

match_stmt         = "match" expression ":" { match_arm } ;
match_arm          = pattern [ "if" expression ] "=>" expression ;

return_stmt        = "return" [ expression ] ;
break_stmt         = "break" [ expression ] ;
continue_stmt      = "continue" ;
defer_stmt         = "defer" expression ;
guard_stmt         = "guard" expression ":" block [ "else" ":" block ] ;
unsafe_block       = "unsafe" ":" block ;
loop_block         = "loop" ":" block ;

expression         = assignment_expr ;
assignment_expr    = ternary_expr
                   | ( identifier | member_access | index_expr ) assign_op expression ;

ternary_expr       = logical_or [ "?" expression ":" ternary_expr ] ;
logical_or         = logical_and [ "or" logical_and ] ;
logical_and        = bitwise_or [ "and" bitwise_or ] ;
bitwise_or         = bitwise_xor [ "|" bitwise_xor ] ;
bitwise_xor        = bitwise_and [ "^" bitwise_and ] ;
bitwise_and        = equality [ "&" equality ] ;
equality           = comparison [ ("==" | "!=") comparison ] ;
comparison         = shift [ ("<" | ">" | "<=" | ">=") shift ] ;
shift              = additive [ ("<<" | ">>") additive ] ;
additive           = multiplicative [ ("+" | "-" | "+%" | "-%") multiplicative ] ;
multiplicative     = unary [ ("*" | "/" | "%" | "*%") unary ] ;
unary              = [ "-" | "~" | "not" | "await" ] power ;
power              = postfix [ "**" power ] ;
postfix            = primary { ("." identifier | "[" expression "]" | "(" args ")"
                                | "?" | "?." identifier | "..." ) } ;

primary            = literal | identifier | list_literal | dict_literal
                   | tuple_literal | closure | "(" expression ")" ;

literal            = integer | float | string | char | "true" | "false" | "None" ;

type               = primitive_type | user_type | generic_type
                   | optional_type | error_type | dict_type ;

primitive_type     = "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64"
                   | "f32" | "f64" | "bool" | "str" | "char" | "void" ;
```

---

# 17. Test Matrix Summary

> Re-run this matrix whenever a release is cut. Tick the box for each item
> verified to compile, type-check, and run end-to-end through `kl run`.

## 17.1 Declarations

- [ ] Immutable variable
- [ ] Mutable variable
- [ ] Constant
- [ ] Typed annotation
- [ ] Type inference

## 17.2 Functions

- [ ] Simple function
- [ ] Generic function
- [ ] Default-arg function
- [ ] Error-returning function
- [ ] Async (expression form)
- [ ] Const function (compile-time)
- [ ] Abstract function
- [ ] Public / protected / private visibility

## 17.3 Control Flow

- [ ] If / elif / else
- [ ] While
- [ ] While-else
- [ ] For-in-list
- [ ] For-in-range
- [ ] For-else
- [ ] Loop
- [ ] Break / continue
- [ ] Match (literal patterns)
- [ ] Match (identifier binding)
- [ ] Match (enum variant)
- [ ] Match (wildcard)
- [ ] Match (or-pattern)
- [ ] Match (guard)
- [ ] Match (is-type)
- [ ] Match as expression
- [ ] Defer
- [ ] Guard
- [ ] Unsafe block

## 17.4 Data Structures

- [ ] Struct
- [ ] Generic struct
- [ ] Enum (no payload)
- [ ] Enum (with payload)
- [ ] Class
- [ ] Class with constructor args
- [ ] Single inheritance
- [ ] Method override (polymorphism)
- [ ] Public / protected / private fields
- [ ] Public / protected / private methods
- [ ] Abstract class
- [ ] Contract declaration
- [ ] Contract implementation
- [ ] Generic contract
- [ ] Type alias

## 17.5 Types & Values

- [ ] i8, i16, i32, i64
- [ ] u8, u16, u32, u64
- [ ] f32, f64
- [ ] bool
- [ ] str
- [ ] char
- [ ] void (return type)
- [ ] any
- [ ] List literal
- [ ] List indexing
- [ ] List slicing
- [ ] List spread
- [ ] Dict literal (str keys)
- [ ] Dict indexing
- [ ] Tuple

## 17.6 Operators

- [ ] `+`, `-`, `*`, `/`, `%` arithmetic
- [ ] `**` power
- [ ] `+%`, `-%`, `*%` percent
- [ ] `==`, `!=`, `<`, `>`, `<=`, `>=` comparison
- [ ] `and`, `or`, `not` logical
- [ ] `&`, `|`, `^`, `<<`, `>>`, `~` bitwise
- [ ] `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=` assignment
- [ ] `..` range
- [ ] `...` spread
- [ ] `?:` ternary default — ❌
- [ ] `?` error propagation
- [ ] `?.` optional chain

## 17.7 Error Handling

- [ ] `T!` return type
- [ ] `error("msg")` constructor
- [ ] `?` propagation
- [ ] `Option<T>` type
- [ ] `Some` / `None` constructors
- [ ] `?.` chaining

## 17.8 Built-ins

- [ ] `print` / `println`
- [ ] `print_int` / `println_int`
- [ ] `print_err`
- [ ] `len`
- [ ] `str()` conversion
- [ ] `int()` conversion
- [ ] `float()` conversion
- [ ] `bool()` conversion
- [ ] `input` (no args)
- [ ] `input(prompt)`
- [ ] `range(n)` / `range(start, end)`
- [ ] `open` / `close` / `read_str` / `write_str`
- [ ] `sleep` / `now`
- [ ] `assert` / `assert_eq` / `assert_ne` / `assert_str`
- [ ] `to_upper` / `to_lower` / `trim` / `replace` / `substr` / `char_at` / `contains` / `ord`
- [ ] `is_digit` / `is_alpha` / `is_alnum` / `is_whitespace` / `is_upper` / `is_lower`
- [ ] `ceil` / `floor` / `round` — ❌ no runtime impl
- [ ] `json_parse` / `json_stringify`

## 17.9 Standard Library

- [ ] `core` — `Option<T>`, `Some`, `None`, `unwrap_or`, `is_some`, `is_none`
- [ ] `math` — `absolute`, `pow`, `sqrt`, `gcd`, `min`, `max`, `clamp`
- [ ] `io` — `read_file`, `write_file`
- [ ] `str` — `starts_with_str`, `ends_with_str`, `capitalize`, `repeat_str`
- [ ] `testing` — `assert`, `assert_eq`, `assert_str`, `assert_ne`
- [ ] `collections` — `list_sum`, `list_product`, `list_max`, `list_min`, `list_range`
- [ ] `json` — `parse`, `stringify`
- [ ] `time` — `timestamp`, `sleep_ms`, `seconds_since`

## 17.10 Modules

- [ ] `import x`
- [ ] `import x from y`
- [ ] `from x import y`
- [ ] `from x import y as z`
- [ ] `import ~x` (relative)

## 17.11 Tooling

- [ ] `kl new <name>`
- [ ] `kl run`
- [ ] `kl build`
- [ ] `kl check`
- [ ] `kl parse`
- [ ] `kl mir`
- [ ] `kl fmt`
- [ ] `kl test`
- [ ] `kl add` / `kl remove`
- [ ] `kl lsp`
- [ ] LSP — completion
- [ ] LSP — hover
- [ ] LSP — go-to-definition
- [ ] LSP — semantic tokens
- [ ] LSP — rename
- [ ] VS Code extension — syntax highlighting
- [ ] VS Code extension — snippets
- [ ] VS Code extension — semantic coloring

---

# 18. Status Summary

| Construct | Status |
|---|---|
| Variables (immutable, mut, const) | ✅ |
| Type inference | ✅ |
| Structs (generic) | ✅ |
| Enums (generic) | ✅ |
| Classes (inheritance, polymorphism) | ✅ |
| Public / protected / private visibility | ✅ |
| Methods (public, private, protected) | ✅ |
| Abstract classes | 🔶 (no `abs fn` enforcement) |
| Contracts | 🔶 (no generic constraints) |
| Closures | ✅ |
| Async / await | ✅ (expression form) |
| Error values (`T!`, `?`) | ✅ |
| Option chaining (`?.`) | ✅ |
| Default-with (`?:`) | ❌ |
| Pattern matching | ✅ (literal, identifier, wildcard, enum) |
| Or-patterns (`a \| b`) | ❌ |
| Match guard (`if cond`) | 🔶 |
| Is-type pattern (`is T`) | ❌ |
| Properties (get/set) | ❌ |
| FFI (`extern "C"`) | ❌ (parsed, no codegen) |
| `async fn name():` declaration | ❌ |
| Compile-time evaluation (`const fn`) | 🔶 type-checks only |
| `Channel<T>` | ❌ |

---

*Version: v0.2.2 · Last updated: 2026-06-27*
