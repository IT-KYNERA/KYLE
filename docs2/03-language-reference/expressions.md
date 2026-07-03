# Expressions

## Binary operators

| Category | Operators |
|----------|-----------|
| Arithmetic | `+` `-` `*` `/` `%` `**` |
| Comparison | `==` `!=` `<` `>` `<=` `>=` |
| Logical | `and` `or` `not` |
| Bitwise | `&` `\|` `^` `<<` `>>` |
| Range | `..` `..=` `..<` |
| Type test | `is` |
| Type cast | `as` |

## Primary expressions

```ky
42                  # literal
"hello"             # string
name                # identifier
a + b               # binary
-a                  # unary negate
not flag            # logical not
a.b                 # property access
a.b()               # method call
a[b]                # index
a[b..c]             # slice
f(x, y)             # function call
```

## Assignment

```ky
x = value           # simple
x += 1              # compound (also -= *= /= %=)
(x, y) = tuple      # destructuring
```

## Conditional expression

```ky
result = if x > 0: "positive" else: "negative"
```

## Match expression

```ky
name = match x:
    1: "one"
    2: "two"
    _: "other"
```

## Range expressions

```ky
0..5       # exclusive end: 0,1,2,3,4
0..=5      # inclusive end: 0,1,2,3,4,5
0..<5      # exclusive (alias for ..)
```

## Async expressions

```ky
task = async fetch_data()
result = await task
```

## Precedence

| Level | Operators | Associativity |
|-------|-----------|---------------|
| 15 | `.` | Left |
| 14 | `()` | Left |
| 13 | `[]` | Left |
| 12 | `as` | Left |
| 11 | `**` | Right |
| 10 | `*` `/` `%` | Left |
| 9 | `+` `-` | Left |
| 8 | `<<` `>>` | Left |
| 7 | `&` | Left |
| 6 | `^` | Left |
| 5 | `\|` | Left |
| 4 | `<` `>` `<=` `>=` | Left |
| 3 | `==` `!=` `is` | Left |
| 2 | `..` `..=` `..<` | Left |
| 1 | `and` | Left |
| 0 | `or` | Left |
