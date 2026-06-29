# SQLite Binding Specification

- **Status:** 📅 Planned
- **Target phase:** Post-v1.0

---

## Purpose

Provide safe, ergonomic access to SQLite databases from Kyle. Wraps the
C SQLite3 library with Kyle's error-handling conventions.

---

## Proposed Syntax

```kl
import sqlite

# Open a database
db = sqlite.open("myapp.db")?

# Create a table
db.execute("CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE
)")?

# Insert with parameters
db.execute(
    "INSERT INTO users (name, email) VALUES (?, ?)",
    ["Alice", "alice@example.com"]
)?

# Query
rows = db.query("SELECT * FROM users WHERE id = ?", [1])?
for row in rows:
    println(row["name"])

# Close
db.close()
```

---

## Key Types

| Type | Description |
| :--- | :--- |
| `Database` | Open SQLite database connection |
| `Row` | A single result row (dict-like access) |
| `SqlError` | Error type for database operations |

---

## Functions

| Function | Signature | Description |
| :--- | :--- | :--- |
| `sqlite.open` | `(path: str) Database!` | Open or create database |
| `db.execute` | `(sql: str, params?: [T]) i64!` | Execute statement (returns affected rows) |
| `db.query` | `(sql: str, params?: [T]) [Row]!` | Execute query, return rows |
| `db.query_one` | `(sql: str, params?: [T]) Row?!` | Execute query, return first row or None |
| `db.transaction` | `(fn: (Database) T!) T!` | Execute in transaction (auto-commit/rollback) |
| `db.close` | `() void` | Close connection |

---

## Implementation Notes

- Links against `libsqlite3` (vendored or system)
- Parameterized queries prevent SQL injection
- `Row` values are lazily converted from SQLite types to Kyle types
- Transactions are `BEGIN IMMEDIATE` by default
