# Types

> **Leyenda:** `[x]` = implementado y funcional. `[ ]` = diseñado pero no implementado.
> La sintaxis mostrada es la sintaxis FINAL de Kyle. Lo que está `[ ]` aún no compila.

---

## Copy vs Move semantics [x]

| Semántica | Tipos |
|-----------|-------|
| **Copy** (automático en `y = x`) | `i8..u64`, `f32..f64`, `bool`, `char`, `ptr` |
| **Move** (ownership transfer en `y = x`) | `str`, `{T}`, `{K:V}`, `[T; N]`, classes, structs, enums |

```ky
x = 42; y = x          # ambos vivos (Copy)
s = "hola"; t = s      # s inválido (Move)
t = s.clone()          # ambos vivos (copia explícita)
```

Ver `ownership.md` para detalle completo.

---

## Primitive types [x]

| Type | Copy? | Size | Description |
|------|-------|------|-------------|
| `i8` | ✅ | 1 | Signed 8-bit |
| `i16` | ✅ | 2 | Signed 16-bit |
| `i32` | ✅ | 4 | Signed 32-bit (default) |
| `i64` | ✅ | 8 | Signed 64-bit |
| `u8` | ✅ | 1 | Unsigned 8-bit |
| `u16` | ✅ | 2 | Unsigned 16-bit |
| `u32` | ✅ | 4 | Unsigned 32-bit |
| `u64` | ✅ | 8 | Unsigned 64-bit |
| `f32` | ✅ | 4 | 32-bit float |
| `f64` | ✅ | 8 | 64-bit float (default) |
| `bool` | ✅ | 1 | `true` or `false` |
| `char` | ✅ | 4 | Unicode code point |
| `str` | 🚫 | ptr | Heap-allocated string |
| `ptr` | ✅ | 8 | Raw pointer (FFI/unsafe) |
| `void` | — | 0 | No value (return only) |
| `never` | — | 0 | Diverging functions |

```ky
x = 42               # i32
x: i64 = 42          # i64 explícito
x = 3.14             # f64
b = true             # bool
c = 'a'              # char (⚠️ bug: infiere como i32)
s = "hello"          # str
p = 0 as ptr         # ptr
```

---

## Compound types

### Array: `[T; N]` [x]

Stack array, tamaño fijo en compile-time. GEP directo, cero runtime calls.

```ky
a = [1, 2, 3]                # → [i32; 3]
b = [0; 100]                 # → [i32; 100], repetido
c = [1 as i64; 10000]        # → [i64; 10000]
first = a[0]                 # GEP + load
a[0] = 99                    # GEP + store
```

### List: `{T}` [x]

Heap list dinámica.

```ky
v = {1, 2, 3}                # → {i32}
v.push(4)
v.reserve(100)               # pre-asigna capacidad
x = v[0]                     # listGet (runtime)
v[0] = 99                    # listSet (runtime)
v.pop()                      # listPop (runtime)
v.len()                      # listLen (runtime)
```

### Dict: `{K: V}` [x]

Heap dictionary (hash map).

```ky
d = {"name": "Kyle", "age": 30}   # → {str: i32}
d["city"] = "NYC"
name = d["name"]
d.len()
```

### Tuple: `(T1, T2, ...)` [ ]

Fixed-size heterogeneous.

```ky
t = (1, "hello", 3.14)       # → (i32, str, f64)
x = t.0                      # → 1
y = t.1                      # → "hello"
```

### Set: `Set<T>` [ ]

Hash set. Construcción vía `Set{...}` o constructor.

```ky
s: Set<i32> = Set{1, 2, 3}   # set literal
s.add(4)
s.contains(1)                # → true
s.remove(1)
for val in s:
    println(val.toStr())
```

### Queue via list [ ]

No hay tipo `Queue<T>` dedicado. Usar `{T}` con `.push()` / `.popFirst()`:

```ky
q: {i32} = {}
q.push(10)                   # enqueue
q.push(20)
val = q.popFirst()           # dequeue → 10 (FIFO)
q.len()
```

### Stack via list [ ]

No hay tipo `Stack<T>` dedicado. Usar `{T}` con `.push()` / `.pop()` — ya funciona:

```ky
st: {i32} = {}
st.push(10)                  # push
st.push(20)
val = st.pop()               # → 20 (LIFO)
st.len()
```

> **Decisión de diseño:** Stack y Queue no tienen tipos dedicados porque `{T}` ya soporta
> todas las operaciones necesarias con métodos `.push()`, `.pop()`, `.popFirst()`.
> Esto sigue el enfoque de Go y JavaScript, donde arrays/listas cubren ambos roles.

### Slice: `&[T]` [ ]

Vista de un array existente (no copia). Similar a Rust `&[T]`.

```ky
a = [1, 2, 3, 4, 5]
s: &[i32] = &a[1..3]        # slice: [2, 3]
first = s[0]                 # → 2
```

---

## Optional / Fallible

### Optional: `T?` [ ]

```ky
name: str? = None
if value = getName():       # pattern matching
    println(value)
```

### Fallible: `T!` [ ]

```ky
fn divide(a: i32, b: i32) i32!:
    if b == 0:
        return error("division by zero")
    a / b

result = divide(10, 2)
match result:
    ok(v): println(v)
    error(e): println(e)
```

---

## Ownership types

### Mutable: `^T` [x]

```ky
x: ^i32 = 0          # mutable
x = x + 1            # reasignación permitida
```

### Borrow: `&T` [x]

```ky
fn read(s: &str):     # parámetro: borrow
    println(s)

fn main():
    name = "Kyle"
    read(&name)        # name prestado
    println(name)       # ✅ name sigue vivo
```

### Mutable borrow: `^&T` [x]

```ky
fn fill(buf: ^&str):
    buf = "datos"

fn main():
    buf: ^str = ""
    fill(^&buf)
    println(buf)         # "datos"
```

### box: `box<T>` [ ]

Heap allocation explícita.

```ky
b: box<i32> = box(42)
*b = *b + 1              # deref + mutate
```

### rc: `rc<T>` [ ]

Reference counting (single-thread).

```ky
r: rc<str> = rc("hello")
r2 = r.clone()          # incrementa refcount
println(*r)               # deref
```

### arc: `arc<T>` [ ]

Atomic reference counting (multi-thread).

```ky
a: arc<i64> = arc(0)
```

---

## Concurrency types

### async fn / await [x]

```ky
async fn fetch(url: &str) str:
    "response"

fn main():
    task = async: fetch("https://...")
    result = await task
```

### future: `future<T>` [ ]

```ky
task: future<str> = async:
    "result"
val = await task
```

### channel: `channel<T>` [ ]

```ky
ch: channel<i32> = channel(16)     # buffer 16
ch.send(42)
val = ch.recv()
ch.len()
ch.close()
```

### select [ ]

```ky
select:
    &msg -> ch1:
        println("got: " + msg)
    &msg -> ch2:
        println("got: " + msg)
    after 1s:
        println("timeout")
```

### mutex: `mutex<T>` [ ]

```ky
m: mutex<i32> = mutex(0)
lock(m):
    *val += 1                 # operación segura
```

### atomic: `atomicI64` / `atomicBool` [ ]

```ky
counter: atomicI64 = atomicI64(0)
counter.fetchAdd(1)
counter.load()                # → 1

flag: atomicBool = atomicBool(false)
flag.store(true)
flag.load()                   # → true
```

### iterator [ ]

```ky
iter = list.iter()
doubled = iter.map(fn(x): x * 2)
filtered = iter.filter(fn(x): x > 5)
result = doubled.collect()     # → {i32}
```

---

## Specialized types (NATIVOS)

> Todos estos son nativos de Kyle (no requieren `from X import Y`).
> Están disponibles globalmente como tipos built-in del lenguaje.

### dateTime [ ]

```ky
dt = dateTime.now()
dt = dateTime.parse("2024-01-01T00:00:00")
dt = dateTime.fromYmdhms(2024, 1, 1, 0, 0, 0)
year = dt.year()
month = dt.month()
day = dt.day()
hour = dt.hour()
minute = dt.minute()
second = dt.second()
dt2 = dt.addDays(7)
dt3 = dt.addHours(3)
diff = dt.diff(dt2)
```

### duration [ ]

```ky
d = duration.fromSecs(60)
d = duration.fromMillis(1000)
d = duration.fromHours(1)
d = duration.fromDays(7)
d.toStr()              # → "1h 0m 0s"
```

### date [ ]

```ky
d = date.today()
d = date.fromYmd(2024, 1, 1)
d = date.parse("2024-01-01")
year = d.year()
month = d.month()
weekday = d.weekday()
d2 = d.addDays(7)
```

### time [ ]

```ky
t = time.now()
t = time.fromHms(12, 30, 0)
t = time.parse("12:30:00")
hour = t.hour()
minute = t.minute()
second = t.second()
```

### bytes [ ]

```ky
b = bytes.new(1024)
b = bytes.fromHex("deadbeef")
b = bytes.fromBase64("SGVsbG8=")
b.len()
val = b.get(0)           # → byte
b.set(0, 255)
hex = b.hex()
b64 = b.toBase64()
```

### decimal [ ]

```ky
d = decimal.fromStr("3.14")
d = decimal.fromI64(314, 2)    # 3.14
d.round(1)                       # → 3.1
d.truncate()                     # → 3
d.toStr()                       # → "3.14"
```

### uuid [ ]

```ky
id = uuid.v4()
id = uuid.parse("550e8400-e29b-41d4-a716-446655440000")
id.toStr()
```

### url [ ]

```ky
u = url.parse("https://user:pass@host:8080/path?q=1#frag")
u.scheme()     # → "https"
u.host()       # → "host"
u.port()       # → 8080
u.path()       # → "/path"
u.query()      # → "q=1"
```

### regex [ ]

```ky
re = regex("[0-9]+")
re.isMatch("abc123")       # → true
m = re.find("abc123")       # → "123"
result = re.replace("abc123", "X")  # → "abcX"
```

### env [ ]

```ky
val = env("PATH")
env("MY_VAR", "value")      # set
```

### json [ ]

```ky
j = json.parse('{"name": "Kyle", "age": 30}')
name = j["name"]          # access field
j["city"] = "NYC"         # set field
str = j.stringify()        # serialize
str = j.pretty()           # pretty-print
```

### file [ ]

```ky
f = file.open("/tmp/test.txt", "w")
f.write("hello world")
f.close()

f = file.open("/tmp/test.txt", "r")
content = f.read()           # → str
f.close()
f.exists()                   # → true
```

### socket [ ]

```ky
server = socket.tcpListen(8080)
client = server.accept()
data = client.read(1024)
client.write("HTTP/1.1 200 OK\r\n\r\n")
client.close()
server.close()

# Client mode
conn = socket.tcpConnect("example.com", 80)
conn.write("GET / HTTP/1.1\r\nHost: example.com\r\n\r\n")
resp = conn.read(4096)
conn.close()
```

### path [ ]

```ky
p = path("/home/user/file.txt")
p.dirname()          # → "/home/user"
p.basename()         # → "file.txt"
p.extension()        # → ".txt"
p.exists()           # → true
p.isFile()           # → true
p.isDir()            # → false
p.join("subdir")     # → "/home/user/file.txt/subdir"
```

### bigInt [ ]

```ky
n = bigInt("123456789012345678901234567890")
n2 = bigInt.fromI64(42)
n3 = n + n2
n4 = n * n2
n.toStr()
```

### strBuilder [ ]

```ky
sb = strBuilder(50000)
sb.append("x", 1)
sb.append("hello", 5)
result = sb.toStr()
```

---

## Status summary

| Categoría | [x] Completo | [ ] Diseñado | ❌ No planeado |
|-----------|:-----------:|:-----------:|:-------------:|
| Primitives | 13 | 2 (`u8-u64` codegen, `never`) | 1 (`byte`) |
| Compounds | 4 | 4 (tuple, Set, slice) | 2 (Queue, Stack — usar `{T}`) |
| Ownership | 3 | 4 (box, rc, arc, weak) | 0 |
| Concurrency | 1 (async/await) | 7 (future, channel, select, mutex, atomic, iterator) | 0 |
| Specialized | 1 (strBuilder) | 14 (dateTime, duration, date, time, bytes, decimal, uuid, url, regex, env, json, file, socket, path) | 0 |
| **Total** | **22** | **31** | **3** |
