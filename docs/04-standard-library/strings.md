# strings — Utilidades de String

> Módulo de manipulación de strings y el tipo `str_builder`.
> Import: `from strings import str, str_builder`

## str: métodos del tipo string

Los strings en Kyle son inmutables, heap-allocados y **Move semantics**.

```ky
from strings import str

s: str = "  Hello World  "
s2: str = s.trim()
s3: str = s.to_upper()
s4: str = s3.replace("HELLO", "HI")
```

### Métodos de str

| Método | Firma | Descripción | Ejemplo |
|--------|-------|-------------|---------|
| `len` | `fn() i32` | Largo del string | `s.len()` |
| `contains` | `fn(sub: str) bool` | `true` si contiene substring | `s.contains("lo")` |
| `starts_with` | `fn(prefix: str) bool` | `true` si empieza con | `s.starts_with("He")` |
| `ends_with` | `fn(suffix: str) bool` | `true` si termina con | `s.ends_with("ld")` |
| `to_upper` | `fn() str` | Mayúsculas | `s.to_upper()` |
| `to_lower` | `fn() str` | Minúsculas | `s.to_lower()` |
| `trim` | `fn() str` | Sin espacios extremos | `s.trim()` |
| `replace` | `fn(from: str, to: str) str` | Reemplazar substring | `s.replace("a", "b")` |
| `char_at` | `fn(idx: i32) i8` | Carácter en posición | `s.char_at(0)` |
| `substr` | `fn(start: i32, count: i32) str` | Substring | `s.substr(0, 5)` |

### Funciones standalone

```ky
n: i32 = len(s)                # largo del string
es_digito: bool = str.is_digit('5')
es_alpha: bool = str.is_alpha('a')
es_alnum: bool = str.is_alnum('x')
es_space: bool = str.is_whitespace(' ')
es_mayus: bool = str.is_upper('A')
es_minus: bool = str.is_lower('a')
```

| Función | Firma | Descripción |
|---------|-------|-------------|
| `len(s)` | `fn(s: str) i32` | Largo del string |
| `str.is_digit(c)` | `fn(c: i8) bool` | `true` si es dígito |
| `str.is_alpha(c)` | `fn(c: i8) bool` | `true` si es letra |
| `str.is_alnum(c)` | `fn(c: i8) bool` | `true` si es alfanumérico |
| `str.is_whitespace(c)` | `fn(c: i8) bool` | `true` si es espacio |
| `str.is_upper(c)` | `fn(c: i8) bool` | `true` si es mayúscula |
| `str.is_lower(c)` | `fn(c: i8) bool` | `true` si es minúscula |

### Chaining

```ky
result: str = "  Hello World  ".trim().to_upper().substr(0, 5)
println(result)    # "HELLO"
```

## str_builder: construcción eficiente de strings

`str_builder` es un buffer mutable para concatenación eficiente.

```ky
from strings import str_builder

sb: str_builder = str_builder(50000)
i: ^i32 = 0
while i < 50000:
    sb.append("x")
    i = i + 1
result: str = sb.to_str()
println(result)
```

### Métodos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `str_builder` | `fn(capacity: i64 = 16) str_builder` | Constructor |
| `append` | `fn(s: &str)` | Agregar string al buffer |
| `to_str` | `fn() str` | Extraer string final |
| `free` | `fn()` | Liberar memoria del builder |

### Performance

`append()` redimensiona con estrategia de duplicación (2× capacidad). Comparado
con `s = s + "x"` (alloc + copy en cada concat), es **~380× más rápido**.
