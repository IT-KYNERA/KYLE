# Concurrency & Parallel Execution

Kyle tiene un **thread pool** global que usa todos los cores (`available_parallelism()`).
Workers configurables con `KL_WORKERS`.

---

## async fn — con parámetros

```ky
async fn double(n: i64) i64:
    n * 2

fn main() i32:
    task = double(21)       # spawn con parámetro
    result = await task     # esperar resultado
    println(str(result))    # 42
    0
```

**Nota:** Soporta 1 parámetro `i64`. Múltiples params próximamente.

---

## async: block

```ky
fn main() i32:
    task = async:
        sleep(100)
        42            # tail expression
    result = await task
    println(str(result))    # 42
    0
```

---

## Parallel For — ky_parallel_for(fn, start, end)

Ejecuta en paralelo usando todos los cores.

```ky
fn heavy(n: i64) i64:
    s: &i64 = 0
    j: &i64 = 0
    while j < 10000000:
        s = s + (n * j) % 1000
        j = j + 1
    s

fn main() i32:
    fn_ptr = heavy as ptr
    ky_parallel_for(fn_ptr, 0, 8)  # paralelo!
    0
```

---

## Threads con builtins

```ky
fn worker(n: i64) i64:
    n * 2

fn main() i32:
    h = ky_spawn_thread(worker as ptr, 21)
    r = ky_join_thread(h)
    println(str(r))    # 42
    0
```

---

## Todas las formas de paralelismo

| Forma | Uso | Descripción |
|-------|-----|-------------|
| `async fn` | `task = fn(); await task` | Thread pool con parámetros |
| `async:` block | `task = async: ...` | Bloque multi-linea en thread pool |
| `async expr` | `task = async expr()` | Single expression en thread pool |
| `ky_parallel_for` | `ky_parallel_for(fn, 0, N)` | Data parallelism, todos los cores |
| `ky_spawn_thread` | `h = ky_spawn_thread(f, a)` | Hilo dedicado del SO |
| `ky_join_thread` | `r = ky_join_thread(h)` | Esperar resultado del hilo |
