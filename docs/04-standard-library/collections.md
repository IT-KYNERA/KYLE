# collections — Lists, Sets, Iterators

> Dynamic collections: lists, sets, iterators.

## List `{T}`

Lista dinámica en heap. **Las listas trabajan por valor** — búsqueda, inserción y eliminación son por valor, no por índice. Si necesitas acceso por índice, usa arrays `[T; N]`.

```ky
libros: {str} = {}
libros.push("El Quijote")
libros.push("Cien Años de Soledad")
libros.push("Rayuela")

# Iterar por valor
for libro in &libros:
    println(libro)

# Buscar por valor
if libros.contains("Rayuela"):
    println("encontrado")

# Eliminar por valor (primera ocurrencia)
libros.remove("Cien Años de Soledad")

# Eliminar por índice (solo cuando sabes la posición exacta)
libros.remove_at(0)

# Obtener y sacar del final (stack LIFO)
ultimo = libros.pop()
```

### Métodos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `push` | `fn(val: T)` | Agregar al final |
| `pop` | `fn() T` | Sacar del final (LIFO) |
| `pop_first` | `fn() T` | Sacar del inicio (FIFO) |
| `len` | `fn() i64` | Cantidad de elementos |
| `get` | `fn(idx: i64) T` | Obtener por índice (para compatibilidad) |
| `set` | `fn(idx: i64, val: T)` | Asignar por índice (para compatibilidad) |
| `contains` | `fn(val: T) bool` | `true` si el valor existe |
| `remove` | `fn(val: T) i32` | Eliminar por valor (1=encontrado, 0=no) |
| `remove_at` | `fn(idx: i64) T` | Eliminar por índice |
| `insert` | `fn(idx: i64, val: T)` | Insertar en posición |
| `clear` | `fn()` | Vaciar la lista |
| `reserve` | `fn(capacity: i64)` | Pre-asignar capacidad |
| `reverse` | `fn()` | Invertir orden |

### Ejemplos con ownership

```ky
# Lista mutable (necesita ^& para mutar)
fn agregar_libro(catalogo: ^&{str}, libro: str):
    catalogo.push(libro)

fn buscar_libro(catalogo: &{str}, titulo: str) bool:
    catalogo.contains(titulo)   # borrow inmutable, no consume

fn main():
    mis_libros: ^_{str} = {}
    mis_libros.push("El Quijote")
    
    agregar_libro(^&mis_libros, "Rayuela")
    
    if buscar_libro(&mis_libros, "Rayuela"):
        println("lo tengo")
    
    # Eliminar por valor
    mis_libros.remove("El Quijote")
```

### Stack (LIFO) via list

```ky
pila: {i32} = {}
pila.push(10)
pila.push(20)
valor = pila.pop()  # → 20
```

### Queue (FIFO) via list

```ky
cola: {i32} = {}
cola.push(10)
cola.push(20)
valor = cola.pop_first()  # → 10
```

### Búsqueda y eliminación por valor

```ky
fn eliminar_usuario(usuarios: ^&{str}, nombre: str):
    if usuarios.contains(nombre):
        usuarios.remove(nombre)
        println("eliminado")
    else:
        println("no encontrado")
```

## Array `[T; N]`

Array nativo en stack. **Acceso por índice únicamente** — para eso están los arrays, son más rápidos y contiguos.

```ky
arr: [5]i32 = [1, 2, 3, 4, 5]
x = arr[2]      # get por índice (O(1))
arr[2] = 99     # set por índice (O(1))
```

## Set `set<T>`

```ky
s: set<i32> = set{1, 2, 3}
s.add(4)
tiene = s.contains(1)
s.remove(1)
n = s.len()
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
for val in &s:
    println(val.to_str())
```

## Iterator

```ky
it = list.iter()
doblados = it.map(fn(x: i32): x * 2).collect()
filtrados = it.filter(fn(x: i32): x > 5).collect()
suma = it.sum()
minimo = it.min()
maximo = it.max()
```

### Métodos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `map` | `fn(fn: fn(T) U) iter<U>` | Transformar cada elemento |
| `filter` | `fn(fn: fn(T) bool) iter<T>` | Filtrar elementos |
| `fold` | `fn(init: U, fn: fn(U, T) U) U` | Reducir a un valor |
| `collect` | `fn() {T}` | Recolectar en lista |
| `next` | `fn() T?` | Siguiente elemento |
| `sum` | `fn() i64` | Sumar todos |
| `min` | `fn() T` | Mínimo |
| `max` | `fn() T` | Máximo |
