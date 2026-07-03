# http — HTTP Client and Server

**Status:** Planned

## Client API

```ky
from http import Client

client = Client.new()
response = client.get("https://example.com")
println(response.status)    # 200
println(response.body)      # "<html>..."
```

## Server API

```ky
from http import Server

server = Server.new()

server.get("/api/users"):

server.start(8080)
```
