# http — HTTP Client and Server for Kyle

**Version:** 2.0  
**Status:** Planned

## Client API

### Quick start

```ky
from http import Client

# Simple GET
response = Client.get("https://api.example.com/users")
println(response.status_code)   # 200
println(response.body)          # JSON string
println(response.headers)       # dict of response headers
```

### Client configuration

```ky
client = Client.new({
    "timeout": 30,
    "headers": {
        "User-Agent": "Kyle/1.0",
        "Accept": "application/json",
    }
})

# These headers are sent with every request
response = client.get("https://api.example.com/users")
```

### Methods

```ky
# GET
response = Client.get(url)
response = client.get(url)
response = client.get(url, {"headers": {"Authorization": "Bearer token"}})

# POST with JSON body
response = Client.post(url, json_string)
response = client.post(url, json_string)
response = client.post(url, json_string, {"headers": {"X-Custom": "value"}})

# PUT
response = Client.put(url, json_string)

# DELETE
response = Client.delete(url)

# PATCH
response = Client.patch(url, json_string)
```

### Response object

```ky
response.status_code    # i32 — 200, 404, 500, etc.
response.body           # str — raw response body
response.headers        # dict<str, str> — response headers
response.ok             # bool — true if status < 400
response.json()         # dict — parse body as JSON
```

### Options (per-request or per-client)

```ky
{
    "timeout": 30,                    # seconds (default: 30)
    "headers": {"Key": "Value"},      # custom headers
    "params": {"page": "1"},          # query params (auto-appended to URL)
    "auth": {"bearer": "token"},      # Bearer token auth
    "auth": {"basic": ["user", "pass"]},  # Basic auth
}
```

### Error handling

```ky
response = Client.get("https://api.example.com")
if response.ok:
    data = response.json()
    println(data)
else:
    println("error: {response.status_code}")
```

---

## Server API

### Quick start

```ky
from http import Server

server = Server.new()

server.get("/api/health"):
    { "status": "ok" }

server.start(8080)
println("Server running on port 8080")
```

### Routes

```ky
server.get("/api/users"):
    { "users": ["ana", "juan"] }

server.get("/api/users/:id"):
    id = request.params["id"]
    { "user": id, "name": "User {id}" }

server.post("/api/users"):
    data = request.json()
    println("created user: {data}")
    { "created": true, "id": 1 }

server.put("/api/users/:id"):
    data = request.json()
    { "updated": true, "id": request.params["id"] }

server.delete("/api/users/:id"):
    { "deleted": true, "id": request.params["id"] }
```

### Request object

```ky
request.method       # str — "GET", "POST", etc.
request.url          # str — full URL
request.path         # str — path only
request.params       # dict<str, str> — path params (:id, :name)
request.query        # dict<str, str> — query params (?page=1)
request.headers      # dict<str, str> — request headers
request.body         # str — raw body
request.json()       # dict — parsed JSON body
```

### Response helpers

```ky
# Return JSON (dict is auto-stringified)
server.get("/api/data"):
    { "key": "value" }

# Return with status code
server.get("/api/create"):
    { "created": true }, 201

# Return with custom headers
server.get("/api/data"):
    { "data": "value" }, 200, {"X-Custom": "value"}
```

### Middleware

```ky
server.before("/api/*"):
    auth = request.headers["Authorization"]
    if auth == "":
        { "error": "unauthorized" }, 401

server.after("/api/*"):
    response.headers["X-Powered-By"] = "Kyle"
```

### CORS

```ky
server.cors({
    "origin": "*",
    "methods": "GET, POST, PUT, DELETE",
    "headers": "Content-Type, Authorization",
})
```

### Static files

```ky
server.static("/static", "./public")
# Serves ./public/index.html at /static/index.html
```

---

## Implementation plan

### Phase 1 — Client MVP (current)
- GET, POST, PUT, DELETE via curl system call
- Returns (status, body)
- 2 days work

### Phase 2 — Client full
- `Client` class with config
- Custom headers, query params, auth
- Response object with status, body, headers, ok
- 3-4 days work

### Phase 3 — Server MVP
- Basic routing (GET, POST, PUT, DELETE)
- Path params (:id)
- JSON response auto-serialization
- 5-7 days work

### Phase 4 — Server full
- Middleware (before/after)
- CORS
- Static files
- Request validation
- 1-2 weeks work

---

**Current status:** Phase 1 complete. Phase 2 planned.
