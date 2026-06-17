# KL Type System Specification v1.0

## Introduction

The KL type system is responsible for:

* Type Safety
* Type Inference
* Generic Validation
* Contract Validation
* Null Safety
* Error Safety
* Compile-Time Verification

The compiler must reject any program whose types cannot be proven correct.

---

# Design Principles

```text
Strong Static Typing

Compile-Time Checking

Type Inference

No Implicit Null

Explicit Error Handling

Generic Support

Predictable Conversions

Zero Hidden Behavior
```

---

# Primitive Types

## Signed Integers

```kl
i8
i16
i32
i64
```

Range:

```text
i8   = -128 to 127

i16  = -32,768 to 32,767

i32  = -2,147,483,648 to 2,147,483,647

i64  = 64-bit signed integer
```

---

## Unsigned Integers

```kl
u8
u16
u32
u64
```

Only positive values.

---

## Integer Overflow Behavior

```text
In debug builds:     arithmetic overflow panics with a runtime error.
In release builds:   overflow wraps in two's complement (defined behavior,
                     NOT undefined behavior like C/C++).

This applies to: +, -, *, **, unary - (negation)
Division by zero:   always panics (debug and release).
```

Explicit overflow control:

```kl
value +% 10     # wrapping add (never panics)
value -% 5      # wrapping sub
value *% 3      # wrapping mul
```

---

## Floating Point

```kl
f32
f64
```

---

## Boolean

```kl
bool
```

Values:

```kl
true
false
```

---

## Character

```kl
char
```

Example:

```kl
letter: char = 'A'
```

---

## String

```kl
str
```

Example:

```kl
name: str = "John"
```

Encoding: UTF-8 internally.

```text
len(str)     = byte count (not code point count)
str[i]       = byte access (O(1), not guaranteed char boundary)
str[0..5]    = byte slice (panics if not on char boundary)
.chars()     = iterator over Unicode code points
```

Rules:

```text
All strings are valid UTF-8. Invalid UTF-8 is a compile-time error for
literals and a runtime error for dynamic input.
Indexing is O(1) by byte offset. Use .chars() for code point iteration.
```

---

# Type Inference

KL automatically infers types whenever possible.

Example:

```kl
age = 25
```

Compiler:

```text
age : i32
```

---

Example:

```kl
price = 10.5
```

Compiler:

```text
price : f64
```

---

Example:

```kl
active = true
```

Compiler:

```text
active : bool
```

---

# Explicit Types

Developer may specify a type.

```kl
age: i64 = 25
```

The assigned value must be compatible.

---

Valid:

```kl
age: i64 = 25
```

---

Invalid:

```kl
name: str = 25
```

Compiler Error:

```text
Cannot assign i32 to str
```

---

# Assignment Rules

Valid:

```kl
age: i32 = 10

age = 20
```

---

Invalid:

```kl
age: i32 = 10

age = "John"
```

Compiler Error:

```text
Expected i32

Received str
```

---

# Constants

Declaration:

```kl
PI = 3.141592
```

Compiler:

```text
Constant<f64>
```

---

Invalid:

```kl
pi = 10
```

Compiler Error:

```text
Cannot modify constant value
```

---

# Numeric Promotion

Allowed:

```kl
i8  -> i16

i16 -> i32

i32 -> i64

u8  -> u16

u16 -> u32

u32 -> u64

f32 -> f64
```

---

Example:

```kl
value: i64 = 25
```

Valid.

---

# Numeric Narrowing

Not automatic.

Invalid:

```kl
value: i8 = 500
```

Compiler Error:

```text
Potential data loss
```

---

Explicit conversion required.

Example:

```kl
value = i8(100)
```

---

# Explicit Casting

KL uses constructor-style syntax for numeric and type casts:

```kl
i32(value)     # i32 -> i32 (identity)
f64(value)     # i32 -> f64 (promotion)
i32(f)         # f64 -> i32 (truncation toward zero)
```

Cast safety rules:

```text
i8(i32)    panics if value is outside -128..127  (debug only)
i16(i32)   panics if value is outside -32768..32767  (debug only)
UINT(i32)  panics if value < 0  (debug only)
```

---

# Type Aliases

```kl
type Age = i32
type Callback = fn(i32) -> str
type UserMap = Dict<str, User>
```

Usage:

```kl
age: Age = 25
callback: Callback = my_fn
```

---

# Function Types

```kl
fn(i32) -> str            # function taking i32, returning str
fn(str, i32) -> void      # 2 params, no return
fn() -> i32               # no params, returns i32
async fn() -> [User]      # async function type
```

Usage:

```kl
handler: fn(str) -> void = log_message
```

---

# is Operator

Runtime type check:

```kl
if value is Admin:
    print("Admin access")

if user is not Admin:
    print("Regular user")
```

Useful for:

```kl
match value:
    is str:
        print("string: " + value)
    is i32:
        print("number: " + str(value))
    is Admin:
        print("admin id: " + value.id)
```

---

# Optional Types

Optional values use:

```kl
Option<T>
```

Example:

```kl
user: Option<User>
```

Meaning:

```text
User

or

None
```

---

# Optional Safety

Invalid:

```kl
user: Option<User>

print(user.name)
```

Compiler Error:

```text
Optional value not checked
```

---

Valid:

```kl
if user:

    print(user.name)
```

---

# Void Type

Functions that return nothing use `void`:

```kl
fn log(message: str) -> void:

    print(message)
```

The `void` type is implicit for functions without `->` return type:

```kl
fn save():

    print("Saved")
```

Is equivalent to:

```kl
fn save() -> void:

    print("Saved")
```

---

# Error Types

KL uses:

```kl
!
```

Example:

```kl
fn find_user(id: i32) -> User!
```

Meaning:

```text
Returns User

May return Error
```

---

# Error Safety

Invalid:

```kl
user = find_user(1)

print(user.name)
```

Compiler Error:

```text
Unhandled error value
```

---

Valid:

```kl
result = find_user(1)

match result:

    ok(user):

        print(user.name)

    error(err):

        print(err)
```

---

# Lists

Declaration:

```kl
users: list<User>
```

---

Inference:

```kl
users = [
    User(),
    User()
]
```

Compiler:

```text
list<User>
```

---

Invalid:

```kl
users = [
    User(),
    123
]
```

Compiler Error:

```text
Mixed list types
```

---

# Object Literals

Example:

```kl
user = {

    name: "John",

    age: 25
}
```

Compiler:

```text
{ name: str, age: i32 }
```

Object literals are structurally typed.

For dynamic key-value maps, use `Dict<K, V>`:

```kl
scores: Dict<str, i32>

scores["player1"] = 100
```

---

# Tuples

Example:

```kl
position = (
    10,
    20
)
```

Compiler:

```text
tuple<i32,i32>
```

---

# Struct Types

Example:

```kl
struct Point:

    x: f64

    y: f64
```

Compiler Type:

```text
Point
```

---

# Class Types

Example:

```kl
class User:
```

Compiler Type:

```text
User
```

---

# Inheritance Rules

Example:

```kl
class Animal:

class Dog : Animal:
```

Valid:

```kl
animal: Animal = Dog()
```

---

Invalid:

```kl
dog: Dog = Animal()
```

Compiler Error:

```text
Cannot assign parent type to child type
```

---

# Contract Compatibility

Contract:

```kl
contract Serializable:

    fn serialize() -> str
```

Implementation:

```kl
class User : Serializable:
```

Valid:

```kl
item: Serializable = User()
```

---

# Generic Types

Example:

```kl
list<User>

Repository<User>
```

Compiler:

```text
Repository<T>
```

---

# Generic Validation

Valid:

```kl
repo: Repository<User>
```

---

Invalid:

```kl
repo: Repository<UnknownType>
```

Compiler Error:

```text
Unknown generic type
```

---

# Generic Function Inference

Example:

```kl
fn first<T>(
    items: list<T>
) -> T:
```

Call:

```kl
name = first(names)
```

Compiler:

```text
T = str
```

---

# Match Validation

Enum:

```kl
enum Status:

    active

    inactive

    suspended
```

Valid:

```kl
match status:

    active:

    inactive:

    suspended:
```

---

Invalid:

```kl
match status:

    active:
```

Compiler Error:

```text
Non exhaustive match
```

---

# Function Validation

Function:

```kl
fn sum(
    a: i32,
    b: i32
) -> i32:
```

Valid:

```kl
sum(10,20)
```

---

Invalid:

```kl
sum("10",20)
```

Compiler Error:

```text
Expected i32

Received str
```

---

# Return Validation

Valid:

```kl
fn get_age() -> i32:

    return 25
```

---

Invalid:

```kl
fn get_age() -> i32:

    return "John"
```

Compiler Error:

```text
Expected i32

Received str
```

---

# Async Types

Function:

```kl
async fn load_users() -> [User]:
```

Compiler Type:

```text
Task<[User]>
```

Await:

```kl
users = await load_users()
```

Resolved Type:

```text
[User]
```

---

# Type Checker Pipeline

```text
AST

↓

Scope Resolution

↓

Type Resolution

↓

Type Inference

↓

Generic Resolution

↓

Contract Validation

↓

Error Validation

↓

Type Safety Verification
```

---

# Type System Principles

```text
Everything Must Be Typed

Inference Is Allowed

Implicit Narrowing Is Forbidden

Null Must Be Explicit

Errors Must Be Explicit

Contracts Must Be Verified

Generics Must Be Verified

Type Safety Is Mandatory
```

---

# End of Type System Specification
