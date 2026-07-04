# http — HTTP Client and Server for Kyle

**Versión:** 5.0  
**Estado:** Especificación

---

## 1. Filosofía

El package `http` unifica cliente y servidor HTTP en una sola biblioteca.
Todo se instancia. No hay funciones globales.

Cualquier `final class` se serializa automáticamente a JSON.

```kyle
from http import Client, HttpStatus, HttpMethod
from http.server import Server, Request, Res
```

---

## 2. Organización del package

```
packages/http/
├── ky.toml
└── src/
    ├── lib.ky           # Tipos compartidos: HttpStatus, HttpMethod, Header, MimeType
    ├── client.ky        # Client class
    └── server.ky        # Server class + routing + middleware
```

---

## 3. Tipos compartidos (`http`)

### `HttpMethod`

```kyle
enum HttpMethod:
    GET | POST | PUT | DELETE | PATCH | HEAD | OPTIONS
```

### `HttpStatus`

```kyle
final class HttpStatus:
    code: i32
    text: str
```

Constantes: `HttpStatusOk` (200), `HttpStatusCreated` (201),
`HttpStatusNotFound` (404), `HttpStatusInternalServerError` (500)

### `Header`

```kyle
final class Header:
    name: str
    value: str
```

---

## 4. Cliente HTTP

### Uso básico

```kyle
from http import Client

client = Client(30)

# GET → Response con body string
res = client.get("https://api.github.com/repos/IT-KYNERA/KYLE")
if res.is_ok:
    print(res.body)

# POST con JSON — cualquier clase se serializa automáticamente
final class User:
    name: str
    age: i32

user = User { name: "Kyle", age: 1 }
res = client.post("https://api.example.com/users", user)

# POST con string raw (XML, texto, etc.)
res = client.post("https://api.example.com/data", "<xml>...</xml>")
```

### Response

```kyle
final class Response:
    status_code: i32
    status_text: str
    body: str
    is_ok: bool
    elapsed_ms: i64

    # Deserializar body como JSON con tipo explícito
    fn json[T]() T

    # Sin genérico → JsonValue (dinámico)
    fn json() JsonValue
```

### Métodos del Client

| Método | Descripción |
|--------|-------------|
| `client.get(url)` | GET |
| `client.post(url, body)` | POST — body puede ser `str`, `JsonValue` o `final class` |
| `client.put(url, body)` | PUT |
| `client.patch(url, body)` | PATCH |
| `client.delete(url)` | DELETE |

`post()`/`put()`/`patch()` detectan el tipo del body:
- `str` → se envía raw
- `final class` → se serializa a JSON automáticamente
- `JsonValue` → se serializa a JSON

---

## 5. Servidor HTTP

### Uso básico

```kyle
from http.server import Server

server = Server()

server.get("/health", (req, res, next):
    res.json({ "status": "ok" })
)

server.listen(8080)
```

### Rutas con parámetros

```kyle
server.get("/users/{id}", (req, res, next):
    id = req.param("id")         # str por defecto
    res.json({ "user": id })
)

server.get("/users/{id:i32}", (req, res, next):
    id = req.param("id")         # i32 — parseado automáticamente
    res.json({ "user": id })
)

server.post("/users", (req, res, next):
    user = req.body<User>()      # JSON → User automático
    res.json(user, 201)
)
```

### Request

```kyle
final class Request:
    method: HttpMethod
    path: str
    params: dict<str, str>      # path params: {id} → "42"
    query: dict<str, str>       # query: ?page=1
    body: str                   # raw body

    fn param(name: str) str     # path param por nombre
    fn header(name: str) str    # header por nombre
    fn body[T]() T              # deserializar body como tipo T (JSON → clase)
```

### Res

```kyle
final class Res:
    fn json(data: final class):           # 200 + cualquier clase → JSON
    fn json(data: final class, code: i32): # status + JSON
    fn text(body: str):                    # 200 + texto
    fn text(body: str, code: i32):         # status + texto
    fn redirect(url: str):                 # 302
```

Cualquier `final class` se serializa a JSON automáticamente en `res.json()`.

### Middleware

```kyle
server.before("/api/*", (req, res, next):
    if req.header("Authorization") == "":
        res.text("unauthorized", 401)
    else:
        next()
)

server.after("/api/*", (req, res, next):
    next()
    # modificar response después del handler
)
```

### Archivos estáticos

```kyle
server.static("/static", "./public")
```

### CORS

```kyle
server.cors(origin="*", methods="GET, POST")
```

---

## 6. Plan de implementación

| Fase | Descripción | Depends on | Status |
|------|-------------|------------|--------|
| 1 | Function pointers (`fn()` como tipo) | Compiler | 🔜 |
| 2 | `JsonValue` type + auto-serialize | Union types | 🔜 |
| 3 | HTTP Client con JSON integrado | Fase 2 | 🔜 |
| 4 | Server routing real con callbacks + path params | Fase 1 + 3 | 🔜 |
| 5 | WebSocket + SSE | Fase 4 | 🔜 |
