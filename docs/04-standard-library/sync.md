# sync — Sincronización

> Módulo de primitivas de sincronización entre hilos.
> Import: `from sync import mutex, atomic, channel, barrier`

## mutex: exclusión mutua

```ky
from sync import mutex

m = mutex(0)
i: ^i64 = 0

lock(m):
    i = i + 1      # operación segura entre hilos
```

### Métodos

| Método | Descripción |
|--------|-------------|
| `mutex(initial)` | Crear mutex con valor inicial |
| `lock(m): ...` | Bloquear hasta adquirir (con bloque) |

## atomic: operaciones atómicas

```ky
from sync import atomic

counter = atomic.i64(0)
counter.fetch_add(1)
counter.load()    # → 1

flag = atomic.bool(false)
flag.store(true)
flag.load()       # → true
```

### Métodos

| Método | Descripción |
|--------|-------------|
| `atomic.i64(val)` | Crear contador atómico |
| `atomic.bool(val)` | Crear flag atómico |
| `c.fetch_add(n)` | Incremento atómico |
| `c.load()` | Lectura atómica |
| `c.store(val)` | Escritura atómica |
| `c.compare_and_swap(old, new)` | CAS atómico |

## channel: comunicación entre hilos

```ky
from sync import channel

ch = channel.i64(16)      # buffer de 16
ch.send(42)
val = ch.recv()
ch.len()
ch.close()
```

### Métodos

| Método | Descripción |
|--------|-------------|
| `channel.i64(capacity)` | Crear canal de i64 |
| `ch.send(val)` | Enviar valor |
| `ch.recv()` | Recibir valor (bloquea si vacío) |
| `ch.len()` | Cantidad de elementos en buffer |
| `ch.close()` | Cerrar canal |

## barrier: sincronización de barrera

```ky
from sync import barrier

b = barrier(4)     # 4 hilos deben llegar
# en cada hilo:
b.wait()           # esperar a que lleguen todos
```
