# Generics

**Status:** [x] `class Box<T>`, `fn identity<T>`, `identity<i32>(42)`, `Box<i32> {value:7}`.

## Generic classes

```ky
final class Stack<T>:
    items: {T}                # lista dinámica interna

    fn push(this, item: T):
        this.items.push(item)

    fn pop(this) T:
        this.items.pop()
```

## Usage

```ky
int_stack = Stack<i32> { items: {} }
int_stack.push(5)
```

## Generic functions

```ky
fn identity<T>(value: T) T:
    value
```

## Type parameter constraints

```ky
class Box<T>:
    value: T
```

Generics support classes and structs with type parameters. Full monomorphization is performed at compile time.
