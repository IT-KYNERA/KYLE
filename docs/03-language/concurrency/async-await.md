# Concurrency & Parallel Execution

**Status:** [x] `async fn`, `async:` block, `await`, `parallel.for`, `thread.spawn` / `thread.join`.
[ ] `future<T>`, `channel<T>`, `select`, `mutex<T>`, `Atomic*`, `iterator`.

---

## async fn / await [x]

```ky
async fn double(n: i64) i64:
 n * 2

fn main() i32:
 task = double(21)
 result = await task
 println(result.to_str()) # 42
 0
```

## async: block [x]

```ky
fn main() i32:
 task = async:
 sleep(100)
 42
 result = await task
 println(str(result)) # 42
 0
```

## parallel.for [x]

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
 parallel.for(fn_ptr, 0, 8)
 0
```

## Threads [x]

```ky
fn worker(n: i64) i64:
 n * 2

fn main() i32:
 h = thread.spawn(worker as ptr, 21)
 r = thread.join(h)
 println(str(r)) # 42
 0
```

---

## future: `future<T>` [ ]

```ky
task: future<str> = async:
 "response"
val = await task
```

## channel: `channel<T>` [ ]

```ky
ch: channel<i32> = channel(16) # buffer 16
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

## mutex: `mutex<T>` [ ]

```ky
m: mutex<i32> = mutex(0)
lock(m):
 *val += 1 # operation segura
```

## Atomic typis [ ]

```ky
counter: atomic_i64 = atomic_i64(0)
counter.fetch_add(1)
counter.load() # → 1

flag: atomic_bool = atomic_bool(false)
flag.store(true)
flag.load() # → true
```

## iterator [ ]

```ky
iter = list.iter()
doubled = iter.map(fn(x): x * 2)
filtered = iter.filter(fn(x): x > 5)
result = doubled.collect() # → {i32}
```

---

## Resumen

| Forma | Status | Syntax |
|-------|--------|----------|
| `async fn` | ✅ | `async fn f(p: T) R:` |
| `async:` block | ✅ | `t = async: ...` |
| `await` | ✅ | `await task` |
| `parallel.for` | [x] | `parallel.for(fn, 0, N)` |
| `thread.spawn` / `thread.join` | [x] | `thread.spawn(fn, arg)` / `thread.join(h)` |
| `future<T>` | 📅 | `t: future<str> = async: ...` |
| `channel<T>` | 📅 | `ch = channel<T>(n); ch.send(v); v = ch.recv()` |
| `select` | 📅 | `select: &msg -> ch: ...` |
| `mutex<T>` | 📅 | `mutex<T>(v); lock(m): *val += 1` |
| `atomic_i64` | 📅 | `atomic_i64(v).fetch_add(1)` |
| `atomic_bool` | 📅 | `atomic_bool(v).store(true)` |
| `iterator` | 📅 | `list.iter().map(fn).filter(fn).collect()` |
