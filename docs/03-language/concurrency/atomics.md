# Atomics

> Operaciones atómicas lock-free para programación concurrente.
> Actualmente no implementadas como tipos Kyle (solo uso interno en runtime Rust).

## Estado actual

| Tipo | Status | Descripción |
|------|--------|-------------|
| `atomic_i64` | 📅 Planeado | Entero atómico de 64 bits |
| `atomic_bool` | 📅 Planeado | Booleano atómico |
| `atomic_ptr` | 📅 Planeado | Puntero atómico |

## Diseño propuesto

```ky
counter: atomic_i64 = atomic_i64(0)

# Operaciones atómicas
counter.store(42)
val: i64 = counter.load()
counter.fetch_add(1)
counter.fetch_sub(1)
counter.compare_and_swap(10, 20)   # CAS
```

### atomic_bool

```ky
flag: atomic_bool = atomic_bool(false)
flag.store(true)
val: bool = flag.load()
```

### Ordenamiento de memoria

```ky
counter.store(42, "relaxed")     # sin barreras
counter.store(42, "acquire")     # acquire semantics
counter.store(42, "release")     # release semantics
counter.store(42, "acqrel")      # acquire + release
counter.store(42, "seqcst")      # sequentially consistent (default)
```

## Verificación

Las operaciones atómicas son **lock-free** (no usan mutex internamente).
Son implementadas con instrucciones CPU como `LDXR`/`STXR` en ARM64.

## Ver también

- `synchronization.md` — Mutex, barriers
- `threads.md` — Hilos del SO
