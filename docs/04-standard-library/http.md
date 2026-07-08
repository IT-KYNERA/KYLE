# http — HTTP Client

> Módulo de cliente y servidor HTTP.
> Import: `from http import client, server, status, method`

## client: HTTP Client

```ky
from http import client

c = client { timeout: 30 }
res = c.get("https://api.example.com/users")
res = c.post("https://api.example.com/users", '{"name": "Kyle"}')
res = c.put("https://api.example.com/users/1", '{"name": "Kyle"}')
res = c.delete("https://api.example.com/users/1")
```

### Constructor

```ky
client { timeout: 30 }        # timeout en segundos (default: 30)
```

### Métodos

| Método | Descripción |
|--------|-------------|
| `c.get(url)` | GET request |
| `c.post(url, body)` | POST request |
| `c.put(url, body)` | PUT request |
| `c.patch(url, body)` | PATCH request |
| `c.delete(url)` | DELETE request |

### response

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `status_code` | i32 | Código HTTP (200, 404, etc.) |
| `status_text` | str | Texto del status ("OK", "Not Found") |
| `body` | str | Cuerpo de la respuesta |
| `is_ok` | bool | `true` si 200-299 |
| `elapsed_ms` | i64 | Milisegundos de la request |

```ky
from http import client

c = client { timeout: 10 }
res = c.get("https://api.github.com/users/kyle")
if res.is_ok:
    println(res.body)
```

## server: HTTP Server

```ky
from http import server, status, method

app = server()

app.get("/hello", fn(req):
    status.ok("Hello World")
)

app.post("/data", fn(req):
    data = req.body()
    status.created(data)
)

app.listen(8080)
```

### status: códigos de respuesta

```ky
from http import status

status.ok()                    # 200
status.created()               # 201
status.not_found()             # 404
status.internal_server_error() # 500
```

### method: métodos HTTP

```ky
from http import method

println(method.get)    # "GET"
println(method.post)   # "POST"
```
