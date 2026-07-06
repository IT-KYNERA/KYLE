# Functions

**Status:** [x] Basic fn, params, return, default params. [ ] `static fn` syntax error. [ ] `Calc.method()` undefined symbol.

## Declaration

```ky
fn add(a: i32, b: i32) i32:
    a + b
```

## Parameters

| Mode | Syntax | Semantics |
|------|--------|-----------|
| Borrow (default) | `s: str` | Immutable borrow |
| Mutable borrow | `s: &str` | Mutable borrow |
| Move | `^s: str` | Ownership transfer |

```ky
fn read(s: str):          # borrow
    println(s)

fn append(s: &str):       # mutable
    s = s + "!"

fn consume(^s: str):      # move
    println(s)
```

## Return type

```ky
fn add(a: i32, b: i32) i32:     # returns i32
    a + b

fn greet(name: str) str:         # returns str
    "Hello, {name}!"

fn log(msg: str):                # returns void
    println(msg)
```

## Multiple return values

```ky
fn divmod(a: i32, b: i32) (i32, i32):
    (a / b, a % b)

(result, remainder) = divmod(10, 3)
```

## Default parameters

```ky
fn greet(name: str, greeting: str = "Hello") str:
    "{greeting}, {name}!"
```

## Methods

Methods are defined inside a class. The instance is accessed via `this` inside the body, but `this` is not declared as a parameter.

```ky
final class Vec2:
    x: i32
    y: i32

    fn len() f64:
        sqrt((this.x * this.x + this.y * this.y) as f64)

a = Vec2 { x: 3, y: 4 }
a.len()     # 5.0
```

## Static methods

```ky
class MathUtils:
    static fn square(x: i32) i32:
        x * x

MathUtils.square(5)    # 25
```

## Operator overloading

```ky
final class Vec2:
    fn op_+(other: Vec2) Vec2:
        Vec2 { x: this.x + other.x, y: this.y + other.y }

a = Vec2 { x: 1, y: 2 }
b = Vec2 { x: 3, y: 4 }
c = a + b               # calls op_+
```

## Closures

```ky
adder = fn(x: i32, y: i32) i32: x + y
result = adder(3, 4)    # 7
```
