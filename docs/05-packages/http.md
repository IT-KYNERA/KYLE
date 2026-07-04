# http — HTTP Client and Server for Kyle

**Version:** 3.0
**Estado:** En desarrollo (Cliente v3, Servidor planeado)

---

## 1. Filosofía

El package `http` debe ser la puerta de entrada a la web para Kyle. No un wrapper, sino
una biblioteca con identidad propia: tipada, idiomática, eficiente.

La implementación actual usa `system("curl ...")` como fase transitoria. La arquitectura
está diseñada para migrar a libcurl directo por FFI sin cambios en la API pública.

---

## 2. Principios de diseño

| Principio | Significado |
|-----------|-------------|
| **Tipado fuerte** | Cada concepto tiene su tipo: `HttpMethod`, `HttpStatus`, `Header`, `MimeType`. Sin strings mágicos. |
| **API idiomática** | Kyle no es JavaScript. No hay option-objects, no hay `new` a pelo. Constructores y métodos con nombres claros. |
| **Inmutable por defecto** | `Request` y `Response` son valores. `Client` tiene estado (conexiones, cookies). |
| **Orientado a casos reales** | 80% del uso es GET/POST con JSON. Esos flujos deben ser una línea. |
| **Extensible sin romper** | El core es estable. Las features avanzadas (websocket, http2) son módulos aparte. |

---

## 3. Organización del package

```
packages/http/
├── ky.toml                  # metadata del package
└── src/
    ├── lib.ky               # API pública (todo se exporta desde aquí)
    ├── client.ky            # Client + configuración
    ├── request.ky           # Request builder
    ├── response.ky          # Response + parseo
    ├── types.ky             # HttpMethod, HttpStatus, MimeType, Header
    ├── cookie.ky            # Cookie jar
    ├── multipart.ky         # Multipart form data
    ├── auth.ky              # Autenticación (Basic, Bearer, Digest)
    └── server/              # (futuro)
        ├── server.ky
        ├── router.ky
        └── middleware.ky
```

Cada archivo se importa internamente. El usuario solo ve:

```ky
from http import Client, Response, HttpStatus
```

---

## 4. Tipos públicos

### `HttpMethod` — enumeración tipada de métodos HTTP

```ky
enum HttpMethod:
    GET
    POST
    PUT
    DELETE
    PATCH
    HEAD
    OPTIONS
    CONNECT
    TRACE
```

### `HttpStatus` — enumeración de códigos de estado

```ky
enum HttpStatus:
    # 2xx Success
    OK = 200
    Created = 201
    Accepted = 202
    NoContent = 204

    # 3xx Redirect
    MovedPermanently = 301
    Found = 302
    NotModified = 304

    # 4xx Client Error
    BadRequest = 400
    Unauthorized = 401
    Forbidden = 403
    NotFound = 404
    Conflict = 409
    TooManyRequests = 429

    # 5xx Server Error
    InternalServerError = 500
    BadGateway = 502
    ServiceUnavailable = 503
    GatewayTimeout = 504
```

### `MimeType` — constantes de tipos MIME

```ky
final class MimeType:
    json: str = "application/json"
    html: str = "text/html"
    text: str = "text/plain"
    xml:  str = "application/xml"
    form: str = "application/x-www-form-urlencoded"
    multipart: str = "multipart/form-data"
    octet_stream: str = "application/octet-stream"
    png:  str = "image/png"
    jpeg: str = "image/jpeg"
    gif:  str = "image/gif"
    svg:  str = "image/svg+xml"
    pdf:  str = "application/pdf"
    zip:  str = "application/zip"
    mp4:  str = "video/mp4"
    css:  str = "text/css"
    js:   str = "application/javascript"
```

### `Header` — par nombre/valor

```ky
final class Header:
    name: str
    value: str
```

### `Request` — solicitud tipada

```ky
final class Request:
    method: HttpMethod
    url: str
    headers: list<Header>
    query: dict<str, str>
    body: str
    timeout: i32
    follow_redirects: bool
```

### `Response` — respuesta completa

```ky
final class Response:
    status: HttpStatus
    status_code: i32        # acceso directo al código numérico
    status_text: str        # "OK", "Not Found", etc.
    headers: list<Header>
    headers_dict: dict<str, str>  # acceso rápido por nombre
    body: str
    ok: bool                # true si status < 400
    elapsed_ms: i64         # tiempo de respuesta
    final_url: str          # URL después de redirecciones

    fn json(self) dict<str, i64>:
        # parsea body como JSON
```

### `ClientConfig` — configuración del cliente

```ky
final class ClientConfig:
    timeout: i32            # segundos, default 30
    headers: list<Header>   # headers por defecto
    follow_redirects: bool  # default true
    max_redirects: i32      # default 10
    proxy: str              # opcional
    user_agent: str         # default "Kyle/x.y.z"
```

### `Client` — el cliente HTTP

```ky
final class Client:
    config: ClientConfig
    cookie_jar: list<Cookie>

    fn get(url: str) Response:
    fn get(url: str, options: RequestOptions) Response:

    fn post(url: str, body: str) Response:
    fn post(url: str, body: str, options: RequestOptions) Response:

    fn put(url: str, body: str) Response:
    fn patch(url: str, body: str) Response:
    fn delete(url: str) Response:
    fn head(url: str) Response:
    fn options(url: str) Response:

    fn request(req: Request) Response:

    # helpers
    fn get_json(url: str) dict:
    fn post_json(url: str, data: dict) Response:
```

### Funciones de alto nivel (convenience)

```ky
fn get(url: str) Response
fn get(url: str, options: RequestOptions) Response
fn post(url: str, body: str) Response
fn put(url: str, body: str) Response
fn patch(url: str, body: str) Response
fn delete(url: str) Response
fn head(url: str) Response
```

---

## 5. Ejemplos de uso

### GET simple

```ky
from http import get, Response

res = get("https://api.github.com/repos/IT-KYNERA/KYLE")
if res.ok:
    data = res.json()
    print(data["description"])
else:
    print("Error: " + str(res.status_code))
```

### GET con headers y query params

```ky
from http import Client, Header

client = Client.new(
    timeout=15,
    headers=[
        Header.new("User-Agent", "KyleApp/1.0"),
        Header.new("Accept", "application/json"),
    ],
)

res = client.get("https://api.github.com/search/repositories",
    {"q": "language:Kyle", "sort": "stars"})

if res.ok:
    repos = res.json()
    for repo in repos["items"]:
        print(repo["full_name"])
```

### POST con JSON

```ky
from http import post

res = post("https://jsonplaceholder.typicode.com/posts",
    '{"title": "Kyle", "body": "lenguaje chido", "userId": 1}')

if res.status_code == 201:
    created = res.json()
    print("Created ID: " + str(created["id"]))
```

### POST con form data

```ky
from http import Client, MimeType

client = Client.new()
res = client.post("https://httpbin.org/post", "key1=value1&key2=value2")

# el Content-Type se auto-detecta como application/x-www-form-urlencoded
```

### Autenticación

```ky
from http import Client

# Bearer token
client = Client.new(
    headers=[Header.new("Authorization", "Bearer " + token)]
)

# Básica
from http import auth
basic_token = auth.basic("user", "pass")
client = Client.new(
    headers=[Header.new("Authorization", basic_token)]
)
```

### Subida de archivo

```ky
from http import Client, Multipart

client = Client.new()
res = client.post("https://httpbin.org/post",
    Multipart.new()
        .add_field("name", "Kyle")
        .add_file("avatar", "./profile.png"))
```

### Manejo de errores

```ky
from http import get, HttpStatus, HttpException

res = get("https://api.example.com/data")
match res.status:
    HttpStatus.OK:
        process(res.json())
    HttpStatus.NotFound:
        print("recurso no encontrado")
    HttpStatus.InternalServerError:
        print("error del servidor, reintentar luego")
    _:
        print("error " + str(res.status_code) + ": " + res.status_text)
```

---

## 6. Arquitectura interna

### Capas

```
┌─────────────────────────────────────────────┐
│             API Pública (lib.ky)            │
│  get, post, put, delete, patch, head       │
│  Client, Request, Response, HttpStatus     │
├─────────────────────────────────────────────┤
│           Capa de abstracción              │
│  Client → ejecuta request                  │
│  Response → parsea desde raw               │
│  CookieJar → gestiona cookies             │
├─────────────────────────────────────────────┤
│            Transporte (FFI)                │
│  extern fn curl_easy_*                    │
│  @link "curl"                              │
│  (hoy: system("curl ..."))                 │
└─────────────────────────────────────────────┘
```

### Flujo de una request

```
1. El usuario llama a `client.get(url)`
2. Se construye un `Request` interno con el método, URL, headers, body
3. Se serializa a formato curl (hoy) o llamada libcurl FFI (futuro)
4. Se ejecuta la petición
5. Se parsea la respuesta cruda en un `Response` tipado
6. Se devuelve `Response` al usuario
```

---

## 7. Implementación: Fases

| Fase | Feature | Entrega |
|------|---------|---------|
| **3.0** | Client class, HttpMethod enum, HttpStatus enum, Response rico, headers, query params, auth básico | ✅ Ahora |
| **3.1** | Cookie jar, multipart, streaming, progress | 📅 |
| **3.2** | Migrar de `system("curl")` a libcurl FFI directo | 📅 |
| **3.3** | HTTP/2 | 📅 |
| **4.0** | Servidor HTTP (router, middleware, static) | 📅 |
| **4.1** | WebSocket, SSE | 📅 |
| **4.2** | HTTP/3, QUIC | 📅 |

---

## 8. Documentación del package

La documentación oficial de cada package vive en dos lugares:

1. **`packages/<name>/README.md`** — Documentación técnica: API, ejemplos, changelog.
   Esto es lo que ve el desarrollador cuando usa el package.

2. **`docs/05-packages/<name>.md`** — Especificación de arquitectura: diseño, decisiones,
   plan de implementación. Esto es para maintainers y contribuidores.

Para el usuario final:

```bash
# El package se documenta a sí mismo
ky add http
# Luego en el código:
from http import Client

# Y en el README del package está la referencia completa
```

---

## 9. Referencia rápida

```ky
# ── Convenience functions ──
get(url) -> Response
get(url, options) -> Response
post(url, body) -> Response
put(url, body) -> Response
delete(url) -> Response

# ── Client ──
client = Client.new(timeout=30, headers=[])
client.get(url)
client.post(url, body)
client.put(url, body)
client.patch(url, body)
client.delete(url)
client.head(url)
client.options(url)
client.request(Request)

# ── Response ──
res.status       -> HttpStatus enum
res.status_code  -> i32
res.status_text  -> str
res.headers      -> list<Header>
res.body         -> str
res.ok           -> bool
res.elapsed_ms   -> i64
res.final_url    -> str
res.json()       -> dict

# ── RequestOptions (passed to get/post/...) ──
{
    "headers": [Header],
    "query": {"key": "val"},
    "auth": "Bearer ...",
    "timeout": 30,
}
```

---

## 10. Notas de migración desde v2

| v2 | v3 |
|----|----|
| `fetch(url)` | `get(url)` |
| `Response.status_code` | `Response.status_code` (igual) |
| `Response.body` | `Response.body` (igual) |
| `post(url, body)` | `post(url, body)` (igual) |
| `Client.get(...)` | `Client.get(...)` (nuevo, v2 no tenía Client) |
| `{ "timeout": 30 }` dict | `Client.new(timeout=30)` constructor tipado |
