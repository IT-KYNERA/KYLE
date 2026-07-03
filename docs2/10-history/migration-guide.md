# Migration Guide

## v0.4 → v0.5

### Variable declaration

Old syntax used `let`, `var`, `const`:

```
let x = 5        # old
var y = 10       # old
const Z = 20     # old
```

New syntax uses direct assignment:

```ky
x = 5            # immutable
y: &i32 = 10     # mutable
Z := 20          # constant
```

### Parameters

Old syntax moved by default. New syntax borrows by default:

```ky
fn read(s: str):         # borrows (new default)
fn consume(^s: str):     # explicit move
```

### struct → final class

```ky
final class Point:       # replaces `struct Point:`
    x: i32
    y: i32
```

### Option → T?

```ky
name: str? = none        # replaces `name: Option<str> = None`
```
