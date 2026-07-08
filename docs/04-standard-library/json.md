# json — JSON

> Módulo de parseo y serialización JSON.
> Import: `from json import json`

## json: parseo y stringify

```ky
from json import json

# Parsear string JSON
data = json.parse('{"name": "Kyle", "age": 30}')
name = data["name"]          # → "Kyle"
age = data["age"]            # → 30 (i64)

# Serializar a string
str = json.stringify(data)   # → '{"name":"Kyle","age":30}'
str = json.pretty(data)      # pretty-print con indentación

# Serializar struct
final class User:
    name: str
    age: i32

user = User { name: "Ana", age: 25 }
str = json.serialize(user)   # → '{"name":"Ana","age":25}'
parsed = json.deserialize<User>(str)
```

### Funciones

| Función | Descripción |
|---------|-------------|
| `json.parse(str)` | Parsear string → dict |
| `json.stringify(val)` | Serializar a string |
| `json.pretty(val)` | Pretty-print con indentación |
| `json.serialize(val)` | Struct/Dict → JSON string |
| `json.deserialize<T>(str)` | JSON string → T |

### Tipos JSON

| JSON | Kyle |
|------|------|
| `"string"` | `str` |
| `123` | `i64` |
| `true/false` | `bool` |
| `[1, 2]` | `{i64}` |
| `{"k": "v"}` | `{str: str}` |

### Ejemplo

```ky
from json import json

text = '{"users": [{"name": "Kyle", "age": 30}]}'
data = json.parse(text)
users = data["users"]
first = users[0]
println(first["name"])
```
