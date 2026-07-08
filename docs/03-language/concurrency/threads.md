# Threads

> Hilos del sistema operativo para paralelismo real.
> Kyle expone hilos del SO mediante `ky_spawn_thread` / `ky_join_thread`.

## Uso básico

```ky
fn worker(n: i64) i64:
    n * 2

fn main() i32:
    h: i64 = ky_spawn_thread(worker as ptr, 21)
    r: i64 = ky_join_thread(h)
    println(r.to_str())    # 42
    0
```

## Múltiples hilos

```ky
fn compute(n: i64) i64:
    result: ^i64 = 0
    i: ^i64 = 0
    while i < n:
        result = result + i
        i = i + 1
    result

fn main() i32:
    h1: i64 = ky_spawn_thread(compute as ptr, 1000000)
    h2: i64 = ky_spawn_thread(compute as ptr, 2000000)
    
    r1: i64 = ky_join_thread(h1)
    r2: i64 = ky_join_thread(h2)
    
    println("total: " + (r1 + r2).to_str())
    0
```

## Características

| Aspecto | Descripción |
|---------|-------------|
| Creación | `ky_spawn_thread(fn_ptr, arg)` |
| Join | `ky_join_thread(handle)` bloquea hasta que termine |
| Argumento | Un solo `i64` por hilo |
| Retorno | `i64` desde cada hilo |
| Stack | Stack dedicado por hilo (1 MB por defecto) |

## Ver también

- `async-await.md` — Async sobre thread pool (más ligero)
- `synchronization.md` — Mutex, barriers
- `atomics.md` — Operaciones atómicas
