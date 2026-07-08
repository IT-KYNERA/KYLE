# json — JSON

> Módulo de parseo y serialización JSON.
> Import: `from json import json`

## json: parseo y stringify

```ky
from json import json

data: {str: i64} = json.parse('{"name": "Kyle", "age": 30}')
name: str = data["name"]
age: i64 = data["age"]

str: str = json.stringify(data)
pretty: str = json.pretty(data)
```

### Funciones

| Función | Firma | Descripción |
|---------|-------|-------------|
| `json.parse(s)` | `fn(s: str) {K: V}` | Parsear string → dict |
| `json.stringify(val)` | `fn(val: T) str` | Serializar a string |
| `json.pretty(val)` | `fn(val: T) str` | Pretty-print con indentación |
| `json.serialize(val)` | `fn(val: T) str` | Struct/Dict → JSON string |
| `json.deserialize<T>(s)` | `fn(s: str) T` | JSON string → T |

### Serialización de structs

```ky
from json import json

final class User:
    name: str
    age: i32

user: User = User { name: "Ana", age: 25 }
json_str: str = json.serialize(user)
parsed: User = json.deserialize<User>(json_str)
```

### Tipos JSON ↔ Kyle

| JSON | Kyle |
|------|------|
| `"string"` | `str` |
| `123` | `i64` |
| `true / false` | `bool` |
| `[1, 2]` | `{i64}` |
| `{"k": "v"}` | `{str: str}` |
