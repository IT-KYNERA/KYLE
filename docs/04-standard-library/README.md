# 04-standard-library

> Kyle standard library. Each module is a namespace that you import with `from module import ...`.

## Modules

| Module | Import | Description |
|--------|--------|-------------|
| `core` | `option`, `result` | `option<T>`, `result<T>` with methods |
| `collections` | `list`, `set`, `iter` | Dynamic lists, sets, iterators |
| `strings` | `str`, `str_builder` | String utilities, builder |
| `io` | `print`, `println`, `input` | Console I/O |
| `fs` | `file` | File operations |
| `path` | `path` | Path manipulation |
| `net` | `tcp` | TCP networking |
| `http` | `client`, `server` | HTTP client/server |
| `json` | `json` | JSON parse/stringify |
| `xml` | `xml` | XML parse/generate |
| `math` | `math` | Math functions |
| `random` | `random` | Random numbers |
| `time` | `date_time`, `duration` | Date, time, duration |
| `process` | `process` | OS processes |
| `thread` | `thread` | OS threads |
| `sync` | `mutex`, `atomic`, `channel` | Synchronization primitives |
| `crypto` | `crypto` | Cryptographic functions |
| `regex` | `regex` | Regular expressions |
| `serialization` | `serialize` | Serialization |
| `database` | `sqlite`, `postgres` | Database access |
| `testing` | `assert` | Test assertions |

## Conventions

- All modules imported explicitly: `from math import math`
- Functions called with namespace: `math.max(a, b)`
- snake_case everywhere: functions, types, methods
- `T` uppercase = type parameter (generics)
