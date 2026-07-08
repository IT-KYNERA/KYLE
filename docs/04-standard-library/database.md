# database — Base de Datos

> Módulo de acceso a bases de datos SQL.
> Import: `from database import sqlite, postgres`

## sqlite: base de datos SQLite

```ky
from database import sqlite

db: sqlite = sqlite.open("data.db")
db.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER, name TEXT)")
db.execute("INSERT INTO users VALUES (?, ?)", {1, "Kyle"})
rows: {row} = db.query("SELECT * FROM users")
for row in rows:
    name: str = row.get_str("name")
    println(name)
db.close()
```

### Métodos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `sqlite.open(path)` | `fn(path: str) sqlite` | Abrir base de datos |
| `db.execute(sql, params)` | `fn(sql: str, params: {i64})` | Ejecutar comando SQL |
| `db.query(sql, params)` | `fn(sql: str, params: {i64}) {row}` | Ejecutar query |
| `db.close()` | `fn()` | Cerrar conexión |

### row: columnas

| Método | Firma | Descripción |
|--------|-------|-------------|
| `row.get_str(name)` | `fn(name: str) str` | Columna como string |
| `row.get_i64(name)` | `fn(name: str) i64` | Columna como entero |
| `row.get_f64(name)` | `fn(name: str) f64` | Columna como float |
| `row.get_bool(name)` | `fn(name: str) bool` | Columna como bool |

## postgres: PostgreSQL

```ky
from database import postgres

pool: postgres = postgres.pool("postgres://user:pass@localhost/db")
conn: postgres = pool.get_conn()
rows: {row} = conn.query("SELECT * FROM users")
for row in rows:
    println(row.get_str("name"))
conn.close()
```

### Métodos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `postgres.pool(conn_str)` | `fn(s: str) postgres` | Crear pool |
| `pool.get_conn()` | `fn() postgres` | Obtener conexión |
| `conn.query(sql)` | `fn(sql: str) {row}` | Ejecutar query |
| `conn.execute(sql)` | `fn(sql: str)` | Ejecutar comando |
| `conn.close()` | `fn()` | Cerrar conexión |
