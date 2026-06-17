# KL Error System Specification v1.0

---

# Philosophy

KL does not use exceptions.

The language follows these principles:

```text
Errors are values.

Errors are explicit.

Errors are part of the type system.

No hidden control flow.

No try/catch.

No runtime surprises.
```

---

# Design Goals

```text
Predictable

Type Safe

Compiler Verified

Zero Hidden Errors

Easy To Read

Easy To Maintain

Compatible With Async

Compatible With LLVM
```

---

# Core Concept

Every operation that may fail must explicitly declare it.

Example:

```kl
fn load_user(id: i32) -> User!
```

Meaning:

```text
Returns:
    User

May fail:
    Yes
```

The `!` symbol indicates a fallible operation.

---

# Success Functions

Functions that cannot fail:

```kl
fn add(a: i32, b: i32) -> i32:

    return a + b
```

Compiler interpretation:

```text
Guaranteed Success
```

---

# Fallible Functions

```kl
fn load_user(id: i32) -> User!
```

Compiler interpretation:

```text
May return:

User

or

Error
```

---

# Error Type

Internal compiler representation:

```text
Result<T>
```

Example:

```kl
fn load_user(id: i32) -> User!
```

Compiler expands internally to:

```text
Result<User>
```

Developer never writes Result manually.

---

# Error Object

Base error structure:

```kl
struct Error:

    code: str

    message: str
```

Example:

```kl
Error(

    code: "USER_NOT_FOUND",

    message: "User does not exist"
)
```

---

# Returning Errors

Syntax:

```kl
error Error(

    code: "USER_NOT_FOUND",

    message: "User does not exist"
)
```

Example:

```kl
fn load_user(id: i32) -> User!:

    if id <= 0:

        error Error(

            code: "INVALID_ID",

            message: "Invalid user id"
        )

    return user
```

---

# Error Propagation

Operator:

```kl
?
```

Purpose:

```text
If success:

    continue

If error:

    return immediately
```

Example:

```kl
fn load_profile(id: i32) -> Profile!:

    user = load_user(id)?

    profile = load_profile_data(user)?

    return profile
```

Equivalent logic:

```text
if error:

    return error

else:

    continue
```

---

# Match Error Handling

Example:

```kl
result = load_user(1)

match result:

    ok(user):

        print(user.name)

    error(err):

        print(err.message)
```

---

# Error Pattern Matching

```kl
match load_user(1):

    ok(user):

        print(user.name)

    error(err):

        print(err.message)
```

---

# Multiple Error Types

Custom Errors:

```kl
struct UserNotFound:

    id: i32
```

```kl
struct InvalidUserId:

    value: i32
```

Usage:

```kl
error UserNotFound(

    id: user_id
)
```

---

# Typed Error Returns

Optional advanced syntax:

```kl
fn load_user(
    id: i32
) -> User!<UserNotFound, InvalidUserId>
```

Meaning:

```text
Returns User

May return:

UserNotFound

InvalidUserId
```

---

# Enum Variants With Data

KL supports enum variants with payload data. This is how `Result<T, E>`
and `Option<T>` are defined internally.

## Internal Definitions

```kl
enum Option<T>:
    some(T)
    none

enum Result<T, E>:
    ok(T)
    error(E)
```

These are **built-in types** available everywhere.

## User-Defined Enums With Data

```kl
enum Message:
    text(str)
    number(i32)
    user(User)
    error(i32, str)
```

## Pattern Matching With Data

```kl
match msg:
    text(t):
        print(t)

    number(n):
        print(n)

    user(u):
        print(u.name)

    error(code, msg):
        print(code + ": " + msg)
```

## Error Handling With Result

```kl
fn load_user(id: i32) -> Result<User, Error>:

    if id <= 0:
        return error(Error("INVALID_ID"))

    return ok(user)
```

The `!` syntax is syntactic sugar for `Result<T, Error>`:

```kl
fn load_user(id: i32) -> User!
# Equivalent to:
fn load_user(id: i32) -> Result<User, Error>
```

---

# Error Propagation (`?`)

The `?` operator is used **exclusively for error propagation** in expressions.

```text
If the value is ok(T):
    unwrap T and continue

If the value is error(E):
    return error(E) immediately from the current function
```

The function must return a compatible `Result<T, E>` or `T!` type for `?` to work.

`?` is NOT used for optional types. Optionals use `Option<T>` syntax.

---

# Compiler Validation

Compiler checks:

```text
All error types exist

All error types are valid

All paths return correctly

No hidden failures
```

---

# Exhaustive Match

Example:

```kl
match load_user(1):

    ok(user):

        print(user.name)

    error(UserNotFound):

        print("Not found")

    error(InvalidUserId):

        print("Invalid id")
```

Compiler verifies:

```text
All declared error types handled.
```

---

# Error Codes

Recommended format:

```text
MODULE_ERROR_NAME
```

Examples:

```text
USER_NOT_FOUND

INVALID_USER_ID

DATABASE_TIMEOUT

NETWORK_FAILURE

FILE_NOT_FOUND
```

---

# Panic

Rare unrecoverable errors.

Syntax:

```kl
panic("Memory corruption")
```

Meaning:

```text
Immediate application termination.
```

Used only for:

```text
Compiler bugs

Internal invariants

Fatal runtime corruption
```

Never for normal business logic.

---

# Assertions

Development checks.

```kl
assert age >= 18
```

Equivalent:

```text
If false:

    panic
```

---

# Nullable Values

KL discourages nullable references.

Instead:

```kl
Option<User>
```

Meaning:

```text
Optional<User>
```

Example:

```kl
user: Option<User>
```

Check:

```kl
if user != none:

    print(user.name)
```

---

# Optional Unwrapping

Safe:

```kl
if user:

    print(user.name)
```

---

# Optional Match

```kl
match user:

    some(value):

        print(value.name)

    none:

        print("Missing")
```

---

# Async Errors

Fallible async function:

```kl
async fn load_user(
    id: i32
) -> User!
```

Usage:

```kl
user = await load_user(1)?
```

Flow:

```text
Await completion

Check error

Return error automatically if failure
```

---

# File Errors

Example:

```kl
fn read_file(
    path: str
) -> str!
```

Usage:

```kl
content = read_file("data.txt")?
```

---

# Network Errors

```kl
async fn get(
    url: str
) -> Response!
```

Usage:

```kl
response = await get(url)?
```

---

# Database Errors

```kl
fn save_user(
    user: User
) -> bool!
```

Usage:

```kl
save_user(user)?
```

---

# Compiler Rules

Rule 1:

```text
Every fallible function
must declare !
```

Rule 2:

```text
Every propagated error
must use ?
```

Rule 3:

```text
No exceptions exist.
```

Rule 4:

```text
No try/catch exists.
```

Rule 5:

```text
Errors are values.
```

Rule 6:

```text
Compiler verifies all paths.
```

---

# Internal Compiler Model

Developer Syntax:

```kl
fn load_user() -> User!
```

Compiler AST:

```text
Result<User>
```

LLVM Layer:

```text
Tagged Union

Success Variant

Error Variant
```

Implementation detail hidden from developer.

---

# Standard Error Types

Initial KL standard library:

```text
Error

IoError

NetworkError

FileError

ParseError

JsonError

DatabaseError

TimeoutError

PermissionError
```

---

# Error System Principles

```text
No Exceptions

No Hidden Control Flow

No Runtime Surprises

Compiler Verified

Strongly Typed

LLVM Friendly

Async Compatible

Predictable

Readable

Safe
```

---

# Future Extensions

Planned:

```text
Stack Traces

Error Chaining

Error Context

Diagnostic Reports

IDE Error Visualization

Structured Logging Integration
```

---

# Version

```text
KL Error System Specification v1.0
```
