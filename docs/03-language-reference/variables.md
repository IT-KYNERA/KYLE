# Variables

## Declaration [x]

Variables are declared with `name = value`. No `let`, `var`, or `const` keywords.

```ky
name = "Ana"        # immutable (default), OWNED (str es Move)
age: ^i32 = 25      # mutable with ^T
```

## Constants [x]

Compile-time constants use `:=`. UPPER_CASE naming by convention.

```ky
VERSION := "1.0.0"
MAX_SIZE := 1024
```

## Mutability [x]

| Form | Mutability | Ownership |
|------|------------|-----------|
| `x = value` | Immutable | Copy o Move según tipo |
| `x: ^T = value` | Mutable | Ídem |
| Copy types | — | `y = x` copia (ambos vivos) |
| Move types | — | `y = x` mueve (x inválido) |

```ky
x = 5              # immutable, COPY (i32)
y: ^i32 = 5        # mutable, COPY
y = y + 1          # reassignment allowed

s = "hola"         # immutable, MOVE (str)
t = s              # MOVE: s inválido después
t = s.clone()      # COPY explícita: ambos vivos
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
