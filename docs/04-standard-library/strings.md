# strings — Utilidades de String

> Módulo de manipulación de strings y el tipo `str_builder`.
> Import: `from strings import str, str_builder`

## str: métodos del tipo string

Los strings en Kyle son inmutables y heap-allocados. Tienen métodos integrados.

```ky
from strings import str

s = "  Hello World  "
s2 = s.trim()
s3 = s.to_upper()
s4 = s3.replace("HELLO", "HI")
```

### Métodos de str

| Método | Descripción | Ejemplo |
|--------|-------------|---------|
| `len()` | Largo del string | `s.len()` |
| `contains(sub)` | `true` si contiene substring | `s.contains("lo")` |
| `starts_with(prefix)` | `true` si empieza con | `s.starts_with("He")` |
| `ends_with(suffix)` | `true` si termina con | `s.ends_with("ld")` |
| `to_upper()` | Mayúsculas | `s.to_upper()` |
| `to_lower()` | Minúsculas | `s.to_lower()` |
| `trim()` | Sin espacios extremos | `s.trim()` |
| `replace(from, to)` | Reemplazar substring | `s.replace("a", "b")` |
| `char_at(idx)` | Carácter en posición | `s.char_at(0)` |
| `substr(start, count)` | Substring | `s.substr(0, 5)` |

### Funciones standalone

| Función | Descripción |
|---------|-------------|
| `len(s)` | Largo del string |
| `str.is_digit(c)` | `true` si es dígito |
| `str.is_alpha(c)` | `true` si es letra |
| `str.is_alnum(c)` | `true` si es alfanumérico |
| `str.is_whitespace(c)` | `true` si es espacio |
| `str.is_upper(c)` | `true` si es mayúscula |
| `str.is_lower(c)` | `true` si es minúscula |

```ky
from strings import str

if str.is_digit('5'):
    println("es dígito")
```

## str_builder: construcción eficiente de strings

`str_builder` es un buffer mutable para concatenación eficiente, similar a
`strings.Builder` de Go o `StringBuilder` de Java.

```ky
from strings import str_builder

sb = str_builder(50000)
i: ^i32 = 0
while i < 50000:
    sb.append("x")
    i = i + 1
result = sb.to_str()
println(result)
```

### Métodos

| Método | Descripción |
|--------|-------------|
| `str_builder(capacity)` | Constructor con capacidad inicial |
| `append(s)` | Agregar string al buffer |
| `to_str()` | Extraer string final |
| `free()` | Liberar memoria del builder |

### Performance

`append()` redimensiona con estrategia de duplicación (2× capacidad) cuando el buffer
se llena, logrando O(1) amortizado por operación. Comparado con `s = s + "x"` (que
asigna + copia en cada concat), `str_builder` es ~380× más rápido para 50k
concatenaciones.
