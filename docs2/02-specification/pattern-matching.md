# Pattern Matching

## Match Statement

```ky
match value:
    pattern1:
        body1
    pattern2:
        body2
    _:
        default_body
```

## Match Expression

```ky
result = match value:
    1: "one"
    2: "two"
    _: "other"
```

## Pattern Types

### Literal
```ky
match x:
    0: println("zero")
    42: println("answer")
```

### Identifier
```ky
match x:
    n: println(n)     # binds value to n
```

### Wildcard
```ky
match x:
    _: println("anything")
```

### Or-pattern
```ky
match x:
    1 | 2: println("one or two")
```

### Guard
```ky
match x:
    n if n > 10: println("big")
    n: println(n)
```

### Enum Variant
```ky
enum Optional:
    Some(i32)
    None

match v:
    Optional.Some(n):
        println(n)
    Optional.None:
        println("none")
```

### Tuple
```ky
match point:
    (0, 0): println("origin")
    (x, 0): println(x)
    _: println("other")
```

## IsType Pattern

```ky
match x:
    is str: println("string")
    is i32: println("integer")
```

## Binding If

```ky
if name = optional_value:
    println(name)       # name is non-none here
```

## Binding While

```ky
while line = read_line():
    println(line)       # loop while read_line returns non-none
```
