# Stack vs Heap

> Cómo Kyle organiza la memoria: stack para valores locales, heap para datos dinámicos.

## Stack

El stack almacena variables locales, parámetros de función y valores temporales.
Es rápido (solo mover un puntero) y automático.

```ky
fn ejemplo() i32:
    x: i32 = 42       # stack
    y: f64 = 3.14     # stack
    z: [i32; 3] = [1, 2, 3]  # stack (array de tamaño fijo)
    x + y as i32 + z[0]
```

### Qué va al stack

| Tipo | Tamaño en stack |
|------|----------------|
| `i32` | 4 bytes |
| `i64` | 8 bytes |
| `f64` | 8 bytes |
| `bool` | 1 byte |
| `ptr` | 8 bytes |
| `[T; N]` | `N * size(T)` |
| `str` | 8 bytes (puntero al heap) |
| `{T}` | 8 bytes (puntero al heap) |

## Heap

El heap almacena datos de tamaño dinámico o que deben persistir más allá de la
función actual. Los strings, listas y diccionarios viven en heap.

```ky
fn ejemplo() str:
    s: str = "Hola, mundo!"     # datos en heap, puntero en stack
    v: {i32} = {1, 2, 3}       # datos en heap, puntero en stack
    s
```

### Qué va al heap

| Tipo | Datos en heap |
|------|---------------|
| `str` | Caracteres + null terminator |
| `{T}` | Array de elementos + metadata (len, cap) |
| `{K: V}` | Hash table entries |
| `Box<T>` | Valor T |

## Representación en memoria

```
Stack:
┌─────────────┐
│ s: ptr ─────┼─────► ┌──────────────────────┐
│ v: ptr ─────┼──┐   │ "Hola, mundo!\0"     │
│ x: i32 = 42 │  │   └──────────────────────┘
│             │  │   ┌──────────────────────┐
└─────────────┘  └──►│ data: ptr ──► [1,2,3] │
                      │ len: 3               │
                      │ cap: 4               │
                      └──────────────────────┘
```

## Strings (SSO planeado)

Actualmente todos los strings van al heap. En el futuro, strings ≤ 15 bytes
se almacenarán inline (Small String Optimization).

## Ver también

- `move.md` — Cómo se mueven los datos entre stack y heap
- `allocator.md` — Estrategia de asignación de heap
