# regex — Expresiones Regulares

> Módulo de expresiones regulares.
> Import: `from regex import regex`

## regex: búsqueda y reemplazo

```ky
from regex import regex

re = regex("[0-9]+")
println(re.is_match("abc123"))    # true
println(re.find("abc123"))        # "123"
println(re.replace("abc123", "X"))  # "abcX"

# Con grupos
re2 = regex("(\\w+)@(\\w+)")
match = re2.find("user@host")
println(match)     # "user@host"
```

### Métodos

| Método | Descripción |
|--------|-------------|
| `regex(pattern)` | Compilar expresión regular |
| `re.is_match(s)` | `true` si el string matchea |
| `re.find(s)` | Primera ocurrencia |
| `re.find_all(s)` | Todas las ocurrencias (lista) |
| `re.replace(s, replacement)` | Reemplazar ocurrencias |
| `re.split(s)` | Dividir string por el patrón |

### Ejemplo

```ky
from regex import regex

# Validar email
email_re = regex("^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$")
if email_re.is_match("user@example.com"):
    println("email válido")

# Extraer números
num_re = regex("[0-9]+")
text = "precio: 42 unidades: 7"
nums = num_re.find_all(text)
println(nums)    # {"42", "7"}
```
