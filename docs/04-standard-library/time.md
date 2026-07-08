# time — Tiempo y Sleep

> Módulo de medición de tiempo y pausas.
> Import: `from time import time`

## time: medición y pausas

```ky
from time import time

# Sleep
time.sleep(1000)           # pausa en milisegundos

# Timestamp
ts = time.now()            # timestamp actual en ms
ts = time.now_ns()         # timestamp actual en nanosegundos

# Medición
start = time.now()
# ... código a medir ...
elapsed = time.now() - start
println("tomó " + elapsed.to_str() + "ms")
```

### Funciones

| Función | Descripción |
|---------|-------------|
| `time.sleep(ms)` | Pausar el hilo actual (milisegundos) |
| `time.now()` | Timestamp actual en milisegundos desde epoch |
| `time.now_ns()` | Timestamp actual en nanosegundos |
| `time.seconds_since(start)` | Segundos desde un timestamp |

### Ejemplo

```ky
from time import time

fn fib(n: i32) i32:
    a: ^i32 = 0
    b: ^i32 = 1
    i: ^i32 = 0
    while i < n:
        tmp = a + b
        a = b
        b = tmp
        i = i + 1
    b

fn main() i32:
    start = time.now()
    result = fib(10000000)
    elapsed = time.now() - start
    println("fib(10M) = " + result.to_str())
    println("tomó " + elapsed.to_str() + "ms")
    0
```
