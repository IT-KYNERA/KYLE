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

- [x] `fn main(args: [str]) -> i32:` declaration recognized as entry point
- [x] `args: [str]` is the list of command-line arguments
- [x] Return value is the process exit code (`0` = success, non-zero = failure)
- [x] `fn main() -> i32:` (no args) is also accepted
- [x] `klc-backend` generates a C-style `main` wrapper that calls the Kyle `main`

```kl
fn main(args: [str]) -> i32:
    println("Hello, World!")
    return 0
```

**Semantics:** The compiler wraps the Kyle `main` in a C `int main(int argc,
const char** argv)` function, marshals `argv` into a `[str]`, calls the
Kyle function, and returns its `i32` result to the OS.

---

## 1.2 Comments

- [x] `#` starts a line comment to end of line
- [x] `##` starts a documentation comment (doc comment), collected for `kl doc`
- [x] No block comments (planned: `-# ... #-`)

```kl
# This is a comment
## Documentation for the next declaration
x = 42   # trailing comment
```

---

# 2. Variables & Mutability

Kyle has three forms of binding: **immutable variable**, **mutable variable**,
and **constant**. There is no `let`, `var`, `mut`, or `const` keyword. Instead,
the **assignment operator** itself signals the mutability:

| Form | Syntax | Mutability | Rebindable |
|---|---|---|---|
| Immutable | `name = value` | ❌ | ❌ |
| Mutable | `name := value` | ✅ | ✅ |
| Constant | `name ::= value` | ❌ | ❌ |

The operator itself is the declaration — no keyword needed.

## 2.1 Immutable Variables

- [x] `name = expr` declares an immutable variable
- [x] Re-assignment is a compile error
- [x] Type is inferred from the right-hand side

```kl
name = "Kyle"           # str, immutable
x = 42                  # i32, immutable
items = [1, 2, 3]       # [i32], immutable
```

## 2.2 Mutable Variables

`:=` (walrus operator) declares a mutable variable and assigns its initial
value. The variable can be re-assigned later (same type only).

- [x] `name := expr` declares a mutable variable
- [x] Re-assignment with `name = expr` is allowed
- [x] Type is fixed at declaration
- [x] `name := name + 1` (read current, write new) is common

```kl
count := 0
count = count + 1       # OK
count = "hello"         # ❌ compile error: type changed
```

**Why `:=` instead of `mut`:** The walrus operator makes mutability visually
distinct at a glance. It is consistent with other languages (Go, Pascal) and
eliminates a keyword from the language.

## 2.3 Constants

`::=` declares a compile-time constant. The value must be evaluable at compile
time (a literal, a `const fn` call, or an expression composed of these).

- [x] `NAME ::= expr` declares a compile-time constant
- [x] Value must be compile-time evaluable
- [x] No naming convention enforced by the compiler (UPPERCASE recommended by convention)

```kl
PI ::= 3.14159
MAX_RETRIES ::= 3
GREETING ::= "Hello"
secret_base_url ::= "https://api.example.com"   # valid, but convention is UPPERCASE
```

## 2.4 Explicit Type Annotations

- [x] `name: T = expr` / `name: T := expr` / `name: T ::= expr` annotates the type explicitly
- [x] Works for immutable, mutable, and constant forms
- [x] The annotation must match the inferred type, or a wider compatible one

```kl
x: i32 = 42
name: str := "Kyle"
PI: f64 ::= 3.14159
items: [i32] = [1, 2, 3]
```

## 2.5 Type Inference

- [x] Local variables are inferred from the initializer
- [x] Function parameters are inferred only if not annotated
- [x] Function return type is inferred if not annotated
- [x] Generics are inferred from call-site argument types

```kl
x = 42                 # i32
items = [1, 2, 3]      # [i32]
pi = 3.14              # f64
```

## 2.6 Destructuring Declaration

A destructuring declaration unpacks a compound value into multiple variables
in one line. No `let` keyword — the syntax is direct.

- [ ] Tuple destructuring: `(x, y) = expr` — ❌ not implemented
- [ ] `(x, y) = (1, "hi")` unpacks into `x = 1`, `y = "hi"`
- [ ] Works with any value that has `.0`, `.1`, ... accessors

```kl
(x, y) = (1, "hello")   # x = 1, y = "hello"
(first, second) = items  # destructure a tuple return
```

---

# 3. Types

## 3.1 Primitive Types

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
| `str` | 8 bytes (ptr+len) | `"hello"` | Null-terminated UTF-8 |
| `char` | 1 byte | `'a'` | Single ASCII byte |
| `void` | 0 bytes | (no value) | Return type only |
| `ptr` | 8 bytes | `null` | Opaque pointer (raw memory address) |

**Semantics:** Strings are null-terminated UTF-8 byte arrays, stored as
`*const u8` in the LLVM IR. Length is computed by `kl_strlen` at runtime.

## 3.2 Composite Types

| Syntax | Type | Example |
|---|---|---|
| `[T]` | List of T | `[1, 2, 3]` |
| `{K: V}` | Dict with K keys, V values | `{"a": 1, "b": 2}` |
| `(T, U, ...)` | Tuple | `(1, "hi", 3.14)` |
| `T?` | Optional T | `i32?` means `i32 \| None` |
| `T!` | Error-returning T | `i32!` = `Result<T, Error>` |
| `final class Name { ... }` | Inline struct type | (structural typing) |

## 3.3 Optional Type `T?`

`T?` is sugar for `Option<T>`. It means "either a value of type `T` or
`None`". This is the **only** way to express optional values — `Option<T>`
is not exposed as a public syntax.

- [ ] `T?` postfix syntax in type annotations — ❌ not implemented
- [ ] `None` is the absent value (from stdlib, always available)
- [ ] `some_value` is constructed implicitly by assigning a `T` to a `T?` variable

```kl
name: str? = None          # optional string, currently absent
age: i32? = 42             # age is an optional i32 containing 42
```

## 3.4 The `ptr` Type

- [ ] `ptr` is an opaque pointer (raw memory address) — ❌ not implemented
- [ ] No arithmetic on `ptr` without `unsafe:`
- [ ] Used for FFI with C libraries

```kl
p: ptr = null              # null pointer
```

## 3.5 Integer Literals

- [x] Decimal: `42`, `1_000_000`
- [x] Hexadecimal: `0xFF`, `0xDEAD_BEEF`
- [x] Binary: `0b1010`, `0b1100_0011`
- [~] Underscores as digit separators
- [~] Suffix annotation: `42i8`, `42u32`, `42i64`

```kl
x = 42           # i32
y = 0xFF         # i32
z = 0b1010       # i32
big = 1_000_000  # i32
b: u8 = 255      # u8
```

## 3.6 Float Literals

- [x] Decimal: `3.14`, `0.5`, `1.0`
- [~] Scientific: `1e10`, `2.5e-3`
- [x] Suffix annotation: `3.14f32`, `2.5f64`

```kl
pi = 3.14159          # f64
small: f32 = 0.5      # f32
huge = 1.0e100        # f64
```

## 3.7 String Literals

- [x] Double-quoted: `"hello"`, `"multi\nline"`
- [x] Escape sequences: `\n`, `\t`, `\r`, `\\`, `\"`, `\0`
- [x] String interpolation: `"Hello, {name}!"` (any expression in `{}`)

```kl
name = "World"
greeting = "Hello, {name}!"    # "Hello, World!"
escape = "Line1\nLine2"
quote = "She said \"hi\""
```

**Semantics:** Strings are null-terminated UTF-8 bytes. No length field
stored alongside the pointer at the value level.

## 3.8 Character Literals

- [x] Single-quoted: `'a'`, `'Z'`, `'0'`
- [x] Escape sequences: `'\n'`, `'\t'`, `'\\'`, `'\''`
- [x] ASCII only (one byte)

```kl
c = 'a'
newline = '\n'
```

## 3.9 Boolean Literals

- [x] `true` and `false` are keywords

```kl
ok = true
done = false
```

## 3.10 None Literal

- [x] `None` is the special absent value. It is available without any import.
- [x] `None` is the only value of the `None` type (`!` — uninhabited)
- [x] `None` is coerced to any `T?` type

```kl
name: str? = None
age: i32? = None
```

## 3.11 Null Literal

- [ ] `null` is the pointer value for `ptr` type — ❌ not implemented
- [ ] `null` is a literal, not a keyword (it is a name from `core`)
- [ ] Only assignable to `ptr` types

```kl
p: ptr = null
```

## 3.12 Explicit Cast `as`

- [ ] `expr as Type` performs an explicit type cast — ❌ not implemented
- [ ] Integer-to-integer: `x as i64` widens or narrows
- [ ] Float-to-integer: `3.14 as i32` truncates
- [ ] Integer-to-float: `42 as f64`
- [ ] Pointer-to-integer: `p as i64` (in `unsafe:`)
- [ ] Integer-to-pointer: `addr as ptr` (in `unsafe:`)

```kl
x: i64 = 42 as i64
y: i32 = 3.14 as i32   # 3 (truncation)
```

## 3.13 Type Check `is`

- [ ] `value is Type` returns `true` if the value has the given type — ❌ not implemented
- [ ] Used in match patterns: `x is str => ...`
- [ ] Useful for `T?` types: `x is None => ...`

```kl
if x is str:
    println("x is a string")
```

## 3.14 Integer Overflow

| Mode | Behavior |
|---|---|
| Debug (`kl run`) | Panics on overflow |
| Release (`kl build --release`) | Wrapping arithmetic (silent) |

```kl
x: i8 = 127
x = x + 1              # panics in debug, wraps to -128 in release
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

- [x] `+` works for `i32`/`i64`/`f32`/`f64` and `str + str` (concatenation)
- [x] `-`, `*`, `/`, `%` work for `i32`/`i64`/`f32`/`f64`
- [x] Integer division by zero panics
- [x] Float division by zero produces inf/nan
- [x] String concatenation with `+` allocates a new buffer

## 4.2 Comparison Operators

| Op | Meaning | Example | Status |
|---|---|---|---|
| `==` | equal | `a == b` | ✅ |
| `!=` | not equal | `a != b` | ✅ |
| `<` | less than | `a < b` | ✅ |
| `>` | greater than | `a > b` | ✅ |
| `<=` | less or equal | `a <= b` | ✅ |
| `>=` | greater or equal | `a >= b` | ✅ |

- [x] `==` and `!=` work for integers, floats, booleans, strings
- [x] `<`, `>`, `<=`, `>=` work for integers, floats, strings (lexicographic)
- [x] Returns a `bool`

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

- [x] `and`, `or` are short-circuit (right side not evaluated if result determined)
- [x] `not` has higher precedence than `and`/`or`

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

- [x] All bitwise operators work for `i32`, `i64`
- [x] All bitwise operators work for `u8`, `u16`, `u32`, `u64`
- [x] Bitwise operators on `f32`/`f64` are a compile error

```kl
mask = 0xFF & 0x0F       # 0x0F
shifted = 1 << 4         # 16
flipped = ~0             # all bits set
```

## 4.5 Assignment Operators

`=` is always reassignment (to an existing mutable variable). `:=` is the
declaration form for mutable variables (see §2.2). Compound forms:

| Op | Meaning | Example | Status |
|---|---|---|---|
| `=` | reassign | `x = 5` | ✅ |
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

- [x] All compound assignments work for the corresponding scalar types
- [x] Compound assignment on immutable is a compile error
- [x] `x op= y` is equivalent to `x = x op y`

## 4.6 Range Operator

| Form | Meaning | Example | Status |
|---|---|---|---|
| `start..end` | exclusive end | `0..5` → 0,1,2,3,4 | ✅ |
| `start..=end` | inclusive end | `0..=5` → 0,1,2,3,4,5 | ❌ |
| `start..<end` | exclusive end (alias) | `0..<5` → 0,1,2,3,4 | ❌ |
| `start..` | open-ended | `3..` → 3,4,... | ❌ |
| `..end` | start-open | `..3` → 0,1,2 | ❌ |
| `..` | full range | `..` → everything | ❌ |

```kl
for i in 0..5:
    println(i)         # 0, 1, 2, 3, 4
```

## 4.7 Spread Operator

- [x] `...expr` inside a list literal spreads its elements
- [x] `...expr` outside a list literal is a no-op (parsed but inert)
- [x] Works for both lists and dicts in literal context

```kl
a = [1, 2, 3]
b = [...a, 4, 5]       # [1, 2, 3, 4, 5]
```

## 4.8 Ternary Operator

- [x] `cond ? a : b` returns `a` if `cond` is true, else `b`
- [x] Right-associative for chained ternaries
- [x] Both branches must have the same type

```kl
status = age >= 18 ? "adult" : "minor"
```

## 4.9 Optional Chaining

- [x] `obj?.field` returns `None` if `obj` is `None`, else the field
- [x] `obj?.method()` returns `None` if `obj` is `None`, else the result
- [x] `?.` chains left-to-right

```kl
name = user?.name
greeting = user?.greet()?.upper()
```

- [ ] `?:` default operator (e.g. `user?.age ?: 0`) — ❌ not implemented

## 4.10 Error Propagation

- [x] `expr?` extracts the value from a `T?` or `T!`, returning the function
- [x] Only valid in functions with `T!` return type
- [x] Propagates the `None`/error case as the function's return

```kl
fn parse_num(s: str) -> i32!:
    n = int(s)?
    if n < 0:
        return error("negative")
    return n
```

## 4.11 Member Access

- [x] `obj.field` accesses a field
- [x] `obj.method()` calls a method
- [x] `obj.field = value` assigns a field
- [x] `obj[index]` indexes a list or dict
- [x] `obj[start..end]` slices a list

```kl
p = Point { x: 1, y: 2 }
p.x = 10
items = [1, 2, 3, 4, 5]
first = items[0]
slice = items[1..3]      # [2, 3]
```

## 4.12 Function Pointer Type

- [ ] `(T, U) -> V` is the type of a function that takes `T, U` and returns `V`
- [ ] Used for callbacks, closures, and higher-order functions

```kl
type Callback = (i32) -> str
fn apply(x: i32, f: (i32) -> i32) -> i32:
    return f(x)
```

## 4.13 Operator Overloading

- [ ] A class/struct can define `op_+(other)` etc. to overload operators — ❌ not implemented
- [ ] Overloadable: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `[]`, `()`

## 4.14 Precedence Table

From highest to lowest:

| Precedence | Operators | Associativity |
|---|---|---|
| 1 (highest) | `()` `[]` `.` `?.` | left |
| 2 | unary `-` `~` `not` `await` | right |
| 3 | `**` | right |
| 4 | `*` `/` `%` `*%` | left |
| 5 | `+` `-` `+%` `-%` | left |
| 6 | `..` `..=` `..<` `...` | left |
| 7 | `<<` `>>` | left |
| 8 | `&` | left |
| 9 | `^` | left |
| 10 | `\|` | left |
| 11 | `is` `as` | left |
| 12 | `==` `!=` `<` `>` `<=` `>=` | left |
| 13 | `and` | left |
| 14 | `or` | left |
| 15 | `?:` | right |
| 16 | `?` | right |
| 17 | `=`, `:=`, `+=`, `-=`, ... | right |
| 18 (lowest) | `,` | left |

---

# 5. Functions

## 5.1 Function Declaration

- [x] `fn name(params) -> RetType:` declares a function
- [x] `fn name(params):` (no return type) returns `void`
- [x] `fn name<T>(params) -> T:` declares a generic function
- [x] Function body is an indented block

```kl
fn add(a: i32, b: i32) -> i32:
    return a + b

fn greet(name: str):
    println("Hello, " + name)
```

## 5.2 Parameters

- [x] `name: type` declares a typed parameter
- [x] `name` (untyped) infers from the body
- [x] Parameters are **immutable** by default (like all bindings)
- [ ] Default values: `name: type = default` — ❌ not implemented
- [ ] Variadic: `...names: T` — ❌ not implemented

```kl
fn add(a: i32, b: i32) -> i32:
    return a + b
```

**Making a local mutable copy of a parameter:**
```kl
fn process(id: i32):
    id := id                  # local mutable copy
    id = id + 1
```

## 5.3 Return Values

- [x] `return expr` exits the function with a value
- [x] The last expression of a block is implicitly returned (if no `return`)
- [x] Reaching the end of a non-void function without `return` is a compile error

```kl
fn add(a: i32, b: i32) -> i32:
    return a + b

# equivalent (implicit return)
fn add(a: i32, b: i32) -> i32:
    a + b
```

## 5.4 Generic Functions

- [x] `<T>` declares a type parameter
- [x] `<T, U>` declares multiple type parameters
- [x] Generics are monomorphized (one specialized function per type combination)

```kl
fn identity<T>(x: T) -> T:
    return x

fn pair<T, U>(a: T, b: U) -> (T?, U?):
    return (a, b)              # returns a tuple (see §8.3)
```

## 5.5 Error-Returning Functions

- [x] `-> T!` declares a function that can return an error
- [x] Internally represented as `Option<T>` (None = error)
- [x] `return error(msg)` returns an error
- [x] `?` propagates `None` as the function's return

```kl
fn read_int() -> i32!:
    line = input("enter number: ")
    n = int(line)?
    return n
```

## 5.6 Async Functions (Expression-Form)

- [~] `async expr` spawns `expr` on a new thread
- [~] Returns a task handle (i64)
- [x] `await task` joins the thread and returns its result
- [ ] `async fn name():` form — ❌ not implemented (use `async <expr>` instead)

```kl
task = async compute_something()
result = await task
```

## 5.7 Const Functions

- [~] `const fn name():` declares a function callable at compile time
- [~] Only allowed in constant expressions
- [~] Body must use only const-allowed operations (literals, other const fns)
- [~] Real compile-time evaluation — 🔶 partial (type-checks only, not evaluated)

```kl
const fn double(x: i32) -> i32:
    return x * 2
```

## 5.8 Abstract Functions

- [ ] `abstract fn name():` declares an abstract function — ❌ not implemented
- [ ] Only `abstract class` exists; `abstract fn` is not yet a syntax

## 5.9 Function Visibility

- [x] `fn name():` — public (default, callable from anywhere)
- [x] `fn _name():` — protected (callable from same class and subclasses)
- [x] `fn __name():` — private (callable only from inside the declaring class)
- [x] The leading underscores are stripped from the stored name; you call
  `this._name()` and `this.__name()` without the prefixes
- [x] Calling a private method from outside the class is a compile error

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

- [x] `if cond:` opens a conditional
- [x] `elif cond:` continues the chain
- [x] `else:` is the final branch
- [x] `if` is a **statement only**, not an expression
- [x] For inline conditionals, use the ternary operator: `cond ? a : b`

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

## 6.2 If Let

- [ ] `if let pattern = expr:` destructures and conditionally executes — ❌ not implemented
- [ ] If the pattern matches, the bound variables are available inside the block
- [ ] Commonly used with `T?` types: `if let Some(v) = opt:`

```kl
if let Some(n) = parse_int(s):
    println("parsed {n}")
else:
    println("failed to parse")
```

## 6.3 While

- [x] `while cond:` loops while the condition is true
- [~] `while-else:` runs the `else` block if the loop completes without `break`

```kl
i := 0
while i < 10:
    println(i)
    i = i + 1
```

## 6.4 While Let

- [ ] `while let pattern = expr:` loops while the pattern matches — ❌ not implemented

```kl
while let Some(val) = iter.next():
    process(val)
```

## 6.5 For

- [x] `for var in iterable:` iterates over a list
- [x] `for var in start..end:` iterates a numeric range
- [~] `for-else:` runs the `else` block if the loop completes without `break`

```kl
for item in items:
    process(item)

for i in 0..10:
    println(i)
```

## 6.6 Loop

- [x] `loop:` is an infinite loop
- [x] Exit with `break`
- [x] Skip iteration with `continue`

```kl
i := 0
loop:
    i = i + 1
    if i > 10:
        break
```

## 6.7 Labeled Loops

- [ ] `'label:` marks a loop with a label — ❌ not implemented
- [ ] `break 'label` exits the labeled loop from nested loops
- [ ] `continue 'label` continues the labeled loop from nested loops

```kl
'outer:
for i in 0..10:
    for j in 0..10:
        if i * j > 50:
            break 'outer     # exits both loops
```

## 6.8 Break

- [x] `break` exits the innermost loop (or labeled loop)
- [x] `break value` exits with a value (used in for/while-else)

```kl
for x in items:
    if x == target:
        break
```

## 6.9 Continue

- [x] `continue` skips the rest of the current loop iteration and jumps to
  the next one (or the labeled loop's next iteration)
- [x] Useful for filtering out items without nested `if` blocks

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

- [x] `match value:` opens a pattern match
- [x] Arms are `pattern : body` (colon separator)
- [x] Patterns: literal, identifier binding, wildcard `_`, enum variant
- [ ] `1 | 2 :` or-patterns — ❌ not implemented
- [~] `if cond` guard — 🔶 partial (parsed, not used for filtering)
- [ ] `is type` is-type pattern — ❌ not implemented in lowering
- [x] Match as expression returns a value

```kl
match status:
    200 : "ok"
    404 : "not found"
    500 : "error"
    _   : "unknown"

# Match as expression
label = match x:
    0 : "zero"
    1 : "one"
    _ : "many"

# Or-patterns (planned)
match value:
    0 | 1 | 2 : "small"
    3 | 4 | 5 : "medium"
    _         : "large"

# Guard (planned)
match x:
    n if n > 0 : "positive"
    n if n < 0 : "negative"
    0          : "zero"

# Destructuring (planned)
match p:
    Point { x, y } : println("{x}, {y}")
    _              : println("unknown")
```

## 6.11 Defer

- [x] `defer expr` schedules the expression to run when the current scope exits
- [x] Multiple defers run in LIFO order (last-in, first-out)
- [x] Used for guaranteed cleanup (closing files, releasing locks, etc.)

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

## 6.12 Guard

- [x] `guard cond else: body` — if `cond` is true, continue; if false, run `body`
- [x] The `body` must return (or `break`/`continue` in a loop)
- [x] Used for "fail fast" validation at the top of a function

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

## 6.13 Unsafe

- [x] `unsafe:` marks a block as containing unsafe operations
- [x] Used for FFI calls, raw pointers, and other low-level work
- [x] The type-checker is more permissive inside the block
- [ ] FFI lowering inside `unsafe:` — ❌ not implemented (planned Phase 10)
- [ ] `alloc`/`free` outside `unsafe:` — ❌ not implemented

**Why:** `unsafe` makes dangerous code **explicit and searchable**. When a
security auditor searches for `unsafe` in your codebase, they find every place
that needs careful review.

**Planned syntax (Phase 10):**

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

## 7.1 Final Classes (Lightweight Structs)

`final class` defines a lightweight, heap-allocated struct that **cannot** be
inherited. This replaces what was `struct` in earlier versions — it is the
simplest form of a user-defined data type.

- [ ] `final class Name:` declares a final class (replaces `struct`) — ❌ not implemented
- [ ] `final class Name<T>:` declares a generic final class
- [ ] Fields are `name: type` declarations
- [ ] `Name { x: 1, y: 2 }` creates an instance via literal syntax
- [ ] `.field` accesses a field
- [ ] `.field = value` assigns a field
- [ ] Passed by reference (no copy overhead)

```kl
final class Point:
    x: i32
    y: i32

p = Point { x: 10, y: 20 }
println(p.x)            # 10
p.y = 30
```

**Why `final class` and not `struct`:** Kyle unifies all user-defined types
under `class`. The `final` modifier signals "no inheritance" — cleaner than
two distinct keywords (`struct` vs `class`). The `struct` keyword is kept as
an **alias** during migration but will be removed.

## 7.2 Generic Final Classes

- [ ] `final class Name<T>:` declares a generic final class — ❌ not implemented
- [ ] `Name<i32> { x: 0 }` instantiates with a concrete type
- [ ] Each instantiation is monomorphized

```kl
final class Box<T>:
    value: T

b = Box<i32> { value: 42 }
b = Box<str> { value: "hi" }
```

## 7.3 Enums

- [x] `enum Name:` declares an enum
- [x] Variants are `Name` (no payload) or `Name(T1, T2, ...)` (with payload)
- [x] `Name.Variant` constructs a value
- [x] Pattern-matched in `match`
- [ ] Methods on enums: `fn name():` inside `enum` — ❌ not implemented

```kl
enum Color:
    Red
    Green
    Blue

enum Result:
    Ok(i32)
    Err(str)

v = Result.Ok(42)

match v:
    Result.Ok(n) : println("got " + str(n))
    Result.Err(e) : println("error: " + e)
```

## 7.4 Classes

- [x] `class Name:` declares a class (supports inheritance)
- [x] `class Name(args):` declares a class with constructor parameters
- [x] `class Name: Parent` declares inheritance from `Parent`
- [x] `class Name: Contract` declares implementation of `Contract`
- [x] `class Name: Parent implements Contract` does both
- [x] Fields are declared inside the class
- [x] Methods are `fn name():` inside the class
- [x] `Name(args)` invokes the constructor
- [x] `instance.field` and `instance.method()` work
- [x] `this` refers to the current instance

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

## 7.5 Inheritance, Polymorphism & Super

- [x] `class Child: Parent` inherits from `Parent`
- [x] Child inherits all parent fields
- [x] Child inherits all parent methods
- [x] Child can override a parent method by re-declaring it
- [x] Method dispatch follows the inheritance chain at call time
- [ ] `super.method()` calls the parent's overridden method — ❌ not implemented

```kl
class Animal:
    fn speak():
        println("generic sound")

class Dog: Animal
    fn speak():
        println("Woof!")

    fn parent_speak():
        super.speak()           # calls Animal.speak()

a = Animal()
a.speak()        # "generic sound"
d = Dog()
d.speak()        # "Woof!"
```

## 7.6 Visibility on Class Members

- [~] `name: type` (no prefix) — public field/method
- [~] `_name: type` — protected field/method (same class + subclasses)
- [~] `__name: type` — private field/method (only inside the class)
- [~] The leading underscores are stripped from the stored name
- [~] You call `this._name` and `this.__name` without the prefixes
- [~] Private access from outside the class is a compile error
- [~] Protected access from outside the class hierarchy is a compile error

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

- [ ] `abstract class Name:` declares an abstract class (cannot be instantiated) — ❌ not implemented
- [x] `abs class Name:` is a temporary alias for `abstract class`
- [x] Subclasses must inherit and provide all methods
- [~] Abstract method enforcement — 🔶 partial (class can be abstract, but no
  abstract method marker is enforced on subclasses)

```kl
abstract class Shape:
    fn area() -> f64

class Circle: Shape
    radius: f64

    Circle(r: f64):
        this.radius = r

    fn area() -> f64:
        return 3.14159 * this.radius * this.radius
```

## 7.8 Static Methods

- [ ] `static fn name():` inside a class declares a static method — ❌ not implemented
- [ ] Called on the class itself: `ClassName.method()`, not on instances
- [ ] Cannot access `this` (no instance)
- [ ] Can access other static methods and constants

```kl
class MathUtils:
    static fn square(x: i32) -> i32:
        return x * x

    static fn cube(x: i32) -> i32:
        return square(x) * x      # calls static method

result = MathUtils.square(5)      # 25
```

## 7.9 Contracts

- [x] `contract Name:` declares a contract (interface)
- [x] Contracts list method signatures (no bodies)
- [x] `class X: Contract` declares that `X` implements the contract
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

## 7.10 Type Aliases

- [~] `type Alias = T` declares a type alias
- [~] `type Alias<T> = T<T>` declares a generic type alias

```kl
type IntList = [i32]
type StringMap = dict<str, str>
type Callback<T> = (T) -> void
```

## 7.11 Properties

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

- [x] `[1, 2, 3]` creates a list literal
- [x] `[1, 2, ...rest]` spreads another list
- [x] `items[i]` indexes
- [~] `items[i..j]` slices
- [x] `items[i] = val` assigns
- [x] `items.add(val)` appends (method)
- [x] `items.pop()` removes and returns the last
- [x] `items.len()` returns the length

```kl
items = [1, 2, 3]
items.add(4)              # [1, 2, 3, 4]
items.pop()               # 4; items is [1, 2, 3]
first = items[0]          # 1
slice = items[0..2]        # [1, 2]
```

## 8.2 Dicts

- [x] `{"key": value}` creates a dict literal
- [x] Keys must be strings
- [x] `dict["key"]` looks up
- [x] `dict["key"] = val` sets
- [x] `dict.len()` returns the number of entries

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

- [x] `(params) => expr` is a closure
- [x] `(params) =>\n  body` is a block-bodied closure
- [x] Captures variables by reference
- [x] First-class: can be passed, returned, stored
- [~] Type annotations on parameters: `(x: i32) => x * 2` — 🔶 partial

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

Kyle supports two forms of async:

| Form | Description | Status |
|---|---|---|
| `async expr` | Spawns an expression as a concurrent task | ✅ Current (thread-based) |
| `async fn name():` | Declares an async function (state machine) | ❌ Planned (work-stealing scheduler) |

## 10.1 Expression Form (Current)

- [x] `async <expr>` spawns the expression on a new thread
- [x] `await task` joins and returns the result
- [x] Tasks are returned as `i64` handles

```kl
task = async expensive_computation()
result = await task
```

## 10.2 Async Functions (Planned)

- [ ] `async fn name():` declares an async function — ❌ not implemented
- [ ] Uses a work-stealing scheduler (like Tokio) instead of OS threads
- [ ] The function body is compiled as a state machine
- [ ] `await` inside an `async fn` yields control to the scheduler

```kl
async fn fetch(url: str) -> str:
    response = await http.get(url)
    return response.body
```

---

# 11. Error Handling

Kyle has **no exceptions**. Errors are values.

## 11.1 The `T!` Return Type

- [x] `-> T!` declares a function that can return an error
- [x] Internally, this is `Option<T>` (None = error)
- [x] `return error("msg")` returns an error
- [x] `?` propagates the error from the calling function

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

- [x] `expr?` extracts the value from a `T?` or `T!`
- [x] If `None`/error, returns from the enclosing function with the same error
- [x] Only valid in functions with `T!` return type

```kl
fn read_file(path: str) -> str!:
    fd = open(path, 0)?
    content = read_str(fd, 4096)?
    close(fd)
    return content
```

## 11.3 The `T?` Type (Optional Values)

- [x] `T?` is the only public syntax for optional values (sugar for internal `Option<T>`)
- [x] `None` is the absent value
- [x] Assigning a `T` to a `T?` implicitly wraps it
- [x] Pattern-matched with `match` or accessed with `?.`

```kl
x: i32? = None
y: i32? = 42

match x:
    Some(n) : println("got " + str(n))
    None    : println("nothing")
```

**Note:** The `Option` enum exists internally, but the public syntax is `T?`.
Use `None` for the absent case; `Some(value)` wrapping is automatic when
assigning a value of type `T` to a `T?`.

## 11.4 Optional Chaining

- [~] `obj?.field` returns `None` if `obj` is `None`
- [~] `obj?.method()` returns `None` if `obj` is `None`

```kl
name = user?.name        # str or None
age = user?.age          # i32 or None
```

---

# 12. Imports & Modules

## 12.1 Import Forms

- [x] `import x` — imports the `x` module
- [x] `import path.to.module` — nested module path (maps to `path/to/module.kl`)
- [x] `from x import y` — imports `y` from module `x`
- [ ] `from x import y as z` — imports `y` as `z`
- [x] `import ~x` — relative import from current file

```kl
import io
import math
import collections.list           # nested path: collections/list.kl
from str import capitalize as cap
```

## 12.2 Module Resolution

- [x] Module name maps to a file `x.kl` in:
  1. The current file's directory
  2. The project's `src/` directory
  3. `cwd/std/`
  4. The compiler's bundled `std/`
- [x] Nested paths (e.g. `a.b.c`) map to `a/b/c.kl` relative to a module root
- [x] `~` prefix is replaced with the current file's directory

## 12.3 Visibility (Module-Level)

- [x] Module-level names are public by default
- [x] `_name` is protected (not re-exported)
- [x] `__name` is private (not importable)

---

# 13. String Operations

## 13.1 Concatenation

- [x] `s1 + s2` concatenates strings
- [x] `"prefix" + var` is a common pattern

```kl
greeting = "Hello, " + name + "!"
```

## 13.2 Interpolation

- [x] `"text {expr} text"` interpolates the expression
- [x] The expression is converted to string via `str()`

```kl
name = "World"
greeting = "Hello, {name}!"    # "Hello, World!"
x = 42
msg = "value is {x}"           # "value is 42"
```

## 13.3 Built-in String Functions

- [x] `len(s)` — character count
- [x] `to_upper(s)` — uppercase
- [x] `to_lower(s)` — lowercase
- [x] `trim(s)` — strip whitespace
- [x] `replace(s, old, new)` — replace all occurrences
- [x] `substr(s, start, count)` — substring
- [x] `char_at(s, i)` — character at index
- [x] `contains(s, sub)` — check if substring is present
- [x] `starts_with_str(s, prefix)` — std lib
- [x] `ends_with_str(s, suffix)` — std lib
- [x] `capitalize(s)` — std lib
- [x] `repeat_str(s, n)` — std lib

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
| `exit(code)` | `(i32) -> void` | Terminate process immediately | ❌ not implemented |
| `eprint(s)` | `(str) -> void` | Print to stderr | ❌ not implemented |
| `eprintln(s)` | `(str) -> void` | Print to stderr with newline | ❌ not implemented |
| `panic(msg)` | `(str) -> void` | Runtime panic with message | ❌ not implemented |
| `dbg(x)` | `(any) -> any` | Print expr + file:line, return value | ❌ not implemented |
| `sizeof(T)` | `(type) -> i32` | Size of a type in bytes | ❌ not implemented |
| `alignof(T)` | `(type) -> i32` | Alignment of a type | ❌ not implemented |
| `offset_of(T, field)` | `(type, str) -> i32` | Offset of a field in bytes | ❌ not implemented |

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
declaration        = import_decl | function_decl | class_decl
                   | enum_decl | contract_decl | type_alias
                   | variable_decl | const_decl | if_let_decl ;

import_decl        = "import" identifier { "." identifier }
                   | "from" identifier { "." identifier } "import" identifier [ "as" identifier ]
                   | "import" "~" identifier ;

function_decl      = [ "export" ] "fn" identifier [ "<" type_params ">" ]
                     "(" [ parameters ] ")" [ "->" type ] ":" block ;

class_decl         = [ "abstract" ] [ "final" ] "class" identifier [ "<" type_params ">" ]
                     [ "(" parameters ")" ] [ ":" identifier [ "implements" identifier ] ] ":" block ;

enum_decl          = "enum" identifier [ "<" type_params ">" ] ":" block ;

contract_decl      = "contract" identifier [ "<" type_params ">" ] ":" block ;

type_alias         = "type" identifier [ "<" type_params ">" ] "=" type ;

variable_decl      = identifier [ ":" type ] "=" expression ;
mutable_decl       = identifier [ ":" type ] ":=" expression ;
const_decl         = identifier [ ":" type ] "::=" expression ;

block              = NEWLINE INDENT { statement } DEDENT ;

statement          = variable_decl | mutable_decl | expression_stmt
                   | if_stmt | if_let_stmt | while_stmt | while_let_stmt
                   | for_stmt | match_stmt | return_stmt
                   | break_stmt | continue_stmt | defer_stmt | guard_stmt
                   | unsafe_block | loop_block ;

if_stmt            = "if" expression ":" block
                     { "elif" expression ":" block }
                     [ "else" ":" block ] ;

if_let_stmt        = "if" "let" pattern "=" expression ":" block
                     [ "else" ":" block ] ;

while_stmt         = "while" expression ":" block [ "else" ":" block ] ;
while_let_stmt     = "while" "let" pattern "=" expression ":" block ;
for_stmt           = "for" identifier "in" expression ":" block
                     [ "else" ":" block ] ;

match_stmt         = "match" expression ":" { match_arm } ;
match_arm          = pattern [ "if" expression ] ":" expression ;

return_stmt        = "return" [ expression ] ;
break_stmt         = "break" [ label ] [ expression ] ;
continue_stmt      = "continue" [ label ] ;
defer_stmt         = "defer" expression ;
guard_stmt         = "guard" expression ":" block [ "else" ":" block ] ;
unsafe_block       = "unsafe" ":" block ;
loop_block         = label ":" "loop" ":" block ;

label              = "'" identifier ;

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
                   | tuple_literal | closure | "(" expression ")"
                   | "as" type | "is" type ;

literal            = integer | float | string | char | "true" | "false"
                   | "None" | "null" ;

type               = primitive_type | user_type | generic_type
                   | optional_type | error_type | dict_type
                   | function_type | pointer_type ;

primitive_type     = "i8" | "i16" | "i32" | "i64"
                   | "u8" | "u16" | "u32" | "u64"
                   | "f32" | "f64" | "bool" | "str" | "char" | "void" | "ptr" ;

optional_type      = type "?" ;
error_type         = type "!" ;
function_type      = "(" [ type { "," type } ] ")" "->" type ;
pointer_type       = "ptr" ;
dict_type          = "{" type ":" type "}" ;

assign_op          = "=" | "+=" | "-=" | "*=" | "/=" | "%="
                   | "&=" | "|=" | "^=" | "<<=" | ">>=" ;

pattern            = literal | identifier | "_" | enum_variant pattern
                   | "(" pattern { "," pattern } ")"
                   | pattern "|" pattern
                   | pattern "if" expression ;
```

---

# 17. Test Matrix Summary

> Re-run this matrix whenever a release is cut. Tick the box for each item
> verified to compile, type-check, and run end-to-end through `kl run`.

## 17.1 Declarations

- [x] Immutable variable (`=`)
- [ ] Mutable variable (`:=`)
- [ ] Constant (`::=`)
- [x] Typed annotation
- [x] Type inference
- [ ] Destructuring declaration

## 17.2 Functions

- [x] Simple function
- [x] Generic function
- [~] Default-arg function
- [x] Error-returning function
- [x] Async (expression form)
- [ ] Async (function form)
- [~] Const function (compile-time)
- [ ] Abstract function

## 17.3 Control Flow

- [x] If / elif / else
- [x] While
- [~] While-else
- [x] For-in-list
- [x] For-in-range
- [~] For-else
- [x] Loop
- [x] Break / continue
- [x] Match (literal patterns)
- [x] Match (identifier binding)
- [x] Match (enum variant)
- [x] Match (wildcard)
- [ ] Match (or-pattern)
- [~] Match (guard)
- [ ] Match (is-type)
- [x] Match as expression
- [x] Defer
- [x] Guard
- [x] Unsafe block

## 17.4 Data Structures

- [ ] Final class (replaces `struct`)
- [ ] Generic final class
- [x] Enum (no payload)
- [x] Enum (with payload)
- [x] Class
- [x] Class with constructor args
- [x] Single inheritance
- [x] Method override (polymorphism)
- [~] Public / protected / private fields
- [x] Public / protected / private methods
- [ ] Abstract class (`abstract class`)
- [ ] Static methods
- [ ] Properties (get/set)
- [ ] `super` keyword
- [x] Contract declaration
- [x] Contract implementation
- [ ] Generic contract
- [~] Type alias

## 17.5 Types & Values

- [x] i8, i16, i32, i64
- [x] u8, u16, u32, u64
- [x] f32, f64
- [x] bool
- [x] str
- [x] char
- [x] void (return type)
- [ ] ptr (raw pointer)
- [ ] T? (optional type)
- [x] List literal
- [x] List indexing
- [~] List slicing
- [x] List spread
- [x] Dict literal (str keys)
- [x] Dict indexing
- [ ] Tuple

## 17.6 Operators

- [x] `+`, `-`, `*`, `/`, `%` arithmetic
- [~] `**` power
- [~] `+%`, `-%`, `*%` percent
- [x] `==`, `!=`, `<`, `>`, `<=`, `>=` comparison
- [x] `and`, `or`, `not` logical
- [x] `&`, `|`, `^`, `<<`, `>>`, `~` bitwise
- [x] `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=` assignment
- [x] `..` range
- [x] `...` spread
- [ ] `?:` ternary default — ❌
- [x] `?` error propagation
- [x] `?.` optional chain

## 17.7 Error Handling

- [x] `T!` return type
- [x] `error("msg")` constructor
- [x] `?` propagation
- [x] `Option<T>` type
- [~] `Some` / `None` constructors
- [~] `?.` chaining

## 17.8 Built-ins

- [x] `print` / `println`
- [x] `print_int` / `println_int`
- [~] `print_err`
- [x] `len`
- [x] `str()` conversion
- [x] `int()` conversion
- [x] `float()` conversion
- [x] `bool()` conversion
- [x] `input` (no args)
- [x] `input(prompt)`
- [x] `range(n)` / `range(start, end)`
- [x] `open` / `close` / `read_str` / `write_str`
- [x] `sleep` / `now`
- [x] `assert` / `assert_eq` / `assert_ne` / `assert_str`
- [x] `to_upper` / `to_lower` / `trim` / `replace` / `substr` / `char_at` / `contains` / `ord`
- [x] `is_digit` / `is_alpha` / `is_alnum` / `is_whitespace` / `is_upper` / `is_lower`
- [~] `ceil` / `floor` / `round` — ❌ no runtime impl
- [x] `json_parse` / `json_stringify`

## 17.9 Standard Library

- [~] `core` — `Option<T>`, `Some`, `None`, `unwrap_or`, `is_some`, `is_none`
- [~] `math` — `absolute`, `pow`, `sqrt`, `gcd`, `min`, `max`, `clamp`
- [~] `io` — `read_file`, `write_file`
- [~] `str` — `starts_with_str`, `ends_with_str`, `capitalize`, `repeat_str`
- [~] `testing` — `assert`, `assert_eq`, `assert_str`, `assert_ne`
- [~] `collections` — `list_sum`, `list_product`, `list_max`, `list_min`, `list_range`
- [~] `json` — `parse`, `stringify`
- [~] `time` — `timestamp`, `sleep_ms`, `seconds_since`

## 17.10 Modules

- [x] `import x`
- [ ] `import x from y`
- [x] `from x import y`
- [ ] `from x import y as z`
- [x] `import ~x` (relative)

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
| Abstract classes | ❌ (`abs` → `abstract`) |
| Contracts | 🔶 (no generic constraints) |
| Closures | ✅ |
| Async / await | ✅ (expression form) / ❌ (async fn) |
| Error values (`T!`, `?`) | ✅ |
| `T?` optional type | ❌ (replaces public `Option<T>`) |
| Default-with (`?:`) | ❌ |
| Pattern matching | ✅ (literal, identifier, wildcard, enum) |
| Or-patterns (`a \| b`) | ❌ |
| Match guard (`if cond`) | 🔶 |
| Is-type pattern (`is T`) | ❌ |
| Destructuring (`(x, y) = expr`) | ❌ |
| `if let` / `while let` | ❌ |
| Labeled loops | ❌ |
| `super` keyword | ❌ |
| Properties (get/set) | ❌ |
| Static methods | ❌ |
| `final class` (replaces `struct`) | ❌ |
| `:=` mutable syntax | ❌ |
| `::=` constant syntax | ❌ |
| Operator overloading | ❌ |
| Move semantics (vs refcount) | ❌ |
| FFI (`extern "C"`) | ❌ (parsed, no codegen) |
| Compile-time evaluation (`const fn`) | 🔶 type-checks only |
| `Channel<T>` | ❌ |

---

*Version: v0.3.0 · Last updated: 2026-06-28*
