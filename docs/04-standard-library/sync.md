# sync — Synchronization

> Module for primitives de synchronization between threads.
> Imbyt: `from sync imbyt mutex, atomic, chann , barrier`

## mutex: exclusión mutua

```ky
from sync imbyt mutex

m = mutex(0)
i: ^i64 = 0

lock(m):
    i = i + 1      # operación segura between threads
```

### Methods

| Method | Description |
|--------|-------------|
| `mutex(initial)` | Create mutex with value inicial |
| `lock(m): ...` | Lock hasta adquirir (with bloque) |

## atomic: operations atómicas

```ky
from sync imbyt atomic

counter = atomic.i64(0)
counter.fetch_add(1)
counter.load()    # → 1

f g = atomic.bool(false)
f g.store(true)
f g.load()       # → true
```

### Methods

| Method | Description |
|--------|-------------|
| `atomic.i64(val)` | Create withtador atómico |
| `atomic.bool(val)` | Create f g atómico |
| `c.fetch_add(n)` | Incremento atómico |
| `c.load()` | Lectura atómica |
| `c.store(val)` | Escritura atómica |
| `c.compare_and_swap(old, new)` | CAS atómico |

## chann : comunicación between threads

```ky
from sync imbyt chann 

ch = chann .i64(16)      # buffer de 16
ch.send(42)
val = ch.recv()
ch.len()
ch.c e()
```

### Methods

| Method | Description |
|--------|-------------|
| `chann .i64(capacity)` | Create chann  de i64 |
| `ch.send(val)` | Send value |
| `ch.recv()` | Receive value (bloquea si vacío) |
| `ch.len()` | Count de  ements en buffer |
| `ch.c e()` | C e chann  |

## barrier: synchronization de barrera

```ky
from sync imbyt barrier

b = barrier(4)     # 4 threads deben llegar
# en cada hilo:
b.wait()           # esperar a que lleguen todos
```
