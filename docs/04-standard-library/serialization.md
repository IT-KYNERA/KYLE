# serialization — Serialización

> Módulo de serialización de tipos Kyle.
> Import: `from serialization import serialize`

## serialize: convertir tipos a string

```ky
from serialization import serialize

# Serializar a string
str = serialize(42)                    # → "42"
str = serialize(3.14)                  # → "3.14"
str = serialize(true)                 # → "true"

# Serializar structs (vía JSON)
final class User:
    name: str
    age: i32

user = User { name: "Kyle", age: 30 }
str = serialize(user)                  # → '{"name":"Kyle","age":30}'

# Deserializar
user2 = deserialize<User>(str)
println(user2.name)
```

### Funciones

| Función | Descripción |
|---------|-------------|
| `serialize(val)` | Serializar cualquier valor a string |
| `deserialize<T>(str)` | Deserializar string a tipo T |

### Tipos soportados

| Tipo | Serialización |
|------|---------------|
| `i32`, `i64`, `f64` | Número como string |
| `bool` | `"true"` / `"false"` |
| `str` | El string mismo |
| `final class` | JSON |
| `{T}` (list) | JSON array |
| `{K: V}` (dict) | JSON object |

### Ejemplo

```ky
from serialization import serialize, deserialize

final class Config:
    host: str
    port: i32

config = Config { host: "localhost", port: 8080 }
json = serialize(config)
println(json)

restored = deserialize<Config>(json)
println(restored.host)    # "localhost"
```
