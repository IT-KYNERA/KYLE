# Variables

## Declaration

Variables are declared with `name = expr` for immutable, or `name: &T = expr` for mutable.

```ky
name = "Ana"          # immutable str
age: &i32 = 25        # mutable i32
```

## Constants

Compile-time constants use `:=` and UPPER_CASE naming.

```ky
VERSION := "1.0.0"
MAX_SIZE := 1024
```

## Mutability

- `name = value` → immutable variable
- `name: &T = value` → mutable variable
- `name = &value` → mutable variable (sugar for `&T`)

```ky
x = 5                 # immutable
y: &i32 = 5           # mutable
z = &5                # mutable (sugar)
```

## Scope

Variables are block-scoped. Each indentation level creates a new scope.

```ky
x = 1
if true:
    y = 2             # inner scope
    x = x + y         # can read outer x
# y is not accessible here
```

## Destructuring

```ky
point = (10, 20)
(x, y) = point        # x=10, y=20

list = [1, 2, 3]
(first, second) = list # first=1, second=2
```
