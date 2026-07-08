# net — Red

> Módulo de redes (TCP, UDP).
> Import: `from net import tcp, udp`

## tcp: conexiones TCP

```ky
from net import tcp

# Servidor
server = tcp.listen(8080)
client = server.accept()
data = client.read(1024)
client.write("HTTP/1.1 200 OK\r\n\r\n")
client.close()
server.close()

# Cliente
conn = tcp.connect("example.com", 80)
conn.write("GET / HTTP/1.1\r\nHost: example.com\r\n\r\n")
resp = conn.read(4096)
conn.close()
```

### Métodos (socket servidor)

| Método | Descripción |
|--------|-------------|
| `tcp.listen(port)` | Crear socket servidor |
| `s.accept()` | Aceptar conexión (retorna socket cliente) |
| `s.close()` | Cerrar socket |

### Métodos (socket cliente)

| Método | Descripción |
|--------|-------------|
| `tcp.connect(host, port)` | Conectar a servidor |
| `c.read(count)` | Leer hasta N bytes |
| `c.write(data)` | Enviar datos |
| `c.close()` | Cerrar conexión |

## udp: UDP

```ky
from net import udp

sock = udp.bind(8080)
data, addr = sock.recv_from(1024)
sock.send_to("response", addr)
sock.close()
```

### Métodos

| Método | Descripción |
|--------|-------------|
| `udp.bind(port)` | Bind a puerto local |
| `s.recv_from(count)` | Recibir datagrama |
| `s.send_to(data, addr)` | Enviar datagrama |
| `s.close()` | Cerrar socket |
