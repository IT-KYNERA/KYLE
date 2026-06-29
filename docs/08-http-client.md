# HTTP Client Library Specification

- **Status:** 📅 Planned
- **Target phase:** Post-v1.0

---

## Purpose

Provide a simple, ergonomic HTTP client for making requests to web servers.
Built on top of the async runtime (Phase 9).

---

## Proposed Syntax

```kl
import http

# Simple GET
response = await http.get("https://api.example.com/users")
println(response.body)

# POST with JSON body
response = await http.post(
    "https://api.example.com/users",
    json={"name": "Alice", "role": "admin"}
)

# With headers and timeout
response = await http.get(
    "https://api.example.com/data",
    headers={"Authorization": "Bearer token123"},
    timeout=5000
)

println(response.status_code)
```

---

## Key Types

| Type | Fields | Description |
| :--- | :--- | :--- |
| `Response` | `status_code: i32`, `body: str`, `headers: {str: str}` | HTTP response |
| `RequestError` | (error variant) | Connection, timeout, parse errors |

---

## Functions

| Function | Signature | Description |
| :--- | :--- | :--- |
| `http.get` | `(url: str, ...) Response!` | GET request |
| `http.post` | `(url: str, ...) Response!` | POST request |
| `http.put` | `(url: str, ...) Response!` | PUT request |
| `http.delete` | `(url: str, ...) Response!` | DELETE request |
| `http.patch` | `(url: str, ...) Response!` | PATCH request |

All functions accept optional keyword arguments: `headers`, `json`, `body`,
`timeout`.

---

## Implementation Notes

- Uses platform TLS (OpenSSL on Linux, Secure Transport on macOS)
- Async by default (uses `async`/`await`)
- Connection pooling for reuse
- Automatic JSON encoding/decoding when `json` kwarg is used
