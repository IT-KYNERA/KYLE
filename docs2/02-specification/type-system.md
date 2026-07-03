# Type System

## Primitive Types

| Type | Size | Description |
|------|------|-------------|
| `i8` | 1 byte | Signed 8-bit integer |
| `i16` | 2 bytes | Signed 16-bit integer |
| `i32` | 4 bytes | Signed 32-bit integer (default literal type) |
| `i64` | 8 bytes | Signed 64-bit integer |
| `f32` | 4 bytes | 32-bit floating point |
| `f64` | 8 bytes | 64-bit floating point |
| `bool` | 1 byte | Boolean (`true` / `false`) |
| `char` | 4 bytes | Unicode code point |
| `str` | pointer | Heap-allocated immutable string |
| `ptr` | 8 bytes | Raw pointer (no provenance tracking) |

## Compound Types

### Optional: `T?`
Sugar for `Option<T>`. Either `T` or `none`.

```ky
name: str? = none
if name = some_value:
    println(name)
```

### Fallible: `T!`
Sugar for `Result<T, Error>`. Either `T` or an error.

```ky
fn divide(a: i32, b: i32) i32!:
    if b == 0:
        return error("division by zero")
    a / b
```

### Mutable: `&T`
Makes a variable, parameter, or field mutable.

```ky
count: &i32 = 0          # mutable variable
fn inc(x: &i32):          # mutable parameter
    x = x + 1
```

### Move: `^T`
Ownership transfer parameter.

```ky
fn consume(^data: str):  # takes ownership
    println(data)
```

### List: `[T]`
Dynamic array of type `T` (i64 internally).

```ky
numbers = [1, 2, 3]
numbers.push(4)
first = numbers[0]
```

### Tuple: `(T1, T2, ...)`
Fixed-size heterogeneous collection.

```ky
point = (10, 20)
x = point.0
y = point.1
```

## User-Defined Types

### `final class`
Lightweight struct with fields. No inheritance.

```ky
final class Vec2:
    x: i32
    y: i32
```

### `class`
Full class with single inheritance.

```ky
class Animal:
    name: str

class Dog :: Animal:
    fn bark(self):
        println("woof")
```

### `enum`
Tagged union with optional payload.

```ky
enum Optional:
    Some(i32)
    None
```

### `contract`
Interface (trait).

```ky
contract Comparable:
    fn compare(self, other: Self) i32
```

## Type Inference

Kyle infers types when possible:

```ky
x = 42          # i32 (default literal type)
y: i64 = 42     # explicit i64
z = [1, 2, 3]   # List<i32>
```

## Type Casting

```ky
x = 42 as i64    # i32 → i64
y = 3.14 as i32  # f64 → i32 (truncation)
z = 42 as f64    # i32 → f64
```
