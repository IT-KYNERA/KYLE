# collections — Listas, Sets

> Módulo de colecciones: listas dinámicas, sets, iteradores.
> Import: `from collections import list, set, iter`

## list: `{T}`

Lista dinámica en heap. Es el tipo de colección principal de Kyle.

```ky
from collections import list

v: {i32} = {1, 2, 3}
v.push(4)
v.reserve(100)           # pre-asigna capacidad
x: i32 = v[0]            # acceso por índice
v[0] = 99                # asignación por índice
ultimo: i32 = v.pop()    # saca el último (LIFO)
primero: i32 = v.pop_first()  # saca el primero (FIFO)
n: i64 = v.len()
tiene: bool = v.contains(10)
```

### Métodos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `push` | `fn(val: T)` | Agregar al final |
| `pop` | `fn() T` | Sacar el último |
| `pop_first` | `fn() T` | Sacar el primero |
| `len` | `fn() i64` | Cantidad de elementos |
| `get` | `fn(idx: i32) T` | Obtener elemento |
| `set` | `fn(idx: i32, val: T)` | Asignar elemento |
| `contains` | `fn(val: T) bool` | `true` si existe |
| `insert` | `fn(idx: i32, val: T)` | Insertar en posición |
| `remove_at` | `fn(idx: i32) T` | Eliminar en posición |
| `clear` | `fn()` | Vaciar la lista |
| `reserve` | `fn(capacity: i64)` | Pre-asignar capacidad |
| `reverse` | `fn()` | Invertir orden |

### Stack via list

```ky
st: {i32} = {}
st.push(10)
st.push(20)
val: i32 = st.pop()   # → 20 (LIFO)
```

### Queue via list

```ky
q: {i32} = {}
q.push(10)
q.push(20)
val: i32 = q.pop_first()   # → 10 (FIFO)
```

## set: `set<T>`

```ky
from collections import set

s: set<i32> = set{1, 2, 3}
s.add(4)
tiene: bool = s.contains(1)
s.remove(1)
n: i64 = s.len()
```

### Métodos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `add` | `fn(val: T)` | Agregar elemento |
| `contains` | `fn(val: T) bool` | `true` si existe |
| `remove` | `fn(val: T) bool` | Eliminar elemento |
| `len` | `fn() i64` | Cantidad |
| `clear` | `fn()` | Vaciar |

### Iteración

```ky
for val in s:
    println(val.to_str())
```

## iter: iterator

```ky
from collections import iter

it: iter = list.iter()
doubled: {i32} = it.map(fn(x: i32): x * 2).collect()
filtered: {i32} = it.filter(fn(x: i32): x > 5).collect()
suma: i64 = it.sum()
minimo: i64 = it.min()
maximo: i64 = it.max()
```

### Métodos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `map` | `fn(fn: fn(T) U) iter<U>` | Transformar cada elemento |
| `filter` | `fn(fn: fn(T) bool) iter<T>` | Filtrar elementos |
| `fold` | `fn(init: U, fn: fn(U, T) U) U` | Reducir a un valor |
| `collect` | `fn() {T}` | Recolectar en lista |
| `next` | `fn() T?` | Siguiente elemento |
| `sum` | `fn() i64` | Sumar todos (si es numérico) |
| `min` | `fn() T` | Mínimo |
| `max` | `fn() T` | Máximo |
