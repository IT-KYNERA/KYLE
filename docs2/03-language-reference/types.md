# Types

## Primitive types

| Type | Size | Description |
|------|------|-------------|
| `i8` | 1 byte | Signed 8-bit integer |
| `i16` | 2 bytes | Signed 16-bit integer |
| `i32` | 4 bytes | Signed 32-bit integer (default) |
| `i64` | 8 bytes | Signed 64-bit integer |
| `f32` | 4 bytes | 32-bit floating point |
| `f64` | 8 bytes | 64-bit floating point |
| `bool` | 1 byte | `true` or `false` |
| `char` | 4 bytes | Unicode code point |
| `str` | pointer | Heap-allocated immutable string |
| `ptr` | 8 bytes | Raw pointer (FFI, unsafe) |

## Compound types

### Optional: `T?`

Sugar for `Option<T>`. A value that is either `T` or `none`.

```ky
name: str? = none
if value = get_name():
    println(value)
```

### Fallible: `T!`

Sugar for `Result<T, Error>`. A value that is either `T` or an error.

```ky
fn divide(a: i32, b: i32) i32!:
    if b == 0:
        return error("division by zero")
    a / b
```

### Mutable: `&T`

Marks a variable, parameter, or field as mutable.

```ky
count: &i32 = 0
fn increment(x: &i32):
    x = x + 1
```

### Move: `^T`

Parameter with ownership transfer.

```ky
fn consume(^data: str):
    println(data)
```

### List: `[T]`

Dynamic array of type `T`.

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

## User-defined types

### final class

```ky
final class Vec2:
    x: i32
    y: i32
```

### class

```ky
class Animal:
    name: str
    fn speak():
        println("...")

class Dog :: Animal:
    fn speak():
        println("woof")
```

### enum

```ky
enum Optional:
    Some(i32)
    None
```

### contract

```ky
contract Comparable:
    fn compare(other: This) i32
```

## Type inference

```ky
x = 42          # i32
y: i64 = 42     # explicit i64
z = [1, 2, 3]   # List<i32>
```

## Type casting

```ky
x = 42 as i64
y = 3.14 as i32
z = 42 as f64
```
