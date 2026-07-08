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
| `push` | `fn(self, val: T)` | Agregar al final |
| `pop` | `fn(self) T` | Sacar el último |
| `pop_first` | `fn(self) T` | Sacar el primero |
| `len` | `fn(self) i64` | Cantidad de elementos |
| `get` | `fn(self, idx: i32) T` | Obtener elemento |
| `set` | `fn(self, idx: i32, val: T)` | Asignar elemento |
| `contains` | `fn(self, val: T) bool` | `true` si existe |
| `insert` | `fn(self, idx: i32, val: T)` | Insertar en posición |
| `remove_at` | `fn(self, idx: i32) T` | Eliminar en posición |
| `clear` | `fn(self)` | Vaciar la lista |
| `reserve` | `fn(self, capacity: i64)` | Pre-asignar capacidad |
| `reverse` | `fn(self)` | Invertir orden |

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
| `add` | `fn(self, val: T)` | Agregar elemento |
| `contains` | `fn(self, val: T) bool` | `true` si existe |
| `remove` | `fn(self, val: T) bool` | Eliminar elemento |
| `len` | `fn(self) i64` | Cantidad |
| `clear` | `fn(self)` | Vaciar |

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
| `map` | `fn(self, fn: fn(T) U) iter<U>` | Transformar cada elemento |
| `filter` | `fn(self, fn: fn(T) bool) iter<T>` | Filtrar elementos |
| `fold` | `fn(self, init: U, fn: fn(U, T) U) U` | Reducir a un valor |
| `collect` | `fn(self) {T}` | Recolectar en lista |
| `next` | `fn(self) T?` | Siguiente elemento |
| `sum` | `fn(self) i64` | Sumar todos (si es numérico) |
| `min` | `fn(self) T` | Mínimo |
| `max` | `fn(self) T` | Máximo |
