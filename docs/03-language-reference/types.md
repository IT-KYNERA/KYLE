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
x = v[0]                     # ky_list_get
v[0] = 99                    # ky_list_set
v.pop()                      # ky_list_pop
v.len()                      # ky_list_len
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
    println(val.to_str())
```

### Queue: `Queue<T>` [ ]

FIFO queue. Constructor vía `Queue{}`.

```ky
q: Queue<i32> = Queue{}
q.push(10)                   # enqueue
q.push(20)
val = q.pop()                # dequeue → 10 (FIFO)
q.len()
```

### Stack: `Stack<T>` [ ]

LIFO stack. Constructor vía `Stack{}`.

```ky
st: Stack<i32> = Stack{}
st.push(10)                  # push
st.push(20)
val = st.pop()               # → 20 (LIFO)
st.len()
```

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
if value = get_name():       # pattern matching
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

### Box: `Box<T>` [ ]

Heap allocation explícita.

```ky
b: Box<i32> = Box(42)
*b = *b + 1              # deref + mutate
```

### Rc: `Rc<T>` [ ]

Reference counting (single-thread).

```ky
rc: Rc<str> = Rc("hello")
rc2 = rc.clone()          # incrementa refcount
println(*rc)               # deref
```

### Arc: `Arc<T>` [ ]

Atomic reference counting (multi-thread).

```ky
arc: Arc<i64> = Arc(0)
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

### Future: `Future<T>` [ ]

```ky
task: Future<str> = async:
    "result"
val = await task
```

### Channel: `Channel<T>` [ ]

```ky
ch: Channel<i32> = Channel(16)     # buffer 16
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

### Mutex: `Mutex<T>` [ ]

```ky
m: Mutex<i32> = Mutex(0)
lock(m):
    *val += 1                 # operación segura
```

### Atomic: `AtomicI64` / `AtomicBool` [ ]

```ky
counter: AtomicI64 = AtomicI64(0)
counter.fetch_add(1)
counter.load()                # → 1

flag: AtomicBool = AtomicBool(false)
flag.store(true)
flag.load()                   # → true
```

### Iterator [ ]

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

### DateTime [ ]

```ky
dt = DateTime.now()
dt = DateTime.parse("2024-01-01T00:00:00")
dt = DateTime.from_ymdhms(2024, 1, 1, 0, 0, 0)
year = dt.year()
month = dt.month()
day = dt.day()
hour = dt.hour()
minute = dt.minute()
second = dt.second()
dt2 = dt.add_days(7)
dt3 = dt.add_hours(3)
diff = dt.diff(dt2)
```

### Duration [ ]

```ky
d = Duration.from_secs(60)
d = Duration.from_millis(1000)
d = Duration.from_hours(1)
d = Duration.from_days(7)
d.to_str()              # → "1h 0m 0s"
```

### Date [ ]

```ky
d = Date.today()
d = Date.from_ymd(2024, 1, 1)
d = Date.parse("2024-01-01")
year = d.year()
month = d.month()
weekday = d.weekday()
d2 = d.add_days(7)
```

### Time [ ]

```ky
t = Time.now()
t = Time.from_hms(12, 30, 0)
t = Time.parse("12:30:00")
hour = t.hour()
minute = t.minute()
second = t.second()
```

### Bytes [ ]

```ky
b = Bytes.new(1024)
b = Bytes.from_hex("deadbeef")
b = Bytes.from_base64("SGVsbG8=")
b.len()
val = b.get(0)           # → byte
b.set(0, 255)
hex = b.hex()
b64 = b.to_base64()
```

### Decimal [ ]

```ky
d = Decimal.from_str("3.14")
d = Decimal.from_i64(314, 2)    # 3.14
d.round(1)                       # → 3.1
d.truncate()                     # → 3
d.to_str()                       # → "3.14"
```

### Uuid [ ]

```ky
id = Uuid.v4()
id = Uuid.parse("550e8400-e29b-41d4-a716-446655440000")
id.to_str()
```

### Url [ ]

```ky
u = Url.parse("https://user:pass@host:8080/path?q=1#frag")
u.scheme()     # → "https"
u.host()       # → "host"
u.port()       # → 8080
u.path()       # → "/path"
u.query()      # → "q=1"
```

### Regex [ ]

```ky
re = Regex("[0-9]+")
re.is_match("abc123")       # → true
m = re.find("abc123")       # → "123"
result = re.replace("abc123", "X")  # → "abcX"
```

### Env [ ]

```ky
val = env("PATH")
env("MY_VAR", "value")      # set
```

### Json [ ]

```ky
json = Json.parse('{"name": "Kyle", "age": 30}')
name = json["name"]          # access field
json["city"] = "NYC"         # set field
str = json.stringify()        # serialize
str = json.pretty()           # pretty-print
```

### File [ ]

```ky
f = File.open("/tmp/test.txt", "w")
f.write("hello world")
f.close()

f = File.open("/tmp/test.txt", "r")
content = f.read()           # → str
f.close()
f.exists()                   # → true
```

### Socket [ ]

```ky
server = Socket.tcp_listen(8080)
client = server.accept()
data = client.read(1024)
client.write("HTTP/1.1 200 OK\r\n\r\n")
client.close()
server.close()

# Client mode
conn = Socket.tcp_connect("example.com", 80)
conn.write("GET / HTTP/1.1\r\nHost: example.com\r\n\r\n")
resp = conn.read(4096)
conn.close()
```

### Path [ ]

```ky
p = Path("/home/user/file.txt")
p.dirname()          # → "/home/user"
p.basename()         # → "file.txt"
p.extension()        # → ".txt"
p.exists()           # → true
p.is_file()          # → true
p.is_dir()           # → false
p.join("subdir")     # → "/home/user/file.txt/subdir"
```

### BigInt [ ]

```ky
n = BigInt("123456789012345678901234567890")
n2 = BigInt.from_i64(42)
n3 = n + n2
n4 = n * n2
n.to_str()
```

### strBuilder [ ]

```ky
sb = strBuilder(50000)
sb.append("x", 1)
sb.append("hello", 5)
result = sb.to_str()
```

---

## Status summary

| Categoría | [x] Completo | [ ] Diseñado | ❌ No planeado |
|-----------|:-----------:|:-----------:|:-------------:|
| Primitives | 13 | 2 (`u8-u64` codegen, `never`) | 1 (`byte`) |
| Compounds | 4 | 6 (tuple, Set, Queue, Stack, slice) | 0 |
| Ownership | 3 | 4 (Box, Rc, Arc, Weak) | 0 |
| Concurrency | 1 (async/await) | 7 (Future, Channel, select, Mutex, Atomic, Iterator) | 0 |
| Specialized | 1 (strBuilder) | 14 (DateTime, Duration, Date, Time, Bytes, Decimal, Uuid, Url, Regex, Env, Json, File, Socket, Path) | 0 |
| Total | 22 | 33 | 1 |
