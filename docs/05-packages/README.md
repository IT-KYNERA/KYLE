# Packages — Official Kyle Packages

## Uso rápido

```bash
ky add http      # HTTP client + server + Router
ky add json      # JSON parse + stringify
ky add sqlite    # SQLite bindings
```

Todos los packages están escritos **100% en Kyle** usando `extern fn` + `@link` para FFI con librerías C.

---

## http — HTTP Client + Server + Router

**Package:** `http` · **Import:** `from http.server import Router` / `from http import Client`

### Server (Router)

```python
from http.server import Router

app = Router()

app.get("/ping", (req, res):
    res.json({"status": "ok"}, 200)           # dict → JSON automático
)

app.get("/users/{id}", (req, res):
    id = req.param("id")
    res.json({"user": id, "name": "Alice"}, 200)
)

app.post("/data", (req, res):
    body = req.body()                          # raw HTTP body
    res.json_str(body, 200)                    # raw JSON string
)

app.listen(8080)
```

**Métodos de `Res`:**

| Método | Descripción |
|--------|-------------|
| `res.json(data: {str: str}, code)` | Serializa dict a JSON automáticamente |
| `res.json_str(data: str, code)`    | Envía JSON string crudo |
| `res.text(body: str, code)`        | Envía texto plano |

### Client

```python
from http import Client

client = Client(timeout=10)
resp = client.get("https://api.example.com/users")
print(resp.status_code, resp.body)
```

---

## json — JSON parse + stringify

**Package:** `json` · **Import:** `from json import parse, stringify`

```python
from json import parse, stringify

data = parse('{"count": 42}')          # → {str: i64}
s    = stringify({"count": 42})        # → '{"count":42}'
```

**Nota:** `parse` devuelve `{str: i64}` (valores numéricos). Para `{str: str}` usa `json_stringify_str` (builtin global).

---

## sqlite — SQLite database

**Package:** `sqlite` · **Import:** `from sqlite import Database`

```python
from sqlite import Database

db = Database("app.db")
db.execute("CREATE TABLE IF NOT EXISTS users (id i32, name str)")
db.execute("INSERT INTO users VALUES (?, ?)", 1, "Alice")
rows = db.query("SELECT * FROM users")
```

---

## Package manager

| Comando | Descripción |
|---------|-------------|
| `ky add http` | Agrega dependencia + instala |
| `ky remove http` | Elimina dependencia |
| `ky install` | Instala todas las deps desde ky.lock |
| `ky update` | Actualiza a últimas versiones compatibles |

Ver [registry.md](registry.md) para más detalles.<｜end▁of▁thinking｜>

<｜｜DSML｜｜tool_calls>
<｜｜DSML｜｜invoke name="read">
<｜｜DSML｜｜parameter name="filePath" string="true">/Users/kynera/HCA/KYNERA/kl/docs/04-platform/standard-library/overview.md
