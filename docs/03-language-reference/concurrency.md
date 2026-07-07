# Concurrency & Parallel Execution

**Status:** [x] `async fn`, `async:` block, `await`, `parallelFor`, threads.
[ ] `Future<T>`, `Channel<T>`, `select`, `Mutex<T>`, `Atomic*`, `Iterator`.

---

## async fn / await [x]

```ky
async fn double(n: i64) i64:
    n * 2

fn main() i32:
    task = double(21)
    result = await task
    println(str(result))    # 42
    0
```

## async: block [x]

```ky
fn main() i32:
    task = async:
        sleep(100)
        42
    result = await task
    println(str(result))    # 42
    0
```

## parallelFor [x]

```ky
fn heavy(n: i64) i64:
    s: ^i64 = 0
    j: ^i64 = 0
    while j < 10000000:
        s = s + (n * j) % 1000
        j = j + 1
    s

fn main() i32:
    fn_ptr = heavy as ptr
    parallelFor(fn_ptr, 0, 8)
    0
```

## Threads [x]

```ky
fn worker(n: i64) i64:
    n * 2

fn main() i32:
    h = spawnThread(worker as ptr, 21)
    r = joinThread(h)
    println(str(r))    # 42
    0
```

---

## Future: `Future<T>` [ ]

```ky
task: Future<str> = async:
    "response"
val = await task
```

## Channel: `Channel<T>` [ ]

```ky
ch: Channel<i32> = Channel(16)     # buffer 16
ch.send(42)
val = ch.recv()
ch.len()
ch.close()
```

## select [ ]

```ky
select:
    &msg -> ch1:
        println("got: " + msg)
    &msg -> ch2:
        println("got: " + msg)
    after 1s:
        println("timeout")
```

## Mutex: `Mutex<T>` [ ]

```ky
m: Mutex<i32> = Mutex(0)
lock(m):
    *val += 1                 # operación segura
```

## Atomic types [ ]

```ky
counter: AtomicI64 = AtomicI64(0)
counter.fetch_add(1)
counter.load()                # → 1

flag: AtomicBool = AtomicBool(false)
flag.store(true)
flag.load()                   # → true
```

## Iterator [ ]

```ky
iter = list.iter()
doubled = iter.map(fn(x): x * 2)
filtered = iter.filter(fn(x): x > 5)
result = doubled.collect()     # → {i32}
```

---

## Resumen

| Forma | Estado | Sintaxis |
|-------|--------|----------|
| `async fn` | ✅ | `async fn f(p: T) R:` |
| `async:` block | ✅ | `t = async: ...` |
| `await` | ✅ | `await task` |
| `parallelFor` | 🔶 renombrar | `parallelFor(fn, 0, N)` |
| threads | 🔶 renombrar | `spawnThread` / `joinThread` |
| `future<T>` | 📅 | `t: future<str> = async: ...` |
| `channel<T>` | 📅 | `ch = channel<T>(n); ch.send(v); v = ch.recv()` |
| `select` | 📅 | `select: &msg -> ch: ...` |
| `mutex<T>` | 📅 | `mutex<T>(v); lock(m): *val += 1` |
| `atomicI64` | 📅 | `atomicI64(v).fetchAdd(1)` |
| `atomicBool` | 📅 | `atomicBool(v).store(true)` |
| `iterator` | 📅 | `list.iter().map(fn).filter(fn).collect()` |
