# net — Red

> Módulo de redes (TCP).
> Import: `from net import tcp`

## tcp: conexiones TCP

```ky
from net import tcp

# Servidor
server: tcp = tcp.listen(8080)
client: tcp = server.accept()
data: str = client.read(1024)
client.write("HTTP/1.1 200 OK\r\n\r\n")
client.close()
server.close()

# Cliente
conn: tcp = tcp.connect("example.com", 80)
conn.write("GET / HTTP/1.1\r\nHost: example.com\r\n\r\n")
resp: str = conn.read(4096)
conn.close()
```

### Métodos (socket servidor)

| Método | Firma | Descripción |
|--------|-------|-------------|
| `tcp.listen(port)` | `fn(port: i32) tcp` | Crear socket servidor |
| `s.accept()` | `fn() tcp` | Aceptar conexión |
| `s.close()` | `fn()` | Cerrar socket |

### Métodos (socket cliente)

| Método | Firma | Descripción |
|--------|-------|-------------|
| `tcp.connect(host, port)` | `fn(host: str, port: i32) tcp` | Conectar |
| `c.read(count)` | `fn(count: i32) str` | Leer hasta N bytes |
| `c.write(data)` | `fn(data: &str)` | Enviar datos |
| `c.close()` | `fn()` | Cerrar conexión |
