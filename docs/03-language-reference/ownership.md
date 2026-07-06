# Ownership and Borrowing

Kyle uses borrow semantics by default. Parameters are borrowed, not moved, unless `^T` is used.

## Borrow (default)

```ky
fn read(s: str):
    println(s)

fn main():
    s = "hello"
    read(s)        # s is borrowed, not moved
    println(s)     # s is still usable
```

## Mutable borrow (&T)

```ky
fn append(s: &str):
    s = s + "!"

fn main():
    s: &str = "hello"
    append(&s)
    println(s)     # "hello!"
```

## Move (^T)

```ky
fn consume(^s: str):
    println(s)

fn main():
    s = "hello"
    consume(^s)    # ownership transferred
    println(s)     # error: s was moved
```

## Copy types

Simple types are copied automatically:

```ky
x = 42
y = x            # copy, not move
x + 1            # x is still usable
```

## Clone

For non-Copy types (str, {T}, dict):

```ky
a = "hello"
b = a.clone()
println(a)       # a is still usable
```

## Mutable variables

```ky
count: &i32 = 0
count = count + 1
```

## Rules

1. One mutable reference XOR multiple immutable references
2. References cannot outlive their source value
3. No dangling pointers (compile-time checked)
