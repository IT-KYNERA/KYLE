# Ownership and Borrowing

## Ownership

Every value in Kyle has exactly one owner at any time.

```ky
fn main():                       # "hello" is owned by `s`
    s = "hello"
    println(s)                   # `s` is borrowed (read-only)
```

## Borrow (Default)

Parameters are **borrowed by default** — the caller retains ownership.

```ky
fn read(s: str):                 # borrows s (default)
    println(s)

fn main():
    s = "hello"
    read(s)                      # s is borrowed, not moved
    println(s)                   # ✅ s still usable here
```

## Mutable Borrow (`&T`)

Use `&T` for mutable parameters. Changes propagate to the caller.

```ky
fn append(s: &str):              # mutable borrow
    s = s + "!"

fn main():
    s: &str = "hello"            # mutable variable
    append(&s)                   # pass mutable reference
    println(s)                   # "hello!"
```

## Move (`^T`)

Use `^T` for ownership transfer. The caller loses access.

```ky
fn consume(^s: str):             # takes ownership
    println(s)

fn main():
    s = "hello"
    consume(^s)                  # ownership transferred
    println(s)                   # ❌ compile error: s moved
```

## Copy Types

Simple types are copied automatically (no move semantics):

```ky
x = 42
y = x               # copy (i32 is Copy)
x + 1               # ✅ x still usable
```

## Clone

For non-Copy types (str, list, dict):

```ky
a = "hello"
b = a.clone()       # explicit clone
println(a)          # ✅ a still usable
```

## Mutable Variables

```ky
count: &i32 = 0     # mutable variable (&T in declaration)
count = count + 1   # reassign
```

## Mutable Fields

```ky
final class Config:
    name: str
    port: &i32       # mutable field

config = Config { name: "server", port: 8080 }
config.port = 9090  # allowed: field is &T
```

## Rules

1. One mutable reference XOR multiple immutable references
2. References cannot outlive their source value
3. No dangling pointers (compile-time checked)
