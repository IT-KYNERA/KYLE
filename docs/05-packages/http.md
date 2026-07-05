# http — HTTP Client + Server + WebSocket

**Versión:** 6.0  
**Estado:** Especificación (Cliente ✅, Server 🔜, WS 🔜)

---

## 1. Filosofía

Un solo package para todo HTTP. Cliente, servidor y WebSocket comparten tipos.

Nada de funciones globales. Todo se instancia.

```kyle
from http.client import Client
from http.server import Router
from http.websocket import ws_upgrade, ws_read_text, ws_send_text
from http import HttpStatus, HttpMethod, Header
from json import serialize, deserialize
```

---

## 2. Tipos compartidos

### HttpMethod

```kyle
enum HttpMethod:
    GET | POST | PUT | DELETE | PATCH | HEAD | OPTIONS
```

### HttpStatus

```kyle
final class HttpStatus:
    code: i32
    text: str
```

Constantes: `HttpStatusOk` (200), `HttpStatusCreated` (201),
`HttpStatusNotFound` (404), `HttpStatusInternalServerError` (500)

### Header

```kyle
final class Header:
    name: str
    value: str
```

---

## 3. Cliente HTTP (`http.client`)

### Uso

```kyle
from http.client import Client

client = Client { timeout: 10 }

# GET
res = client.get("https://api.github.com/repos/IT-KYNERA/KYLE")
if res.is_ok:
    print(res.body)

# POST con clase → JSON automático
final class User:
    name: str
    age: i32

user = User { name: "Kyle", age: 1 }
res = client.post("https://api.example.com/users", user)

# POST con string raw
res = client.post("https://api.example.com/data", "<xml>...</xml>")

# PUT, PATCH, DELETE
res = client.put(url, data)
res = client.patch(url, data)
res = client.delete(url)
```

### Response

```kyle
final class Response:
    status_code: i32
    status_text: str
    body: str
    is_ok: bool
    elapsed_ms: i64
```

### Auto-JSON

- `client.post(url, data)` donde `data` es `str` → se envía raw
- `client.post(url, data)` donde `data` es `final class` → se serializa a JSON automáticamente
- `client.post(url, data)` donde `data` es `dict` → se serializa a JSON

---

## 4. Servidor HTTP (`http.server`)

El servidor usa un `Router` con handlers como closures, estilo Express.

### Uso básico

```kyle
from http.server import Router

app = Router()

app.get("/health", (req, res):
    res.json({ "status": "ok" })
)

app.listen(8080)
```

### Rutas con parámetros

```kyle
app.get("/users/{id}", (req, res):
    id = req.param("id")         # str
    res.json({ "user": id })
)

app.get("/users/{id:i32}", (req, res):
    id = req.param("id")         # i32 — parseado automático
    res.json({ "user": id })
)
```

### Métodos del Router

| Método | Descripción |
|--------|-------------|
| `app.get(path, handler)` | GET route |
| `app.post(path, handler)` | POST route |
| `app.put(path, handler)` | PUT route |
| `app.patch(path, handler)` | PATCH route |
| `app.delete(path, handler)` | DELETE route |
| `app.listen(port)` | Inicia servidor |

### Request

```kyle
final class Request:
    method: HttpMethod
    path: str
    params: dict<str, str>      # path params: {id} → "42"
    query: dict<str, str>       # query: ?page=1
    body: str                   # raw body string
    headers: dict<str, str>     # request headers

    fn param[T](name: str) T    # path param con tipo
    fn header(name: str) str    # header por nombre
    fn body[T]() T              # body parseado como JSON → clase T
```

### Res

```kyle
final class Res:
    fn json(data):                  # 200 + JSON (cualquier clase serializable)
    fn json(data, code: i32):       # status + JSON
    fn text(body: str):             # 200 + texto plano
    fn text(body: str, code: i32):  # status + texto
    fn redirect(url: str):          # 302 redirect
    fn status(code: i32):           # solo status, sin body
```

Cualquier `final class` se serializa a JSON automáticamente en `res.json()`.

### Middleware

```kyle
# Before — se ejecuta antes de las rutas
app.before("/api/*", (req, res, next):
    if req.header("Authorization") == "":
        res.text("unauthorized", 401)
    else:
        next()
)

# After — se ejecuta después de las rutas
app.after("/api/*", (req, res, next):
    next()
    res.header("X-Powered-By", "Kyle")
)
```

Los middleware reciben `(req, res, next)` y deben llamar `next()` para continuar la cadena. Si no llaman `next()`, la respuesta se envía inmediatamente.

### Archivos estáticos

```kyle
app.static("/static", "./public")
# GET /static/index.html → ./public/index.html
```

### CORS

```kyle
app.cors(
    origin="*",
    methods="GET, POST, PUT, DELETE",
    headers="Content-Type, Authorization",
)
```

### Error handling

```kyle
app.get("/users/{id:i32}", (req, res):
    user = find_user(req.param("id"))
    if user == none:
        res.text("Not Found", 404)
    else:
        res.json(user)
)
```

### Ejemplo completo: API REST

```kyle
from http.server import Router
from json import deserialize

final class User:
    name: str
    age: i32

final class CreateUser:
    name: str
    age: i32

app = Router()

# Listar usuarios
app.get("/users", (req, res):
    res.json([
        { "id": 1, "name": "Ana" },
        { "id": 2, "name": "Juan" },
    ])
)

# Obtener usuario por ID
app.get("/users/{id:i32}", (req, res):
    id = req.param("id")
    res.json({ "id": id, "name": "User " + str(id) })
)

# Crear usuario
app.post("/users", (req, res):
    data = req.body[CreateUser]()
    res.json({ "created": true, "name": data.name }, 201)
)

app.listen(3000)
```

---

## 5. WebSocket (`http.websocket`)

WebSocket se maneja como una ruta más. El Router hace el upgrade automático.

### Echo server

```kyle
from http.server import Router

app = Router()

app.ws("/echo", (ws):
    ws.on("message", (msg):
        ws.send(msg)
    )
)

app.listen(8080)
```

### Chat server

```kyle
from http.server import Router

app = Router()

app.ws("/chat", (ws):
    ws.on("message", (msg):
        ws.broadcast(msg)
    )
)

app.listen(8080)
```

### WebSocket handler

```kyle
app.ws("/path", (ws):
    ws.on("message", (msg): ...)   # mensaje de texto
    ws.on("binary", (data): ...)   # datos binarios
    ws.on("close", (): ...)        # cierre de conexión
    ws.on("ping", (): ...)         # ping (auto-responder)
)
```

### WebSocket object

```kyle
final class WebSocket:
    fn send(text: str)           # enviar mensaje texto
    fn send(data: bytes)         # enviar datos binarios
    fn broadcast(msg: str)       # enviar a todos los conectados
    fn broadcast(msg: str, except: list<WebSocket>)  # a todos menos uno
    fn close()                   # cerrar conexión
    fn on(event: str, handler)   # registrar evento
```

---

## 6. Plan de implementación

| Fase | Descripción | Depende de | Estado |
|------|-------------|------------|--------|
| 1 | Function pointers (closes como valores) | Compiler | ✅ |
| 2 | JSON: serialize/deserialize<T> | Fase 1 | ✅ |
| 3 | HTTP Client con auto-JSON | Fase 2 | ✅ |
| 4 | **Router: rutas, path params, middleware** | Fase 1 + 3 | 🔜 |
| 5 | WebSocket + SSE | Fase 4 | 🔜 |
| 6 | PostgreSQL package | — | 🔜 |
| 7 | WASM target + ky-web | — | 🔜 |
