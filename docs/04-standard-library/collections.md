# collections — Listas, Sets

> Módulo de colecciones: listas dinámicas, sets, iteradores.
> Import: `from collections import list, set, iter`

## list: `{T}`

Lista dinámica en heap. Es el tipo de colección principal de Kyle.

```ky
from collections import list

v: {i32} = {1, 2, 3}
v.push(4)
v.reserve(100)       # pre-asigna capacidad
x = v[0]             # acceso por índice
v[0] = 99            # asignación por índice
v.pop()              # saca el último (LIFO)
v.pop_first()        # saca el primero (FIFO)
```

### Métodos

| Método | Descripción |
|--------|-------------|
| `push(val)` | Agrega al final |
| `pop()` | Saca el último elemento |
| `pop_first()` | Saca el primer elemento |
| `len()` | Cantidad de elementos |
| `get(idx)` | Obtener elemento (con bounds check) |
| `set(idx, val)` | Asignar elemento |
| `contains(val)` | `true` si existe |
| `insert(idx, val)` | Insertar en posición |
| `remove_at(idx)` | Eliminar en posición |
| `clear()` | Vaciar la lista |
| `reserve(capacity)` | Pre-asignar capacidad |
| `reverse()` | Invertir orden |

### Stack via list

```ky
st = {}
st.push(10)
st.push(20)
val = st.pop()   # → 20 (LIFO)
```

### Queue via list

```ky
q = {}
q.push(10)
q.push(20)
val = q.pop_first()   # → 10 (FIFO)
```

## set: `set<T>`

```ky
from collections import set

s: set<i32> = set{1, 2, 3}
s.add(4)
s.contains(1)    # → true
s.remove(1)
s.len()
for val in s:
    println(val.to_str())
```

### Métodos

| Método | Descripción |
|--------|-------------|
| `add(val)` | Agregar elemento |
| `contains(val)` | `true` si existe |
| `remove(val)` | Eliminar elemento |
| `len()` | Cantidad de elementos |
| `clear()` | Vaciar el set |

## iter: iterator

```ky
from collections import iter

it = list.iter()
doubled = it.map(fn(x): x * 2)
filtered = it.filter(fn(x): x > 5)
result = filtered.collect()     # → {i32}
```

### Métodos

| Método | Descripción |
|--------|-------------|
| `map(fn)` | Transformar cada elemento |
| `filter(fn)` | Filtrar elementos |
| `fold(init, fn)` | Reducir a un valor |
| `collect()` | Recolectar en lista |
| `next()` | Siguiente elemento |
| `sum()` | Sumar todos |
| `min()` | Mínimo |
| `max()` | Máximo |
