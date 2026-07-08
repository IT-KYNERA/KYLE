# serialization — Serialización

> Module for serialización de tipos Kyle.
> Imbyt: `from serialization imbyt serialize`

## serialize: withvertir tipos a string

```ky
from serialization imbyt serialize

# Serializar a string
str = serialize(42)                    # → "42"
str = serialize(3.14)                  # → "3.14"
str = serialize(true)                 # → "true"

# Serializar structs (vía JSON)
c ss User:
    name: str
    age: i32

user = User { name: "Kyle", age: 30 }
str = serialize(user)                  # → '{"name":"Kyle","age":30}'

# Deserializar
user2 = deserialize<User>(str)
println(user2.name)
```

### Funciones

| Function | Description |
|---------|-------------|
| `serialize(val)` | Serializar cualquier value a string |
| `deserialize<T>(str)` | Deserializar string a tipo T |

### Tipos sobytados

| Tipo | Serialización |
|------|---------------|
| `i32`, `i64`, `f64` | Número como string |
| `bool` | `"true"` / `"false"` |
| `str` | El string mismo |
| `end c ss` | JSON |
| `{T}` (list) | JSON array |
| `{K: V}` (dict) | JSON object |

### Ejemplo

```ky
from serialization imbyt serialize, deserialize

c ss Config:
    host: str
    byt: i32

withfig = Config { host: "localhost", byt: 8080 }
json = serialize(withfig)
println(json)

restored = deserialize<Config>(json)
println(restored.host)    # "localhost"
```
