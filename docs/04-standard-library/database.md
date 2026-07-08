# database — Base de Datos

> Módulo de acceso a bases de datos SQL.
> Import: `from database import sqlite, postgres`

## sqlite: base de datos SQLite

```ky
from database import sqlite

db = sqlite.open("data.db")
db.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER, name TEXT)")
db.execute("INSERT INTO users VALUES (?, ?)", {1, "Kyle"})
rows = db.query("SELECT * FROM users")
for row in rows:
    println(row.get_str("name"))

db.close()
```

### Métodos

| Método | Descripción |
|--------|-------------|
| `sqlite.open(path)` | Abrir base de datos |
| `db.execute(sql, params)` | Ejecutar comando SQL |
| `db.query(sql, params)` | Ejecutar query (retorna rows) |
| `db.close()` | Cerrar conexión |

### row: acceso a columnas

| Método | Descripción |
|--------|-------------|
| `row.get_str(name)` | Columna como string |
| `row.get_i64(name)` | Columna como i64 |
| `row.get_f64(name)` | Columna como f64 |
| `row.get_bool(name)` | Columna como bool |

## postgres: PostgreSQL

```ky
from database import postgres

pool = postgres.pool("postgres://user:pass@localhost/db")
conn = pool.get_conn()
rows = conn.query("SELECT * FROM users")
for row in rows:
    println(row.get_str("name"))
conn.close()
```

### Métodos

| Método | Descripción |
|--------|-------------|
| `postgres.pool(conn_str)` | Crear pool de conexiones |
| `pool.get_conn()` | Obtener conexión del pool |
| `conn.query(sql)` | Ejecutar query |
| `conn.execute(sql)` | Ejecutar comando |
| `conn.close()` | Cerrar conexión |

### Ejemplo

```ky
from database import sqlite

db = sqlite.open("test.db")
db.execute("""
    CREATE TABLE IF NOT EXISTS posts (
        id INTEGER PRIMARY KEY,
        title TEXT NOT NULL,
        body TEXT
    )
""")
db.execute("INSERT INTO posts VALUES (?, ?, ?)", {1, "Hello", "World"})
rows = db.query("SELECT * FROM posts")
for row in rows:
    println(row.get_str("title") + ": " + row.get_str("body"))
db.close()
```
