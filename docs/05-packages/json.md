# json — JSON Parsing and Generation for Kyle

**Versión:** 2.0  
**Estado:** ✅ Completo (Fase 3)

---

## 1. Funciones básicas

```kyle
from json import parse, stringify

data = parse("{\"name\":\"Ana\",\"age\":30}")
print(data["name"])     # "Ana"

text = stringify(data)
print(text)             # {"name":"Ana","age":30}
```

## 2. Clase → JSON (`serialize`)

Built-in global. Sin descriptor manual:

```kyle
final class User:
    name: str
    age: i32

user = User { name: "Kyle", age: 1 }
json = serialize(user)       # → {"name":"Kyle","age":1}
```

También soporta comas en struct literals:

```kyle
user = User { name: "Kyle", age: 1 }    # con comas
user = User { name: "Kyle" age: 1 }     # sin comas (legacy)
```

## 3. JSON → Clase (`deserialize<T>`)

```kyle
final class User:
    name: str
    age: i32

user = deserialize<User>("{\"name\":\"Ana\",\"age\":30}")
print(user.name)   # "Ana"
print(user.age)    # 30
```

## 4. Integración con HTTP

```kyle
from http import Client

final class Post:
    userId: i32
    id: i32
    title: str
    body: str

client = Client { timeout: 10 }

# Enviar: clase → auto-JSON
data = Post { userId: 1, id: 0, title: "test", body: "hello" }
res = client.post("https://api.example.com/posts", data)

# Recibir: JSON → clase automático
res = client.get("https://jsonplaceholder.typicode.com/posts/1")
post = deserialize<Post>(res.body)
print(post.title)
```

## 5. API completa

| Función | Descripción |
|---------|-------------|
| `parse(str) → dict` | JSON string → dict |
| `stringify(dict) → str` | dict → JSON string |
| `serialize(val) → str` | Cualquier `final class` → JSON string |
| `deserialize<T>(str) → T` | JSON string → clase `T` |
