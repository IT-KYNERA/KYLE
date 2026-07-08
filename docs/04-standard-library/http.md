# http — HTTP Client

> Módulo de cliente y servidor HTTP.
> Import: `from http import client, server, status, method`

## client: HTTP Client

```ky
from http import client

c: client = client { timeout: 30 }
res: response = c.get("https://api.example.com/users")
res = c.post("https://api.example.com/users", '{"name": "Kyle"}')
```

### response

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `status_code` | `i32` | Código HTTP (200, 404, etc.) |
| `status_text` | `str` | Texto del status |
| `body` | `str` | Cuerpo de la respuesta |
| `is_ok` | `bool` | `true` si 200-299 |
| `elapsed_ms` | `i64` | Milisegundos de la request |

## server: HTTP Server

```ky
from http import server, status

app: server = server()

app.get("/hello", fn(req: request) response:
    status.ok("Hello World")
)

app.listen(8080)
```

## status: códigos de respuesta

```ky
from http import status

status.ok()                    # 200
status.created()               # 201
status.not_found()             # 404
status.internal_server_error() # 500
```

## method: métodos HTTP

```ky
from http import method

println(method.get)    # "GET"
println(method.post)   # "POST"
```
