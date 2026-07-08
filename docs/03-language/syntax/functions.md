# Functions

> Declaration y uso de functions en Kyle.

## Declaration basica

```ky
fn add(a: i32, b: i32) i32:
 a + b
```

- `fn` keyword
- Parameters with type: `name: Type`
- Type de retorno after de parameters
- Cuerpo indentado
- La ultima expression is retorno (implicito)

## Parameters (Move by defecto)

```ky
fn consume(s: str): # MOVE: caller pierde ownership
 println(s)

fn read(s: &str): # BORROW: caller presta
 println(s)

fn fill(s: ^&str): # MUT BORROW: caller presta mutable
 s = s + "!"
```

| Modo | Syntax | Semantics |
|------|----------|-----------|
| Move (default) | `s: str` | Ownership transfer |
| Borrow | `s: &str` | Referencia inmutable |
| Mutable Borrow | `s: ^&str` | Referencia mutable |

## Return type

```ky
fn add(a: i32, b: i32) i32: # returns i32
 a + b

fn greet(name: str) str: # returns str
 "Hello, {name}!"

fn log(msg: str): # returns void
 println(msg)

fn divide(a: i32, b: i32) i32!: # returns fallible
 if b == 0:
 return error("div by zero")
 a / b
```

## Default parameters

```ky
fn connect(host: str = "localhost", port: i32 = 8080):
 println("conectando a " + host + ":" + port.to_str())

connect() # localhost:8080
connect("example.com") # example.com:8080
connect("example.com", 80) # example.com:80
```

## Function pointers

```ky
fn double(n: i64) i64:
 n * 2

fn main() i32:
 fn_ptr: ptr = double as ptr
 result: i64 = fn_ptr(21) # call indirect
 println(result.to_str()) # 42
 0
```

## Closures

```ky
doubled: {i32} = list.map(fn(x: i32): x * 2)
filtered: {i32} = list.filter(fn(x: i32): x > 5)
```

## async fn

```ky
async fn fetch(url: &str) str:
 "response"

fn main() i32:
 task = fetch("https://...")
 result: str = await task
 0
```

## static fn

```ky
class MathUtils:
 static fn square(x: i32) i32:
 x * x

MathUtils.square(5) # 25
```
