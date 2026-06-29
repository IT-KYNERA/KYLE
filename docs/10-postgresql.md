# PostgreSQL Binding Specification

- **Status:** 📅 Planned
- **Target phase:** Post-v1.0

---

## Purpose

Provide async PostgreSQL database access for Kyle applications. Uses the
native PostgreSQL wire protocol (no ODBC/JDBC dependency).

---

## Proposed Syntax

```kl
import pg

# Connect
conn = pg.connect(
    host="localhost",
    port=5432,
    dbname="myapp",
    user="admin",
    password="secret"
)?

# Query
rows = conn.query("SELECT * FROM users WHERE active = true")?
for row in rows:
    println(row["name"] + ": " + str(row["age"]))

# Typed query
users: [User] = conn.query(User, "SELECT * FROM users")?
for user in users:
    println(user.name)

conn.close()
```

---

## Key Types

| Type | Description |
| :--- | :--- |
| `Connection` | PostgreSQL database connection |
| `Row` | A single result row (dict-like access) |
| `PgError` | Error type for database operations |
| `Pool` | Connection pool (optional) |

---

## Functions

| Function | Signature | Description |
| :--- | :--- | :--- |
| `pg.connect` | `(config: {str: str}) Connection!` | Open connection |
| `conn.query` | `(sql: str, params?: [T]) [Row]!` | Execute query |
| `conn.query_one` | `(sql: str, params?: [T]) Row?!` | Query first row |
| `conn.execute` | `(sql: str, params?: [T]) i64!` | Execute (no results) |
| `conn.close` | `() void` | Close connection |
| `pg.pool` | `(config: {str: str}, size: i32) Pool!` | Create connection pool |

---

## Implementation Notes

- Async by default (uses `async`/`await`)
- Native PostgreSQL wire protocol (no C library dependency)
- Prepared statements for parameterized queries
- Connection pooling with configurable size
- SSL/TLS support via platform TLS
