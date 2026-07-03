# Expressions

## Binary Operators

| Category | Operators |
|----------|-----------|
| Arithmetic | `+` `-` `*` `/` `%` `**` |
| Comparison | `==` `!=` `<` `>` `<=` `>=` |
| Logical | `and` `or` `not` |
| Bitwise | `&` `\|` `^` `<<` `>>` |
| Range | `..` `..=` `..<` |
| Type | `is` `as` |

## Primary Expressions

```ky
42                  # literal
"hello"             # string
name                # identifier
a + b               # binary
-a                  # unary
a.b                 # property access
a.b()               # method call
a[b]                # index
a[b..c]             # slice
f(x, y)             # function call
```

## Assignment

```ky
x = value           # simple assignment
x += 1              # add-assign (also -= *= /= %=)
(x, y) = tuple      # destructuring
```

## Control Flow Expressions

```ky
result = if x > 0: "positive" else: "negative"

result = match x:
    1: "one"
    2: "two"
    _: "other"
```

## Range Expressions

```ky
0..5        # exclusive: 0,1,2,3,4
0..=5       # inclusive: 0,1,2,3,4,5
0..<5       # exclusive (alias)
```

## Async Expressions

```ky
task = async fetch_data()     # spawn async task
result = await task            # wait for result
```

## Unsafe Block

```ky
result = unsafe:
    ptr = addr_of(some_var)    # raw pointer operations
    ptr[0]
```
