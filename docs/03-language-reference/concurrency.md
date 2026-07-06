# Concurrency

## Estado actual

Async/await está implementado pero con limitaciones importantes.
El sistema usa un **thread pool** con canales para pasar resultados.

---

## Async fn (sin parámetros)

```ky
async fn compute() i64:
    sleep(100)
    42

fn main() i32:
    task = compute()        # spawn en thread pool
    result = await task     # esperar y obtener resultado
    println(str(result))    # 42
    0
```

**Importante:** `async fn` actualmente **no soporta parámetros**. Los parámetros se pierden y la función recibe siempre `0`. Tampoco soporta retorno `str` — el tipo de retorno se fuerza a `i64`.

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

**Nota:** `async <expr>` solo acepta una expresión, no un bloque `{ }`.

---

## Await

```ky
task = async sleep(100)
await task                  # esperar sin capturar resultado
result = await task         # capturar resultado (i64)
```

**Advertencia:** El handle se consume al hacer `await`. Llamar `await` dos veces sobre el mismo handle causa **use-after-free** (crash).

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
    # t1 y t2 se ejecutan en paralelo en el thread pool
    r1 = await t1
    r2 = await t2
    println(str(r1) + " " + str(r2))  # 10 20
    0
```

---

## Threads (low-level)

```ky
extern fn ky_spawn_thread(fn_ptr: ptr, arg: i64) i64
extern fn ky_join_thread(handle: i64) i64

fn worker(n: i64) i64:
    n * 2

fn main() i32:
    fn_addr = worker as ptr
    h = ky_spawn_thread(fn_addr, 21)
    r = ky_join_thread(h)
    println(str(r))          # 42
    0
```

---

## Limitaciones conocidas

| Problema | Detalle | ¿Fix planeado? |
|----------|---------|----------------|
| Parámetros perdidos en `async fn` | El wrapper spawn con `arg=0`, los params reales se ignoran | 🔜 Futuro |
| Return type forzado a `i64` | `async fn` que retorna `str` da basura | 🔜 Futuro |
| No `async { }` block | Solo `async <expr>` | 🔜 Futuro |
| Handle consume-after-await | `await` dos veces sobre el mismo handle crashea | 🔜 Futuro |
| Thread/Channel no son builtins | Hay que usar `extern fn` para `ky_spawn_thread` | 🔜 Próximo release |
