# thread — Hilos

> Módulo de hilos del sistema operativo.
> Import: `from thread import thread`

## thread: spawn y join

```ky
from thread import thread

fn worker(n: i64) i64:
    n * 2

h = thread.spawn(worker as ptr, 21)
result = thread.join(h)
println(result.to_str())    # 42
```

### Funciones

| Función | Descripción |
|---------|-------------|
| `thread.spawn(fn_ptr, arg)` | Crear nuevo hilo del SO |
| `thread.join(handle)` | Esperar que el hilo termine |
| `thread.yield()` | Ceder el turno al scheduler |
| `thread.sleep(ms)` | Dormir el hilo actual |
| `thread.id()` | ID del hilo actual |

### Ejemplo

```ky
from thread import thread

fn compute(n: i64) i64:
    i: ^i64 = 0
    result: ^i64 = 0
    while i < n:
        result = result + i
        i = i + 1
    result

h1 = thread.spawn(compute as ptr, 1000000)
h2 = thread.spawn(compute as ptr, 2000000)
r1 = thread.join(h1)
r2 = thread.join(h2)
println("total: " + (r1 + r2).to_str())
```
