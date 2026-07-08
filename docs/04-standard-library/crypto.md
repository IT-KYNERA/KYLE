# crypto — Criptografía

> Módulo de funciones criptográficas básicas.
> Import: `from crypto import crypto`

## crypto: hash, encoding

```ky
from crypto import crypto

# SHA-1
hash = crypto.sha1("data")
println(hash)   # hex string

# Base64
encoded = crypto.base64_encode("hello world")
decoded = crypto.base64_decode(encoded)

# Digest
digest = crypto.digest("sha256", "data")
```

### Funciones

| Función | Descripción |
|---------|-------------|
| `crypto.sha1(data)` | SHA-1 hash (retorna hex string) |
| `crypto.sha256(data)` | SHA-256 hash (retorna hex string) |
| `crypto.base64_encode(data)` | Codificar a base64 |
| `crypto.base64_decode(str)` | Decodificar base64 |

### Ejemplo

```ky
from crypto import crypto

token = crypto.sha1("user:password")
println("token: " + token)
```
