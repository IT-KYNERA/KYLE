# Functions

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
fn read(s: str):          # borrow (default)
    println(s)

fn append(s: &str):       # mutable borrow
    s = s + "!"

fn consume(^s: str):      # move (ownership)
    println(s)
```

## Return Type

```ky
fn add(a: i32, b: i32) i32:     # returns i32
    a + b

fn greet(name: str):              # returns str
    "Hello, {name}!"

fn log(msg: str):                 # returns void
    println(msg)
```

## Multiple Return Values

Returns a tuple:

```ky
fn divmod(a: i32, b: i32) (i32, i32):
    (a / b, a % b)

(result, remainder) = divmod(10, 3)
```

## Default Parameters

```ky
fn greet(name: str, greeting: str = "Hello") str:
    "{greeting}, {name}!"
```

## Methods

```ky
final class Vec2:
    x: i32
    y: i32
    
    fn len(self) f64:
        sqrt((self.x * self.x + self.y * self.y) as f64)

a = Vec2 { x: 3, y: 4 }
a.len()     # 5.0
```

## Static Methods

```ky
class MathUtils:
    static fn square(x: i32) i32:
        x * x

MathUtils.square(5)    # 25
```

## Operators as Methods

```ky
final class Vec2:
    fn op_+(self, other: Vec2) Vec2:
        Vec2 { x: self.x + other.x, y: self.y + other.y }

a = Vec2 { x: 1, y: 2 }
b = Vec2 { x: 3, y: 4 }
c = a + b    # calls op_+
```

## Closures

```ky
adder = fn(x: i32, y: i32) i32: x + y
result = adder(3, 4)    # 7
```
