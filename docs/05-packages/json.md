# json — JSON Parsing and Generation for Kyle

**Versión:** 1.0  
**Estado:** ✅ Completo (Fase 2)

---

## 1. Funciones básicas

```kyle
from json import parse, stringify

data = parse("{\"name\":\"Ana\",\"age\":30}")
print(data["name"])     # "Ana"

text = stringify(data)
print(text)             # {"name":"Ana","age":30}
```

## 2. Clases → JSON (`struct_to_json`)

Cualquier `final class` se serializa a JSON usando un descriptor de campos:

```kyle
from json import struct_to_json

final class User:
    name: str
    age: i32
    active: bool

user = User { name: "Kyle" age: 25 active: true }
json = struct_to_json(user, "name:str,age:i32,active:bool")
# → {"name":"Kyle","age":25,"active":true}
```

**Descriptor format:** `"field1:type1,field2:type2,..."`

| Type | Bytes | JSON output |
|------|-------|-------------|
| `str` | 8 (ptr) | `"string"` |
| `i32` | 4 | `42` |
| `i64` | 8 | `42` |
| `bool` | 1 | `true` / `false` |
| `f64` | 8 | `3.14` |

## 3. JSON → Clases (`json_to_struct`)

```kyle
from json import json_to_struct

final class User:
    name: str
    age: i32
    active: bool

user = User { name: "" age: 0 active: false }
json_to_struct("\{\"name\":\"Ana\",\"age\":30,\"active\":true\}", 
               "name:str,age:i32,active:bool", user)
print(user.name)   # "Ana"
print(user.age)    # 30
```

**Nota:** Escapar `\{` y `\}` en el string JSON para evitar interpolación.

## 4. Integración con HTTP

```kyle
from http import Client
from json import struct_to_json

client = Client(30)

final class Payload:
    title: str
    body: str
    userId: i32

data = Payload { title: "Kyle" body: "hello" userId: 1 }
json_str = struct_to_json(data, "title:str,body:str,userId:i32")
res = client.post("https://api.example.com/posts", json_str)
```

## 5. Plan de implementación

| Feature | Estado |
|---------|--------|
| `parse(str) → dict` | ✅ |
| `stringify(dict) → str` | ✅ |
| `struct_to_json[T](val, descriptor) → str` | ✅ |
| `json_to_struct[T](json, descriptor, out)` | ✅ |
| Auto-generación de descriptor por el compilador | 🔜 Fase 3 |
| Union types (`JsonValue`) | 🔜 Futuro |
