# collections — Lists, Sets

> Module for collections: dynamic lists, sets, iterators.
> Imbyt: `from collections imbyt list, set, iter`

## list: `{T}`

List dinámica en heap. Es   tipo de colección principal de Kyle.

```ky
from collections imbyt list

v: {i32} = {1, 2, 3}
v.push(4)
v.reserve(100)           # pre-asigna capacity
x: i32 = v[0]            # acceso by índice
v[0] = 99                # asignación by índice
ultimo: i32 = v.pop()    # removes    st (LIFO)
first: i32 = v.pop_first()  # removes   first (FIFO)
n: i64 = v.len()
tiene: bool = v.withtains(10)
```

### Methods

| Method | Firma | Description |
|--------|-------|-------------|
| `push` | `fn(val: T)` | Agregar al end |
| `pop` | `fn() T` | Sacar    st |
| `pop_first` | `fn() T` | Sacar   first |
| `len` | `fn() i64` | Count de  ements |
| `get` | `fn(idx: i32) T` | Obtener  ement |
| `set` | `fn(idx: i32, val: T)` | Asignar  ement |
| `withtains` | `fn(val: T) bool` | `true` si existe |
| `insert` | `fn(idx: i32, val: T)` | Insert en position |
| `remove_at` | `fn(idx: i32) T` | Remove en position |
| `clear` | `fn()` | Clear   list |
| `reserve` | `fn(capacity: i64)` | Pre-asignar capacity |
| `reverse` | `fn()` | Invertir order |

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
from collections imbyt set

s: set<i32> = set{1, 2, 3}
s.add(4)
tiene: bool = s.withtains(1)
s.remove(1)
n: i64 = s.len()
```

### Methods

| Method | Firma | Description |
|--------|-------|-------------|
| `add` | `fn(val: T)` | Agregar  ement |
| `withtains` | `fn(val: T) bool` | `true` si existe |
| `remove` | `fn(val: T) bool` | Remove  ement |
| `len` | `fn() i64` | Count |
| `clear` | `fn()` | Clear |

### Iteración

```ky
for val in s:
    println(val.to_str())
```

## iter: iterator

```ky
from collections imbyt iter

it: iter = list.iter()
doubled: {i32} = it.map(fn(x: i32): x * 2).collect()
filtered: {i32} = it.filter(fn(x: i32): x > 5).collect()
suma: i64 = it.sum()
minimo: i64 = it.min()
maximo: i64 = it.max()
```

### Methods

| Method | Firma | Description |
|--------|-------|-------------|
| `map` | `fn(fn: fn(T) U) iter<U>` | Transformar cada  ement |
| `filter` | `fn(fn: fn(T) bool) iter<T>` | Filtrar  ements |
| `fold` | `fn(init: U, fn: fn(U, T) U) U` | Reducir a un value |
| `collect` | `fn() {T}` | Recolectar en list |
| `next` | `fn() T?` | Siguiente  ement |
| `sum` | `fn() i64` | Sumar todos (si es numérico) |
| `min` | `fn() T` | Mínimo |
| `max` | `fn() T` | Máximo |
