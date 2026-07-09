# Collections — Lists, Arrays, Dicts, Sets, Iterators

> All collection types in Kyle: dynamic lists, fixed arrays, hash dicts, hash sets, and lazy iterators.
> Every example includes ownership semantics (`&` borrow, `^&` mutable borrow, move).

---

## List `{T}`

Lista dinámica en heap. **Trabaja por valor:** las operaciones buscan, agregan y eliminan por valor, no por posición. Si necesitas acceso por índice, usa arrays.

### Creación

```ky
vacia: {i32} = {0}; vacia.pop()   # inicializar y vaciar
nums: {i32} = {1, 2, 3}           # con valores
nums.push(4)                       # → {1, 2, 3, 4}
```

### Iteración (`for`)

Kyle tiene tres modos de iterar, cada uno con distintas semánticas de ownership:

```ky
libros: {str} = {0 as i64}; libros.pop()
libros.push("El Quijote")
libros.push("Rayuela")
libros.push("Cien Años")

# 1. ITERAR POR VALOR (move) — consume la lista
for libro in libros:
    println(libro)       # cada string se mueve a 'libro'
    # 'libro' se libera al final de cada iteración
# libros ya NO es accesible (move)

# 2. ITERAR POR BORROW (inmutable) — solo lectura
for libro in &libros:
    println(libro)       # prestado, no consumido
# libros sigue siendo accesible

# 3. ITERAR POR MUT BORROW — para modificar elementos
for libro in ^&libros:
    # libro es ^&str — puedes mutarlo
    if libro.contains("Quijote"):
        libro.remove("Quijote")
        libro.push("Quijote de la Mancha")
```

### Modificar durante la iteración

Para modificar (agregar, eliminar, reemplazar) mientras recorres, usa un bucle `while` con índice:

```ky
fn limpiar_nombres(nombres: ^&{str}, suffix: str):
    i = 0
    while i < nombres.len():
        nombre = nombres.get(i)
        if nombre.endswith(suffix):
            nombres.remove_at(i)   # eliminar este, no avanzar
        else:
            i = i + 1              # solo avanzar si no eliminamos

fn duplicar_pares(numeros: ^&{i32}):
    i = 0
    while i < numeros.len():
        val = numeros.get(i)
        if val % 2 == 0:
            numeros.set(i, val * 2)
        i = i + 1
```

### Búsqueda por valor

```ky
fn encontrar_por_titulo(biblioteca: &{str}, titulo: str) bool:
    for libro in &biblioteca:
        if libro == titulo:
            return true
    false

# O directamente:
if biblioteca.contains(titulo):
    println("encontrado")
```

### Eliminar por valor

```ky
fn eliminar_ceros(nums: ^&{i32}):
    nums.remove(0)      # elimina el primer 0 que encuentra
    # Para eliminar TODOS los ceros:
    while nums.contains(0):
        nums.remove(0)
```

### Ownership — Resumen

| Operación | Firma | Ownership |
|-----------|-------|-----------|
| `list.push(val)` | `(^&{T}, T)` | Borrow mutable + move del valor |
| `list.pop()` | `(^&{T}) T` | Borrow mutable, retorna valor movido |
| `list.get(i)` | `(&{T}, i64) T` | Borrow inmutable |
| `list.set(i, val)` | `(^&{T}, i64, T)` | Borrow mutable |
| `list.contains(val)` | `(&{T}, T) bool` | Borrow inmutable |
| `list.remove(val)` | `(^&{T}, T) i32` | Borrow mutable |
| `for x in list` | — | **Move** (consume la lista) |
| `for x in &list` | — | **Borrow** (no consume) |
| `for x in ^&list` | — | **Mut borrow** (puedes mutar) |

### Stack (LIFO) vía list

```ky
pila: {i32} = {0}; pila.pop()
pila.push(10); pila.push(20); pila.push(30)
valor = pila.pop()  # → 30
```

### Queue (FIFO) vía list

```ky
cola: {i32} = {0}; cola.pop()
cola.push(10); cola.push(20)
valor = cola.pop_first()  # → 10
```

### Métodos completo

| Método | Firma | Descripción | Ownership |
|--------|-------|-------------|-----------|
| `push` | `fn(val: T)` | Agregar al final | `^&` + move |
| `pop` | `fn() T` | Sacar del final (LIFO) | `^&` |
| `pop_first` | `fn() T` | Sacar del inicio (FIFO) | `^&` |
| `len` | `fn() i64` | Cantidad de elementos | `&` |
| `get` | `fn(idx: i64) T` | Obtener por índice | `&` |
| `set` | `fn(idx: i64, val: T)` | Asignar por índice | `^&` |
| `contains` | `fn(val: T) bool` | `true` si el valor existe | `&` |
| `remove` | `fn(val: T) i32` | Eliminar por valor (1=encontrado) | `^&` |
| `remove_at` | `fn(idx: i64) T` | Eliminar por índice | `^&` |
| `insert` | `fn(idx: i64, val: T)` | Insertar en posición | `^&` |
| `clear` | `fn()` | Vaciar la lista | `^&` |
| `reserve` | `fn(capacity: i64)` | Pre-asignar capacidad | `^&` |
| `reverse` | `fn()` | Invertir orden | `^&` |

---

## Array `[T; N]`

Array nativo en **stack**. Tamaño fijo en compile-time.
**Acceso por índice únicamente** — es más rápido y contiguo que una lista.

```ky
# Declaración
arr: [5]i32 = [1, 2, 3, 4, 5]
repetido: [100]i32 = [0; 100]   # 100 ceros

# Lectura/escritura por índice (O(1), GEP directo)
x = arr[2]      # load
arr[2] = 99     # store

# Longitud
n = arr.len()   # → 5

# Iterar
for i in 0..arr.len():
    println(arr[i].to_str())

# Los arrays se copian por valor (stack)
arr2 = arr          # COPIA (no move)
arr[0] = 0          # no afecta a arr2

# Para evitar copia, usa borrow
fn sumar(arr: &[100]i32) i64:
    total = 0
    for i in 0..arr.len():
        total = total + arr[i]
    total
```

### Ownership

| Operación | Ownership |
|-----------|-----------|
| `arr[i]` | Borrow inmutable (`&`) |
| `arr[i] = val` | Borrow mutable (`^&`) |
| `y = arr` | **Copia** (stack, no move) |
| `fn f(a: [100]i32)` | Copia (pasaje por valor) |
| `fn f(a: &[100]i32)` | Borrow (sin copia) |

### Cuándo usar lista vs array

| Criterio | Lista `{T}` | Array `[T; N]` |
|----------|-------------|----------------|
| Tamaño | Dinámico (crece/decrece) | Fijo (compile-time) |
| Memoria | Heap | Stack |
| Acceso | Por valor (contains, remove) | Por índice (`arr[i]`) |
| Performance | O(1) amortizado push/pop | O(1) GEP directo |
| Iteración | `for x in &list` | `for i in 0..n` |
| Copia | Move por defecto | Copia (stack) |

---

## Dict `{K: V}`

Diccionario hash en heap. Acceso por clave O(1) promedio.
Usa funciones del módulo `dict` (notación de namespace).

```ky
d: {str: i32} = {}
dict.set(d, "clave", 42)
val = dict.get(d, "clave")

# Verificar si una clave existe
if dict.contains(d, "clave"):
    println("existe")

# Eliminar por clave
dict.remove(d, "clave")

# Cantidad de pares
n = dict.len(d)

# Vaciar
dict.clear(d)

# Iterar claves
for key in &d:
    println(key)
```

### Métodos

| Método | Firma | Descripción | Ownership |
|--------|-------|-------------|-----------|
| `dict.set` | `fn(d: ^&{K: V}, key: K, val: V)` | Asignar por clave | `^&` |
| `dict.get` | `fn(d: &{K: V}, key: K) V` | Obtener por clave | `&` |
| `dict.contains` | `fn(d: &{K: V}, key: K) bool` | `true` si la clave existe | `&` |
| `dict.remove` | `fn(d: ^&{K: V}, key: K)` | Eliminar por clave | `^&` |
| `dict.len` | `fn(d: &{K: V}) i64` | Cantidad de pares | `&` |
| `dict.clear` | `fn(d: ^&{K: V})` | Vaciar | `^&` |

---

## Set `set<T>`

Hash set sin duplicados. Búsqueda O(1) promedio.

```ky
s: set<i32> = set{1, 2, 3}
s.add(4)
tiene = s.contains(1)   # true
s.remove(1)              # elimina 1
n = s.len()

# Iterar
for val in &s:
    println(val.to_str())
```

### Métodos

| Método | Firma | Descripción | Ownership |
|--------|-------|-------------|-----------|
| `add` | `fn(val: T)` | Agregar elemento | `^&` |
| `contains` | `fn(val: T) bool` | `true` si existe | `&` |
| `remove` | `fn(val: T) bool` | Eliminar por valor | `^&` |
| `len` | `fn() i64` | Cantidad | `&` |
| `clear` | `fn()` | Vaciar | `^&` |

---

## Iterator

Iterador **lazy** sobre listas. No asigna hasta `collect()`.

```ky
nums: {i32} = {0}; nums.pop()
nums.push(1); nums.push(2); nums.push(3); nums.push(4); nums.push(5)

# Crear iterador
it = nums.iter()

# Map (transformar cada elemento)
dobles = it.map(fn(x: i32): x * 2).collect()  # asigna nueva lista

# Filter (filtrar)
pares = nums.iter().filter(fn(x: i32): x % 2 == 0).collect()

# Fold (reducir)
suma = nums.iter().fold(0, fn(acc: i64, x: i64): acc + x)

# Chain
resultado = nums.iter()
    .map(fn(x: i32): x * 2)
    .filter(fn(x: i32): x > 5)
    .collect()
```

### Métodos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `next` | `fn() T?` | Siguiente elemento (None si termina) |
| `map` | `fn(fn(T) U) iter<U>` | Transformar cada elemento |
| `filter` | `fn(fn(T) bool) iter<T>` | Filtrar elementos |
| `fold` | `fn(init: U, fn(U, T) U) U` | Reducir a un valor |
| `collect` | `fn() {T}` | Recolectar en nueva lista |
| `sum` | `fn() i64` | Sumar todos (numérico) |
| `min` | `fn() T` | Mínimo valor |
| `max` | `fn() T` | Máximo valor |

---

## Comparativa de ownership

| Operación | Copy type (i32, f64, bool) | Move type (str, {T}, struct) |
|-----------|---------------------------|------------------------------|
| `for x in col` | Copia cada elemento | Move (consume) |
| `for x in &col` | Borrow inmutable | Borrow inmutable |
| `for x in ^&col` | Mut borrow | Mut borrow |
| `fn f(col: {T})` | N/A (no Copy) | Move (ownership transfer) |
| `fn f(col: &{T})` | N/A | Borrow |
| `fn f(col: ^&{T})` | N/A | Mut borrow |
| `y = col` | Copia | **Move** (source inválido) |
| `y = col.clone()` | Copia | Copia explícita |

---

## Comparativa de rendimiento

| Operación | Array `[T;N]` | Lista `{T}` | Set `set<T>` | Dict `{K: V}` |
|-----------|:------------:|:-----------:|:------------:|:--------------:|
| Get por índice/clave | O(1) GEP | O(1) GEP | — | O(1) hash |
| Set por índice/clave | O(1) GEP | O(1) GEP | — | O(1) hash |
| Push al final | — | O(1)* | — | — |
| Pop del final | — | O(1) | — | — |
| Contains | O(n) for | O(n) for | **O(1)** hash | **O(1)** hash |
| Remove por valor/clave | — | O(n) find+shift | **O(1)** hash | **O(1)** hash |
| Insert medio | — | O(n) shift | — | — |
| Memoria | Stack (N × sizeof) | Heap + ptr | Heap + hash | Heap + hash |

*\* Amortizado, con crecimiento ocasional O(n).*

---

## See also

- `compound-types.md` — Sintaxis de tipos compuestos (arrays, listas, tuplas, dicts)
- `ownership.md` — Reglas de ownership y borrowing
- `primitive-types.md` — Tipos primitivos (`i32`, `str`, `bool`, etc.)
