# Kyle Programming Language Specification v1.0

## Philosophy

Kyle is a compiled programming language designed around:

- Simplicity
- Readability
- Strong Static Typing
- Type Inference
- High Performance
- Explicit Error Handling
- Modern Tooling
- LLVM Backend

## Design Goals

- No self
- No let
- No var

- No semicolons
- No braces for blocks
- No exceptions
- No hidden magic
- Maximum readability
- `this` for instance reference only

---

# Entry Point

Every executable Kyle program must have a `main` function in the root of `src/`:

```kl
fn main():
    print("Hello from Kyle")
```

With arguments and exit code:

```kl
fn main(args: [str]) -> i32:
    print("arg count: " + str(len(args)))
    return 0
```

The entry point is located in `src/main.kl`. A return value of `0` indicates success; non-zero indicates an error.

---

# File Extension

```text
.kl
```

Examples:

```text
main.kl
user.kl
auth.kl
```

---

# Blocks

All code blocks use:

```kl
:
```

Examples:

```kl
if age >= 18:

    print("Adult")
```

```kl
fn greet():
```

```kl
class User:
```

```kl
while running:
```

```kl
match result:
```

---

# Primitive Types

```kl
i8
i16
i32
i64

u8
u16
u32
u64

f32
f64

bool

char

str
```

## Char Literals

Single-quoted characters with optional escape sequences:

```kl
c = 'x'
newline = 'n'     # newline character (same as \n in strings)
tab = 't'         # tab character
backslash = '\\'  # backslash
quote = '\''      # single quote
```

Supported escape sequences match string escapes: `\n`, `\t`, `\r`, `\\`, `\'`, `\0`.

---

# Optional Types

```kl
name: Option<str>

user: Option<User>
```

Rule:

```text
Option<T>
indicates that a value may be absent.
```

---

# Variables

## Type Inference

```kl
name = "John"

age = 25

salary = 1000.50

active = true
```

---

## Explicit Types

```kl
name: str = "John"

age: i32 = 25

salary: f64 = 1000.50
```

---

# Constants

Constants are declared with UPPERCASE names.

Declaration:

```kl
PI = 3.141592

API_URL = "https://api.company.com"

MAX_USERS = 1000
```

Usage:

```kl
radius = 10

area = PI * radius * radius
```

Rule:

```text
Any name in UPPERCASE (ALL_CAPS) is treated as a constant.

Constants cannot be reassigned after declaration.

Variables in lowercase or camelCase are immutable by default. Use the `mut` keyword to make a variable mutable.

```kl
mut total = 0       # mutable variable
nombre = "Ana"      # immutable variable
```
```

---

# Operators

## Arithmetic

```kl
+
-
*
/
%

**
```

Example:

```kl
result = (a + b) * 2
```

---

## Bitwise

```kl
&     # bitwise AND
|     # bitwise OR
^     # bitwise XOR
~     # bitwise NOT
<<    # left shift
>>    # right shift (arithmetic for signed, logical for unsigned)
```

Example:

```kl
flags = READ | WRITE
masked = flags & ~EXECUTE
shifted = value << 4
```

---

## Comparison

```kl
==
!=

>
<
>=
<=
```

Example:

```kl
if age >= 18:

    print("Adult")
```

---

## is (Type Check)

```kl
is        # runtime type check
is not    # negated type check
```

Example:

```kl
if value is Admin:
    print("Admin access")
```

---

## Logical

```kl
&&

||

!
```

Example:

```kl
if active && verified:

    print("Access granted")
```

---

## Assignment

```kl
=

+=
-=

*=
/=

%=

&=
|=
^=
<<=
>>=
```

Example:

```kl
count += 1
flags |= READ
value <<= 4
```

---

# Functions

```kl
fn greet(name: str) -> str:

    return "Hello " + name
```

---

# Default Parameters

```kl
fn greet(name: str, greeting: str = "Hello"):

    print(greeting + " " + name)
```

Call:

```kl
greet("Juan")           # Hello Juan
greet("Juan", "Hola")   # Hola Juan
```

Default values must be constants or literals (evaluated at compile time).

---

# Named Arguments

Pass arguments by name, in any order:

```kl
fn configure(host: str, port: i32, ssl: bool):

    ...

configure(
    host: "localhost",
    ssl: true,
    port: 443
)
```

Named arguments can be mixed with positional, but positional must come first.

---

# Variadic Functions

Accept zero or more trailing arguments:

```kl
fn log(level: str, ...messages: str):

    for msg in messages:

        print(level + ": " + msg)
```

Call:

```kl
log("INFO", "started", "processing", "done")
```

The variadic parameter is always the last parameter and is typed as `[T]` (a list) inside the function body.

---

# Void Functions

```kl
fn save():

    print("Saved")
```

---

# Closures

```kl
double = (x) => x * 2
```

```kl
sum = (a, b) => a + b
```

```kl
evens = numbers.filter(
    (x) => x % 2 == 0
)
```

---

# Attributes / Annotations

Metadata attached to declarations:

```kl
#[deprecated("use new_api instead")]
fn old_api():
    ...

#[allow(dead_code)]
fn helper():
    ...

#[test]
fn test_math():
    assert_eq(2 + 2, 4)

#[inline]
fn fast_path():
    ...
```

Attributes appear before the declaration on a separate line, starting with `#[`

---

# String Literals

Double-quoted strings:

```kl
name = "Hello"
```

## Escape Sequences

String literals support these escape sequences:

| Sequence | Meaning |
|----------|---------|
| `\n`     | Newline |
| `\t`     | Tab |
| `\r`     | Carriage return |
| `\\`     | Backslash |
| `\"`     | Double quote |
| `\{`     | Opening brace (escaped interpolation) |
| `\0`     | Null character |

```kl
text = "Line1\nLine2\tindented"
path = "C:\\Users\\name"
```

# String Interpolation

Embed expressions inside double-quoted strings with `{}`:

```kl
name = "Juan"
age = 30
print("Hello {name}, you are {age} years old")
```

Arbitrary expressions:

```kl
print("Sum: {a + b}")
print("User: {user.name}, role: {get_role(user)}")
```

Rule:

```text
Interpolation is only supported in double-quoted strings.
Single-quoted chars (char) do not support interpolation.
```

---

# Built-in Functions

Available without imports:

## String & Character

```kl
len(s)                    # → i32: length of string s
str(value)                # → str: convert i32/f64/bool to string
char_at(s, i)             # → char (i8): character at index i
ord(c)                    # → i32: Unicode codepoint of char
is_digit(c)               # → i32: 1 if '0'-'9'
is_alpha(c)               # → i32: 1 if 'a'-'z' or 'A'-'Z'
is_alnum(c)               # → i32: 1 if alphanumeric
is_whitespace(c)          # → i32: 1 if space/tab/newline
is_upper(c)               # → i32: 1 if uppercase letter
is_lower(c)               # → i32: 1 if lowercase letter
contains(s, sub)          # → i32: 1 if s contains sub
to_upper(s)               # → str: uppercase copy
to_lower(s)               # → str: lowercase copy
trim(s)                   # → str: trim whitespace
replace(s, from, to)      # → str: replace all occurrences
input(prompt)             # → str: read line from stdin
```

## I/O

```kl
print(value)              # print without newline
println(value)            # print with newline
open(path, mode)          # → i64: file handle (mode: 0=read, 1=write)
read_str(handle)          # → str: read entire file
write_str(handle, text)   # write string to file
close(handle)             # close file handle
```

## Time

```kl
sleep(ms)                 # sleep for milliseconds
now()                     # → i64: Unix timestamp in milliseconds
```

## Math (top-level builtins)

```kl
range(end)                # returns a range value
range(start, end)         # returns a range value
```

---

# Defer

Schedule a call to run when the current scope exits:

```kl
fn process_file():
    file = open("data.txt")
    defer file.close()
    ...
```

Multiple defers run in reverse order (LIFO):

```kl
fn transaction():
    acquire_lock()
    defer release_lock()
    open_db()
    defer close_db()
    ...
```

---

# Guard

Early return or continuation with explicit condition:

```kl
fn process(user: Option<User>):
    guard user != None else:
        return
    print(user.name)
```

Guard is equivalent to:

```kl
if user == None:
    return
```

---

# Async / Await

Declaration:

```kl
async fn load_users() -> [User]:

    ...
```

Direct execution:

```kl
users = await load_users()
```

Background task:

```kl
task = async load_users()

users = await task
```

Rule:

```text
No spawn keyword.

async creates an asynchronous task.

await waits for completion.
```

---

# Classes

```kl
class User:

    name: str

    age: i32
```

---

# Constructors

Constructors use the same name as the class.

```kl
class User:

    name: str

    age: i32

    User(
        name: str,
        age: i32
    ):

        this.name = name

        this.age = age
```

---

# Visibility

## Public

```kl
name: str

fn save():
```

---

## Protected

```kl
_name: str

fn _validate():
```

---

## Private

```kl
__password: str

fn __encrypt():
```

---

# Properties

```kl
class User:

    _name: str

    name: str:

        get:

            return _name

        set(value):

            _name = value
```

Rule:

```text
If a declaration contains:

get:
set:

the compiler treats it as a property.

Within getters and setters, `this` refers to the current instance.
```

---

# Inheritance

```kl
class Animal:

    fn speak()


class Dog : Animal:

    fn speak():

        print("Woof")
```

---

# Polymorphism

```kl
animal: Animal = Dog()

animal.speak()
```

Output:

```text
Woof
```

---

# Abstract Classes

```kl
abs class Animal:

    abs fn speak()
```

Rule:

```text
abs = abstract
```

---

# Contracts

```kl
contract Serializable:

    fn serialize() -> str
```

Implementation:

```kl
class User : Serializable:

    fn serialize() -> str:

        return name
```

---

# Methods Without Implementation

Contract:

```kl
contract Serializable:

    fn serialize() -> str
```

Abstract Class:

```kl
abs class Animal:

    abs fn speak()
```

Regular Class:

```kl
class Animal:

    fn speak()
```

Rule:

```text
A declaration without a body is interpreted as
a method without implementation.

No pass keyword exists.
```

---

# Structs

```kl
struct Point:

    x: f64

    y: f64
```

Usage:

```kl
point = Point(
    x: 10,
    y: 20
)
```

---

# Enums

```kl
enum Status:

    active

    inactive

    suspended
```

Usage:

```kl
status = Status.active
```

---

# Match

```kl
match status:

    active:

        print("Active")

    inactive:

        print("Inactive")

    suspended:

        print("Suspended")
```

---

# Match Guards

Extra condition on a match arm:

```kl
match x:

    n if n > 0:

        print("positive")

    n if n < 0:

        print("negative")

    0:

        print("zero")
```

---

# Break

Terminate the current loop early. Optionally return a value from a `loop:` expression:

```kl
while running:

    if done:

        break
```

Break with value (only in `loop:` expressions):

```kl
result = loop:

    if done:

        break result_value
```

Equivalent to a labeled break with expression. `break` terminates only the innermost loop; `break` in nested loops targets the direct enclosing loop.

---

# Conditionals

```kl
if age >= 18:

    print("Adult")

elif age >= 13:

    print("Teen")

else:

    print("Child")
```

---

# Binding Condition

Create a variable and check truthiness in one step:

```kl
if user = maybe_user:

    print(user.name)
```

Equivalent to:

```kl
if maybe_user:

    user = maybe_user

    print(user.name)
```

The variable `user` is scoped to the `if` block.
Distinct from `==` (comparison) — `=` in a condition always binds a new variable.

---

# While

```kl
while age < 18:

    age += 1
```

Infinite loop:

```kl
while true:

    work()

    if done:

        break
```

---

# While-Bind

Loop over an optional or iterator, binding each iteration:

```kl
while value = iterator.next():

    process(value)
```

The variable `value` is rebound at each iteration.
Loop exits when `value` evaluates to falsy (e.g. `None`).

---

# While-Else

Execute an else block when the loop condition is never met:

```kl
while condition:

    work()
else:

    print("Condition was never true")
```

---

# For-Else

Execute an else block when no items are iterated:

```kl
for item in items:

    print(item)
else:

    print("No items found")
```

Rule:

```text
break

Terminates the current loop.
```

---

# For

```kl
for item in items:

    print(item)
```

Range:

```kl
for i in 0..10:

    print(i)
```

Rule:

```text
No continue keyword.

Execution naturally proceeds to the next iteration.
```

---

# Tuples

```kl
position = (
    10,
    20
)
```

---

# Destructuring

```kl
x, y = position
```

---

# Lists

```kl
names = [
    "John",
    "Anna",
    "Mike"
]
```

Typed:

```kl
ages: list<i32> = [
    10,
    20,
    30
]
```

---

# Range Slicing

Slice lists and strings with `[start..end]`:

```kl
items = [0, 1, 2, 3, 4, 5]

first = items[0..3]       # [0, 1, 2]
rest  = items[3..]        # [3, 4, 5]
copy  = items[..]         # full copy
```

Omitting start defaults to `0`. Omitting end defaults to `len(items)`.

---

# Spread

Spread a list into another:

```kl
combined = [...list_a, ...list_b]

updated = [...original, new_item]
```

Spread an object literal:

```kl
defaults = { theme: "dark", lang: "en" }
config = { ...defaults, lang: "es" }
```

---

# Optional Chaining

Safe access through optional values:

```kl
name = user?.name          # returns Option<str>
city = user?.address?.city # deep optional access
```

Equivalent to:

```kl
name = if user:
    user.name
else:
    None
```

---

# Object Literals

Object literals create values with named fields. Access is via dot notation.

Creation:

```kl
user = {

    name: "John",

    age: 25
}
```

Access:

```kl
user.name

user.age
```

Nested:

```kl
config = {

    database: {

        host: "localhost",

        port: 5432
    }
}
```

Usage:

```kl
config.database.host

config.database.port
```

Type inference:

```text
{ name: str, age: i32 }
```

Object literals are structurally typed. No class declaration needed.

For dynamic key-value maps, use `Dict<K, V>`:

```kl
scores: Dict<str, i32>

scores["player1"] = 100
```

Rule:

```text
Object literals use dot access.

Bracket access is for Dict types only.
```

---

# Error Handling

## Proposed Syntax

```kl
fn find_user(id: i32) -> User!
```

Meaning:

```text
Returns User

May return an error
```

Usage:

```kl
result = find_user(1)

match result:

    ok(user):

        print(user.name)

    error(err):

        print(err)
```

Rule:

```text
Kyle does not use exceptions.

Errors are explicit.
```

---

# Operator Overloading

Types can define custom behavior for operators by implementing specific methods:

```kl
struct Vector:

    x: f64
    y: f64

    fn add(other: Vector) -> Vector:

        return Vector(x + other.x, y + other.y)

    fn mul(scalar: f64) -> Vector:

        return Vector(x * scalar, y * scalar)
```

Operator mapping:

```text
+   ->  add
-   ->  sub
*   ->  mul
/   ->  div
%   ->  rem
**  ->  pow
==  ->  eq
!=  ->  neq
<<  ->  shl
>>  ->  shr
&   ->  bitand
|   ->  bitor
^   ->  bitxor
```

When a binary operator is used, the compiler looks for a matching method
on the left operand's type.

---

# Compile-Time Evaluation

Constants are evaluated at compile time:

```kl
PI = 3.141592
RADIUS = 10
AREA = PI * RADIUS * RADIUS   # computed at compile time
```

Compile-time function calls:

```kl
const fn factorial(n: i32) -> i32:

    if n <= 1:
        return 1
    return n * factorial(n - 1)

RESULT = factorial(10)   # evaluated at compile time
```

The `const` keyword before `fn` guarantees the function can be
evaluated at compile time with restrictions:

```text
No FFI calls
No I/O operations
No heap allocations
No runtime polymorphism
Only pure computations on primitive types
```

---

# Generics

## Generic Class

```kl
class Repository<T>:

    fn add(item: T)
```

Usage:

```kl
users = Repository<User>()
```

---

## Generic Function

```kl
fn first<T>(
    items: list<T>
) -> T:
```

---

# Imports

Import Module:

```kl
import math

import net

import io
```

Specific Import:

```kl
from math import sqrt
```

Alias:

```kl
import database as db
```

---

# Project Structure

```text
src/

    main.kl

    models/

        user.kl

    services/

        auth.kl

    database/

        connection.kl
```

---

# Removed Features

```text
self

let
var

public
private
protected

virtual
override

try
catch
finally

export

pass

continue

spawn

{}

;

user["name"]
```

---

# Future Compiler Stages

1. Lexer
2. Parser
3. AST
4. Semantic Analyzer
5. Type Checker
6. LLVM IR Generator
7. Native Compiler
8. Package Manager
9. Standard Library
10. Self-Hosting Compiler

---

# Package Manager

```bash
kl init

kl build

kl run

kl test

kl add package

kl remove package
```

---

# Compiler Commands

```bash
kl build

kl run

kl test
```

Example:

```bash
kl run main.kl
```

---

# Standard Library (Initial Proposal)

```text
io
math
net
json
time
collections
filesystem
crypto
```

---

# Language Core Principles

```text
Readable like Python

Typed like Rust

Compiled

LLVM Based

No Exceptions

No Hidden Behavior

Fast

Simple

Predictable
```

---

# Long-Term Roadmap

Phase 1

```text
Lexer
Parser
AST
Type System
```

Phase 2

```text
LLVM Backend
Native Compilation
```

Phase 3

```text
Package Manager
Standard Library
```

Phase 4

```text
IDE Support
Language Server
Debugger
Formatter
```

Phase 5

```text
Self-Hosting Compiler
```

---

# Official Language Name

```text
Kyle
```

Meaning:

```text
Kyle
```

---

# Version

```text
Kyle Programming Language Specification v2.0
Last updated: 2026-11-19
```
