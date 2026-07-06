# Variables

## Declaration [x]

Variables are declared with `name = value`. No `let`, `var`, or `const` keywords.

```ky
name = "Ana"        # immutable (default)
age: &i32 = 25      # mutable with &T
```

## Constants [x]

Compile-time constants use `:=`. UPPER_CASE naming by convention.

```ky
VERSION := "1.0.0"
MAX_SIZE := 1024
```

## Mutability [x]

| Form | Mutability |
|-------|-------------|
| `x = value` | Immutable |
| `x: &T = value` | Mutable |
| `x = &value` | Mutable (sugar) |

```ky
x = 5              # immutable
y: &i32 = 5        # mutable
z = &5             # mutable (sugar)

y = y + 1          # reassignment allowed
```

## Scope [x]

Variables are block-scoped. Each indentation level creates a new scope.

```ky
x = 1
if true:
    y = 2
    x = x + y
# y is not accessible here
```

## Destructuring [x]

```ky
point = (10, 20)
(x, y) = point      # x=10, y=20

lst = {1, 2, 3}
(first, second) = lst
```
Nota: destructuring de listas da punteros (valores raw i64), no funciona para tipos reference como lista/string.
