# http — HTTP Client and Server for Kyle

**Versión:** 4.0  
**Estado:** Especificación

---

## 1. Filosofía

El package `http` unifica cliente y servidor HTTP en una sola biblioteca.
No hay funciones globales `get()`/`post()`. Todo se instancia:

```kyle
from http.client import Client
from http.server import Server
from http import HttpStatus, HttpMethod
```

El cliente y servidor comparten tipos (`HttpStatus`, `HttpMethod`, `Header`)
pero son módulos separados internamente.

---

## 2. Organización del package

```
packages/http/
├── ky.toml
└── src/
    ├── lib.ky           # Tipos compartidos: HttpStatus, HttpMethod, Header, MimeType
    ├── client.ky        # Client class + request execution
    └── server.ky        # Server class + routing + middleware
```

El usuario importa directamente de los submódulos:

```kyle
from http import HttpStatus
from http.client import Client
from http.server import Server
```

---

## 3. Tipos compartidos (`http`)

### `HttpMethod`

```kyle
enum HttpMethod:
    GET
    POST
    PUT
    DELETE
    PATCH
    HEAD
    OPTIONS
```

### `HttpStatus`

```kyle
final class HttpStatus:
    code: i32
    text: str
```

Constantes predefinidas:

| Constante | Código |
|-----------|--------|
| `HttpStatusOk` | 200 |
| `HttpStatusCreated` | 201 |
| `HttpStatusNoContent` | 204 |
| `HttpStatusMovedPermanently` | 301 |
| `HttpStatusNotFound` | 404 |
| `HttpStatusInternalServerError` | 500 |

### `Header`

```kyle
final class Header:
    name: str
    value: str
```

### `MimeType` (constantes)

```kyle
MimeType.json   # "application/json"
MimeType.html   # "text/html"
MimeType.text   # "text/plain"
MimeType.form   # "application/x-www-form-urlencoded"
```

---

## 4. Cliente HTTP (`http.client`)

### Uso básico

```kyle
from http.client import Client

client = Client.new(timeout=30)

# GET
res = client.get("https://api.github.com/repos/IT-KYNERA/KYLE")
if res.is_ok:
    print(res.body)

# POST con JSON
res = client.post(
    "https://jsonplaceholder.typicode.com/posts",
    "{\"title\": \"Kyle\", \"body\": \"hello\", \"userId\": 1}"
)

# PUT
res = client.put("https://api.example.com/users/1", json_data)

# DELETE
res = client.delete("https://api.example.com/users/1")

# PATCH
res = client.patch("https://api.example.com/users/1", json_data)
```

### Configuración del Client

```kyle
client = Client.new(
    timeout=15,                    # segundos
    headers=[
        Header.new("User-Agent", "KyleApp/1.0"),
        Header.new("Accept", "application/json"),
    ],
    follow_redirects=true,
    max_redirects=10,
)
```

### Response

```kyle
final class Response:
    status_code: i32
    status_text: str
    headers: list<Header>
    body: str
    is_ok: bool
    elapsed_ms: i64

    fn header(self, name: str) str:
        # Busca header por nombre (case-insensitive)
```

### Métodos del Client

| Método | Descripción |
|--------|-------------|
| `client.get(url)` | GET request |
| `client.post(url, body)` | POST con body |
| `client.put(url, body)` | PUT con body |
| `client.patch(url, body)` | PATCH con body |
| `client.delete(url)` | DELETE |
| `client.head(url)` | HEAD |
| `client.options(url)` | OPTIONS |

---

## 5. Servidor HTTP (`http.server`)

El servidor recibe **funciones handler** como callbacks, al estilo Express.js.

Cada handler recibe `(req, res, next)` donde:
- `req` — la request entrante
- `res` — helper para construir la respuesta
- `next` — función para pasar al siguiente middleware/ruta

### Uso básico

```kyle
from http.server import Server, Request, Res

server = Server()

# GET /api/health — handler como función anónima
server.get("/api/health", (req, res, next):
    res.json({ "status": "ok" })
)

# POST /api/users — handler como función nombrada
fn create_user(req, res, next):
    body = req.json()
    res.json({ "created": true, "id": 1 }, 201)

server.post("/api/users", create_user)

server.listen(8080)
```

### Router — métodos

```kyle
server.get("/users", handler)          # GET /users
server.get("/users/:id", handler)      # GET /users/42 → req.params["id"]
server.post("/users", handler)         # POST /users
server.put("/users/:id", handler)      # PUT /users/42
server.delete("/users/:id", handler)   # DELETE /users/42
server.patch("/users/:id", handler)    # PATCH /users/42
```

### `Request` — objeto de solicitud

```kyle
final class Request:
    method: HttpMethod
    path: str
    url: str
    params: dict<str, str>      # path params (:id → "42")
    query: dict<str, str>       # query params (?page=1)
    body: str
    
    fn json() dict:             # parsea body como JSON
    fn header(name: str) str    # header por nombre
```

### `Res` — helper de respuesta

```kyle
final class Res:
    status: i32
    body: str

    fn json(data: dict):                   # 200 + JSON
        this.status = 200
        this.body = stringify(data)

    fn json(data: dict, code: i32):        # status + JSON
        this.status = code
        this.body = stringify(data)

    fn text(body: str):                    # 200 + texto
        this.status = 200
        this.body = body

    fn text(body: str, code: i32):         # status + texto
        this.status = code
        this.body = body

    fn redirect(url: str):                 # 302 redirect
        this.status = 302
```

### Middleware

Los middleware son handlers que llaman a `next()` para continuar la cadena.

```kyle
fn auth_handler(req, res, next):
    if req.header("Authorization") == "":
        res.text("unauthorized", 401)
    else:
        next()

server.before("/api/*", auth_handler)

fn cors_handler(req, res, next):
    next()
    # modificar respuesta después

server.after("/api/*", cors_handler)
```

### Archivos estáticos

```kyle
server.static("/static", "./public")
# GET /static/index.html → ./public/index.html
```

### CORS

```kyle
server.cors("*", "GET, POST, PUT, DELETE", "Content-Type, Authorization")
```

---

## 6. Ejemplos completos

### Cliente + JSON

```kyle
from http import Client
from http import HttpStatus

client = Client(10)
res = client.get("https://jsonplaceholder.typicode.com/todos/1")

if res.status_code == HttpStatusOk.code:
    print(res.body)
else:
    print("error: " + str(res.status_code))
```

### Servidor mínimo

```kyle
from http.server import Server

server = Server()

server.get("/", (req, res, next):
    res.json({ "message": "hello world" })
)

server.listen(3000)
```

---

## 7. Fases de implementación

| Fase | Contenido | Estado |
|------|-----------|--------|
| 1 | Tipos compartidos (HttpStatus, HttpMethod, Header) | ✅ |
| 2 | Client con `system("curl")` | ✅ |
| 3 | Client con respuesta tipada (status_text, is_ok, elapsed_ms) | ✅ |
| 4 | Server (router, listen, middleware) via libc sockets | 🔜 |
| 5 | WebSocket sobre Server | 🔜 |
| 6 | SSE sobre Server | 🔜 |

---

## 8. Migración desde v3

| v3 | v4 |
|----|-----|
| `from http import http_get` | `from http import Client` |
| `from http import post` | `client = Client(30)` → `client.post(url, body)` |
| `from http import HttpStatus` | `from http import HttpStatus` (igual) |
| `server.get("/path"):` | `server.get("/path", handler_fn)` |
