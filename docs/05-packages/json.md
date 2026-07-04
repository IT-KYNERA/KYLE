# json — JSON Parsing and Generation for Kyle

**Versión:** 1.0  
**Estado:** Especificación

---

## 1. JsonValue — tipo unión para JSON

Cualquier valor JSON se representa con `JsonValue`:

```kyle
type JsonValue = i64 | f64 | str | bool | list<JsonValue> | dict<str, JsonValue>
```

## 2. Funciones principales

```kyle
from json import JsonValue, parse, stringify

# String → JsonValue
data = parse("{\"name\": \"Kyle\", \"version\": 1}")

# JsonValue → String
text = stringify(data)
```

## 3. Clases → JSON automático

Cualquier `final class` se serializa a JSON sin configuración:

```kyle
final class User:
    name: str
    age: i32
    active: bool

user = User { name: "Kyle", age: 1, active: true }
text = stringify(user)   # {"name":"Kyle","age":1,"active":true}
```

Anidación:

```kyle
final class Address:
    city: str
    country: str

final class Person:
    name: str
    address: Address

p = Person {
    name: "Ana",
    address: Address { city: "CDMX", country: "MX" }
}
text = stringify(p)   # {"name":"Ana","address":{"city":"CDMX","country":"MX"}}
```

## 4. JSON → Clases

```kyle
json_str = "{\"name\":\"Kyle\",\"age\":1}"
user = parse(json_str) as User   # casteo a User
```

Con genéricos (cuando existan):

```kyle
user = parse[User](json_str)     # User inferido
```

## 5. En HTTP

El cliente y servidor usan `JsonValue` automáticamente:

```kyle
# Cliente — cualquier clase se serializa al enviar
res = client.post(url, user)        # User → JSON

# Servidor — JSON se deserializa al tipo pedido
fn create_user(req, res, next):
    user = req.body<User>()          # JSON → User
    res.json(user, 201)
```

---

## 6. Plan de implementación

| Feature | Status |
|---------|--------|
| `JsonValue` type (`A \| B`) | 🔜 Compiler |
| `stringify(value: final class)` | 🔜 |
| `parse[T](s: str)` | 🔜 |
