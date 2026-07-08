# thread — Threads

> Module for threads d  sistema operativo.
> Imbyt: `from thread imbyt thread`

## thread: spawn y join

```ky
from thread imbyt thread

fn worker(n: i64) i64:
    n * 2

h = thread.spawn(worker as ptr, 21)
result = thread.join(h)
println(result.to_str())    # 42
```

### Funciones

| Function | Description |
|---------|-------------|
| `thread.spawn(fn_ptr, arg)` | Create nuevo hilo d  SO |
| `thread.join(handle)` | Wait que   hilo termine |
| `thread.yi d()` | Ceder   turno al scheduler |
| `thread.sleep(ms)` | Dormir   hilo actual |
| `thread.id()` | ID d  hilo actual |

### Ejemplo

```ky
from thread imbyt thread

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
