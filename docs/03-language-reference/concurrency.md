# Concurrency & Parallel Execution

## Thread Pool

Kyle tiene un **thread pool** global que usa **todos los cores de la CPU**.
Workers: `available_parallelism()` (ej: 8 en M3 Max). Configurable via `KL_WORKERS`.

---

## Parallel For — `ky_parallel_for(fn_ptr, start, end)`

Ejecuta una función en paralelo para un rango de índices.
Distribuye el trabajo entre todos los cores del thread pool.

```ky
fn trabajo_pesado(n: i64) i64:
    s: &i64 = 0
    j: &i64 = 0
    while j < 10000000:
        s = s + (n * j) % 1000
        j = j + 1
    s

fn main() i32:
    fn_ptr = trabajo_pesado as ptr

    # Secuencial
    t1 = now()
    i: &i64 = 0
    while i < 8:
        trabajo_pesado(i)      # uno tras otro
        i = i + 1
    t2 = now()
    print("seq: " + str(t2 - t1) + "ms\n")

    # Paralelo — usa TODOS los cores
    t1 = now()
    ky_parallel_for(fn_ptr, 0, 8)  # todos a la vez
    t2 = now()
    print("par: " + str(t2 - t1) + "ms\n")
    0
```

**Nota:** `ky_parallel_for` descarta los valores de retorno. La función debe tener efectos secundarios
(como escribir a un array o acumular en variables compartidas).

---

## Async fn

```ky
async fn compute() i64:
    sleep(100)
    42

fn main() i32:
    task = compute()        # spawn en thread pool
    result = await task     # esperar resultado
    println(str(result))    # 42
    0
```

**Limitación:** `async fn` actualmente **no soporta parámetros** — se pierden.
Tampoco soporta retorno `str` (forzado a `i64`).

---

## Async expression

```ky
fn slow_add(a: i64) i64:
    sleep(50)
    a + 1

fn main() i32:
    task = async slow_add(41)
    result = await task
    println(str(result))    # 42
    0
```

**Nota:** `async <expr>` solo acepta una expresión, no un bloque.

---

## Concurrencia (múltiples tareas)

```ky
async fn task1() i64:
    sleep(100)
    10

async fn task2() i64:
    sleep(50)
    20

fn main() i32:
    t1 = task1()
    t2 = task2()
    # t1 y t2 se ejecutan en paralelo
    r1 = await t1
    r2 = await t2
    println(str(r1) + " " + str(r2))  # 10 20
    0
```

---

## Threads (low-level)

```ky
fn worker(n: i64) i64:
    n * 2

fn main() i32:
    fn_addr = worker as ptr
    h = ky_spawn_thread(fn_addr, 21)   # hilo dedicado
    r = ky_join_thread(h)              # esperar resultado
    println(str(r))                     # 42
    0
```

---

## Ejemplo: procesar lista en paralelo

```ky
fn process(n: i64) i64:
    n * n

fn main() i32:
    items = {1, 2, 3, 4, 5, 6, 7, 8}
    tasks = {}
    
    # Lanzar todos en paralelo
    for item in items:
        tasks.push(ky_spawn_thread(process as ptr, item))
    
    # Recoger resultados
    results = {}
    for task in tasks:
        results.push(ky_join_thread(task))
    
    print("results: " + str(len(results)) + "\n")
    0
```

---

## Configuración

```bash
export KL_WORKERS=16    # thread pool size (default: número de cores)
```

---

## Limitaciones conocidas

| Problema | Detalle |
|----------|---------|
| Parámetros perdidos en `async fn` | El wrapper spawn con `arg=0` |
| Return type forzado a `i64` | `async fn` retornando `str` da basura |
| No `async { }` block | Solo `async <expr>` |
| Handle consume-after-await | `await` dos veces sobre el mismo handle crashea |
