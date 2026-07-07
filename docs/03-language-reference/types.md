# Types

## Primitive types [x]

| Type | Size | Description |
|------|------|-------------|
| `i8` | 1 byte | Signed 8-bit integer |
| `i16` | 2 bytes | Signed 16-bit integer |
| `i32` | 4 bytes | Signed 32-bit integer (default) |
| `i64` | 8 bytes | Signed 64-bit integer |
| `f32` | 4 bytes | 32-bit floating point |
| `f64` | 8 bytes | 64-bit floating point |
| `bool` | 1 byte | `true` or `false` |
| `char` | 4 bytes | Unicode code point |
| `str` | pointer | Heap-allocated immutable string |
| `ptr` | 8 bytes | Raw pointer (FFI, unsafe) |

**Bug:** `char = 'a'` type mismatch ("expected 'char', found 'i32'"). Usar `i32` como workaround.

## Compound types

### Optional: `T?` [ ]

Sugar for `Option<T>`. A value that is either `T` o `None`.

```ky
name: str? = None
if value = get_name():
    println(value)
```
**Bug:** `str?` causa type mismatch "'str' expects 1 argument(s), got 2". No funciona.

### Fallible: `T!` [ ]

Sugar for `Result<T, Error>`. A value that is either `T` or an error.

```ky
fn divide(a: i32, b: i32) i32!:
    if b == 0:
        return error("division by zero")
    a / b
```

### Mutable: `&T` [x]

Marks a variable, parameter, or field as mutable.

```ky
count: &i32 = 0
fn increment(x: &i32):
    x = x + 1
```

### Move: `^T` [x]

Parameter with ownership transfer.

```ky
fn consume(^data: str):
    println(data)
```

### Array: `[T; N]` [x]

Array nativo, stack inline, tamaño fijo conocido en compile-time. Acceso vía GEP + load/store — **cero runtime calls**.

```ky
numbers = [1, 2, 3]           # → [i32; 3]
numbers = [person1, person2]  # → [Person; 2]

matrix: [[i32; 3]; 3] = [     # anidado
    [1, 0, 0],
    [0, 1, 0],
    [0, 0, 1]
]

first = numbers[0]             # GEP + load (nativo)
numbers[0] = 42                # GEP + store (nativo)
```

### Array repeat: `[val; N]` [x]

Creación de arrays con valor repetido. `N` debe ser un literal entero (constante en compile-time para el tamaño del array).

```ky
zeros = [0; 100]               # → [i32; 100], todos 0
ones = [1 as i64; 10000]       # → [i64; 10000], todos 1
floats = [0.0; 50]             # → [f64; 50], todos 0.0
```

> **Nota:** Los arrays son **copy-by-value**. Para arrays grandes (>1000 elementos), cada acceso por `arr[i]` copia el array completo a un temporal. Para esos casos, usa `{T}` (listas) que tienen semántica de puntero.

### List: `{T}` [x]

Lista dinámica, heap, redimensionable. Usa `ky_list_*` runtime.

```ky
numbers = {1, 2, 3}            # → {i32}
numbers.push(4)
first = numbers[0]             # ky_list_get
```

### Tuple: `(T1, T2, ...)` [x]

Fixed-size heterogeneous collection.

```ky
point = (10, 20)
x = point.0
y = point.1
```

## User-defined types

### final class

```ky
final class Vec2:
    x: i32
    y: i32
```

### class

```ky
class Animal:
    name: str
    fn speak():
        println("...")

class Dog :: Animal:
    fn speak():
        println("woof")
```

### enum

```ky
enum Optional:
    Some(i32)
    None
```

### contract

```ky
contract Comparable:
    fn compare(other: This) i32
```

## Type inference

```ky
x = 42          # i32
y: i64 = 42     # explicit i64
z = [1, 2, 3]   # [i32; 3]
z2 = {1, 2, 3}  # {i32}
```

## Type casting

```ky
x = 42 as i64
y = 3.14 as i32
z = 42 as f64
```

## Type introspection

Kyle permite obtener información sobre tipos en tiempo de compilación mediante `type<T>()` y el método `.type()`.

### `type<T>()` — función genérica

```ky
t = type<i32>()
t.name     # "i32"
t.kind     # "primitive"
t.size     # 4 (bytes)

t = type<str>()
t.name     # "str"
t.kind     # "primitive"
t.size     # 8

t = type<User>()
t.name     # "User"
t.kind     # "struct"
t.size     # 12 (suma de campos)

t = type<{i32}>()
t.name     # "{i32}"
t.kind     # "list"
t.size     # 8 (handle)

t = type<[i32; 3]>()
t.name     # "[i32; 3]"
t.kind     # "array"
t.size     # 12 (3 * 4 bytes)
```

### `.type()` — método en cualquier valor

```ky
x = 42
t = x.type()
t.name     # "i32"

s = "hello"
t = s.type()
t.name     # "str"

user = User { name: "Ana", age: 30 }
t = user.type()
t.name     # "User"
t.kind     # "struct"
```

### Campos de TypeInfo

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `name` | `str` | Nombre del tipo (`"i32"`, `"User"`, `"{i32}"`, `"[i32; 3]"`, etc.) |
| `kind` | `str` | Categoría: `"primitive"`, `"struct"`, `"list"`, `"dict"`, `"ptr"`, `"array"` |
| `size` | `i32` | Tamaño en bytes |

### Tipos y sus nombres

| Tipo | `name` | `kind` | `size` |
|------|--------|--------|--------|
| `i8` | `"i8"` | `"primitive"` | 1 |
| `i16` | `"i16"` | `"primitive"` | 2 |
| `i32` | `"i32"` | `"primitive"` | 4 |
| `i64` | `"i64"` | `"primitive"` | 8 |
| `f32` | `"f32"` | `"primitive"` | 4 |
| `f64` | `"f64"` | `"primitive"` | 8 |
| `bool` | `"bool"` | `"primitive"` | 1 |
| `char` | `"char"` | `"primitive"` | 4 |
| `str` | `"str"` | `"primitive"` | 8 |
| `ptr` | `"ptr"` | `"ptr"` | 8 |
| `{T}` | `"{T}"` | `"list"` | 8 |
| `[T; N]` | `"[T; N]"` | `"array"` | N × size(T) |
| `{K: V}` | `"{K: V}"` | `"dict"` | 8 |
| `User` (class) | `"User"` | `"struct"` | suma campos |
| `fn(...) T` | `"fn"` | `"function"` | 8 |

> **Nota:** `type<T>()` es resuelto en compilación, no tiene overhead en runtime.
> Para tipos genéricos anidados usa espacios: `type<{i32}>()` no `type<{i32}>()`.

---

## Built-in methods

Kyle proporciona métodos integrados en tipos primitivos y compuestos.

### String methods

| Método | Descripción | Ejemplo |
|--------|-------------|---------|
| `s.len()` | Longitud de la cadena | `"hola".len()` → `4` |
| `s.upper()` | Mayúsculas | `"hola".upper()` → `"HOLA"` |
| `s.lower()` | Minúsculas | `"HOLA".lower()` → `"hola"` |
| `s.trim()` | Eliminar espacios | `"  hola  ".trim()` → `"hola"` |
| `s.contains(sub)` | Contiene subcadena | `"hola".contains("ol")` → `1` |
| `s.replace(from, to)` | Reemplazar | `"a-b".replace("-", "/")` → `"a/b"` |
| `s.substr(start, len)` | Subcadena | `"hello".substr(1, 3)` → `"ell"` |
| `s.char_at(i)` | Carácter en posición | `"abc".char_at(1)` → `98` (ASCII) |
| `s.ord()` | ASCII del primer char | `"A".ord()` → `65` |
| `s.is_digit()` | Es dígito? | `"5".is_digit()` → `1` |
| `s.is_alpha()` | Es letra? | `"a".is_alpha()` → `1` |
| `s.is_alnum()` | Es alfanumérico? | `"a1".is_alnum()` → `1` |
| `s.is_whitespace()` | Es espacio? | `" ".is_whitespace()` → `1` |
| `s.is_upper()` | Es mayúscula? | `"A".is_upper()` → `1` |
| `s.is_lower()` | Es minúscula? | `"a".is_lower()` → `1` |

### Number conversion methods

Métodos específicos por tipo. Disponibles en cualquier valor numérico.

| Método | Descripción | Ejemplo |
|--------|-------------|---------|
| `val.to_i32()` | Convertir a i32 | `42.to_i32()` → `42` |
| `val.to_i64()` | Convertir a i64 | `42.to_i64()` → `42` |
| `val.to_i16()` | Convertir a i16 | `42.to_i16()` → `42` |
| `val.to_i8()` | Convertir a i8 | `42.to_i8()` → `42` |
| `val.to_u32()` | Convertir a u32 | `42.to_u32()` → `42` |
| `val.to_u64()` | Convertir a u64 | `42.to_u64()` → `42` |
| `val.to_u16()` | Convertir a u16 | `42.to_u16()` → `42` |
| `val.to_u8()` | Convertir a u8 | `42.to_u8()` → `42` |
| `val.to_f64()` | Convertir a f64 | `42.to_f64()` → `42.0` |
| `val.to_f32()` | Convertir a f32 | `42.to_f32()` → `42.0` |
| `val.to_char()` | Convertir a char (por código) | `65.to_char()` → `'A'` |
| `val.to_bool()` | Convertir a booleano | `1.to_bool()` → `true` |
| `val.to_str()` | Convertir a string | `42.to_str()` → `"42"` |
| `val.to_decimal()` | Convertir a decimal string | `12345.to_decimal()` → `"123.45"` |

**Nota:** `str()`, `int()`, `float()`, `bool()` como funciones sueltas fueron removidas. Usar los métodos `val.to_*()`.

### Collection methods

En listas `{T}`:

| Método | Descripción | Ejemplo |
|--------|-------------|---------|
| `lst.len()` | Cantidad de elementos | `{1,2,3}.len()` → `3` |
| `lst.push(val)` | Agregar al final | `{1}.push(2)` → `{1,2}` |
| `lst.pop()` | Quitar y retornar último | `{1,2,3}.pop()` → `3` |
| `lst.add(val)` | Sinónimo de push | |
| `lst.contains(val)` | Contiene valor? | `{1,2}.contains(2)` → `1` |
| `lst.sum()` | Suma de elementos | |
| `lst.product()` | Producto de elementos | |
| `lst.max()` | Valor máximo | |
| `lst.min()` | Valor mínimo | |
| `lst.reverse()` | Invertir lista | |
| `lst.map(fn)` | Aplicar función a c/e | |
| `lst.filter(fn)` | Filtrar elementos | |

En arrays `[T; N]`:

| Método | Descripción |
|--------|-------------|
| `arr.len()` | Cantidad de elementos (compile-time) |
| `arr[i]` | Leer elemento (GEP + load) |
| `arr[i] = val` | Escribir elemento (GEP + store) |

En diccionarios `{K: V}`:

| Método | Descripción |
|--------|-------------|
| `dict.len()` | Cantidad de entradas |
| `dict[key]` | Obtener valor por clave |
| `dict[key] = val` | Asignar valor por clave |
| `dict.key` | Obtener valor por clave identifier (azúcar) |

### Char methods

En caracteres (`char`):

| Método | Descripción | Ejemplo |
|--------|-------------|---------|
| `c.ord()` | Código ASCII | `'A'.ord()` → `65` |
| `c.is_digit()` | Es dígito? | `'5'.is_digit()` → `1` |
| `c.is_alpha()` | Es letra? | `'a'.is_alpha()` → `1` |
| `c.is_alnum()` | Es alfanumérico? | `'a'.is_alnum()` → `1` |
| `c.is_whitespace()` | Es espacio? | `' '.is_whitespace()` → `1` |
| `c.is_upper()` | Es mayúscula? | `'A'.is_upper()` → `1` |
| `c.is_lower()` | Es minúscula? | `'a'.is_lower()` → `1` |

### Universal method

| Método | Descripción |
|--------|-------------|
| `val.type()` | Retorna `TypeInfo` con `.name`, `.kind`, `.size` |
| `val.stringify()` | Convertir a JSON string (dicts) |

---

## Platform types — implementados

### datetime ✅

Package: `from datetime import datetime`

```ky
from datetime import datetime

d = datetime.now()
d = datetime.parse("2026-07-04T12:00:00Z")
d = datetime.from_ymdhms(2026, 7, 4, 12, 0, 0)

d.year()       # 2026
d.month()      # 7
d.day()        # 4
d.hour()       # 12
d.minute()     # 30
d.second()     # 0

d.to_str()     # "2026-07-04 12:30:00"
d.format("%Y-%m-%d")  # "2026-07-04"

d2 = d.add_days(7)
d2 = d.add_hours(3)
```

| Método | Retorno | Descripción |
|--------|---------|-------------|
| `datetime.now()` | datetime | Momento actual UTC |
| `datetime.parse(s)` | datetime | Parsear ISO 8601 |
| `datetime.from_ymdhms(y,m,d,h,min,s)` | datetime | Desde componentes |
| `d.year()` | i32 | Año |
| `d.month()` | i32 | Mes (1-12) |
| `d.day()` | i32 | Día (1-31) |
| `d.hour()` | i32 | Hora (0-23) |
| `d.minute()` | i32 | Minuto (0-59) |
| `d.second()` | i32 | Segundo (0-59) |
| `d.to_str()` | str | Formato ISO |
| `d.format(fmt)` | str | Formatear con strftime |
| `d.add_days(n)` | datetime | Sumar días |
| `d.add_hours(n)` | datetime | Sumar horas |

### duration ✅

Package: `from datetime import duration`

```ky
from datetime import duration

dur = duration.from_millis(86400000)
dur.seconds()     # 86400
dur.minutes()     # 1440
dur.hours()       # 24
dur.days()        # 1
```

| Método | Retorno | Descripción |
|--------|---------|-------------|
| `duration.from_millis(ms)` | duration | Desde milisegundos |
| `dur.seconds()` | i64 | Total segundos |
| `dur.minutes()` | i64 | Total minutos |
| `dur.hours()` | i64 | Total horas |
| `dur.days()` | i64 | Total días |
| `dur.to_str()` | str | Representación |

---

## Platform types (futuro)

### bytes

Arreglo de bytes para datos binarios.

```ky
data = bytes.new(4)          # [0, 0, 0, 0]
data[0] = 0xFF
data.len()                   # 4
hex = data.hex()             # "ff000000"
b64 = data.base64()          # "/wAAAA=="
```

| Método | Retorno | Descripción |
|--------|---------|-------------|
| `bytes.new(size)` | bytes | Crear con tamaño |
| `bytes.from_hex(s)` | bytes | Desde hex string |
| `bytes.from_base64(s)` | bytes | Desde base64 |
| `b.len()` | i32 | Longitud |
| `b[i]` | i32 | Leer byte |
| `b[i] = val` | — | Escribir byte |
| `b.hex()` | str | A hex string |
| `b.base64()` | str | A base64 string |
| `b.slice(start, end)` | bytes | Sub-arreglo |

### decimal

Número de precisión fija.

```ky
d = decimal.new("99.99")
d2 = decimal.new("0.01")
r = d + d2                   # 100.00
r.round(2)                   # 100.00
```

| Método | Retorno | Descripción |
|--------|---------|-------------|
| `decimal.new(val)` | decimal | Desde string |
| `d.round(n)` | decimal | Redondear |
| `d.truncate()` | i32 | Truncar a entero |
| `a + b` | decimal | Suma |
| `a - b` | decimal | Resta |

### uuid

```ky
id = uuid.v4()
id2 = uuid.parse("550e8400-e29b-41d4-a716-446655440000")
id.to_str()
```

| Método | Retorno | Descripción |
|--------|---------|-------------|
| `uuid.v4()` | uuid | Generar UUID v4 |
| `uuid.parse(s)` | uuid | Desde string |
| `u.to_str()` | str | A string |

### url

```ky
u = url.parse("https://api.example.com/users?page=1")
u.scheme()      # "https"
u.host()        # "api.example.com"
u.path()        # "/users"
```

| Método | Retorno | Descripción |
|--------|---------|-------------|
| `url.parse(s)` | url | Desde string |
| `u.scheme()` | str | Protocolo |
| `u.host()` | str | Host |
| `u.port()` | i32 | Puerto |
| `u.path()` | str | Ruta |
| `u.query()` | str | Query string |

### regex

```ky
re = regex.new(r"\d+")
re.is_match("abc123")        # true
re.replace("abc123", "X")    # "abcX"
```

| Método | Retorno | Descripción |
|--------|---------|-------------|
| `regex.new(pattern)` | regex | Compilar patrón |
| `re.is_match(s)` | bool | Coincide? |
| `re.find(s)` | str | Primera coincidencia |
| `re.find_all(s)` | list<str> | Todas las coincidencias |
| `re.replace(s, with)` | str | Reemplazar |
| `re.split(s)` | list<str> | Dividir |

### Plan de implementación

| Fase | Tipos | Estado | Dependencias |
|------|-------|--------|-------------|
| **1** | `datetime` + `duration` | ✅ Implementado | `chrono` crate |
| **2** | `bytes` | 🔜 Pendiente | Runtime Rust |
| **3** | `decimal` | 🔜 Pendiente | Runtime Rust |
| **4** | `uuid` | 🔜 Pendiente | `uuid` crate |
| **5** | `url` | 🔜 Pendiente | Runtime Rust |
| **6** | `regex` | 🔜 Pendiente | `regex` crate |
