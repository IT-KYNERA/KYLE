# Synchronization

> Primitivas de sincronización entre hilos. Actualmente no implementadas como tipos Kyle.

## Estado actual

| Primitiva | Status | Descripción |
|-----------|--------|-------------|
| `mutex<T>` | 📅 Planeado | Exclusión mutua con dato protegido |
| `rwlock<T>` | 📅 Planeado | Readers-writer lock |
| `barrier` | 📅 Planeado | Barrera de sincronización |
| `condvar` | 📅 Planeado | Condition variable |
| `once` | 📅 Planeado | Inicialización única |

## Diseño propuesto

### mutex

```ky
m: mutex<i32> = mutex(0)

lock(m):
    *val += 1     # operación segura entre hilos
```

### barrier

```ky
b: barrier = barrier(4)     # esperar a 4 hilos

fn worker(b: &barrier):
    # ... trabajar ...
    b.wait()                 # esperar que todos lleguen
```

### Uso con threads

```ky
m: mutex<i32> = mutex(0)
n: ^i64 = 0

fn worker(arg: i64) i64:
    lock(m):
        *val += 1
    *val

# spawn múltiples workers compartiendo el mutex
```

## Alternativa: channels

Para muchos casos de sincronización, los canales son más seguros y simples:

```ky
ch: i64 = ky_channel_new(16)
ky_channel_send(ch, 42)
val: i64 = ky_channel_recv(ch)
```

## Ver también

- `channels.md` — Comunicación por canales
- `atomics.md` — Operaciones atómicas lock-free
- `threads.md` — Hilos del SO
