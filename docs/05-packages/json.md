# json — JSON Parsing and Generation

**Versión:** 2.0  
**Estado:** Especificación

---

## 1. Funciones principales

Built-in globales. No necesita import.

```kyle
# Clase → JSON string
json = serialize(user)

# JSON string → Clase (con <T> genérico)
user = deserialize<User>(json_str)

# Dict → JSON (legacy)
text = stringify(data)

# JSON → Dict
data = parse(json_str)
```

---

## 2. Clase → JSON (`serialize`)

```kyle
final class User:
    name: str
    age: i32
    active: bool

user = User { name: "Kyle", age: 1, active: true }
json = serialize(user)
# → {"name":"Kyle","age":1,"active":true}
```

Auto-detecta los campos. Sin descriptor manual.

---

## 3. JSON → Clase (`deserialize<T>`)

```kyle
json_str = "{\"name\":\"Ana\",\"age\":30,\"active\":true}"
user = deserialize<User>(json_str)
print(user.name)   # "Ana"
print(user.age)    # 30
```

---

## 4. En HTTP

```kyle
from http.client import Client

final class Todo:
    title: str
    body: str
    userId: i32

client = Client { timeout: 10 }

# POST con clase → auto-JSON
data = Todo { title: "Kyle", body: "test", userId: 1 }
res = client.post(url, data)

# GET + deserializar
res = client.get("https://api.example.com/todos/1")
todo = deserialize<Todo>(res.body)
print(todo.title)
```

---

## 5. En el servidor

```kyle
from http.server import Router

app = Router()

app.post("/users", (req, res):
    user = req.body<User>()
    res.json({created: true, id: 1}, 201)
)

app.get("/users/{id:i32}", (req, res):
    user = find_user(req.param("id"))
    res.json(user)
)
```

---

## 6. Referencia rápida

| Función | Descripción |
|---------|-------------|
| `serialize(val)` | Cualquier `final class` → JSON string |
| `deserialize<T>(str)` | JSON string → clase `T` |
| `stringify(dict)` | `{K: V}` → JSON string (legacy) |
| `parse(str)` | JSON string → `{K: V}` (legacy) |
