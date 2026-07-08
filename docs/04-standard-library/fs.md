# fs — Sistema de Archivos

> Módulo de operaciones con archivos.
> Import: `from fs import file`

## file: lectura y escritura de archivos

```ky
from fs import file

# Escritura
f: file = file.open("/tmp/test.txt", "w")
f.write("hello world")
f.close()

# Lectura
f = file.open("/tmp/test.txt", "r")
content: str = f.read()
f.close()
println(content)
```

### Métodos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `file.open(path, mode)` | `fn(path: str, mode: str) file` | Abrir archivo |
| `f.read()` | `fn(self) str` | Leer todo como string |
| `f.read_bytes(count)` | `fn(self, count: i64) bytes` | Leer N bytes |
| `f.write(text)` | `fn(self, text: &str)` | Escribir texto |
| `f.write_bytes(data)` | `fn(self, data: &bytes)` | Escribir bytes |
| `f.close()` | `fn(self)` | Cerrar archivo |
| `f.exists()` | `fn(self) bool` | `true` si existe |
| `f.len()` | `fn(self) i64` | Tamaño en bytes |

### Modos de apertura

| Modo | Descripción |
|------|-------------|
| `"r"` | Lectura (texto) |
| `"w"` | Escritura (texto, truncar) |
| `"a"` | Append (texto) |
| `"rb"` | Lectura (binario) |
| `"wb"` | Escritura (binario) |

### Ejemplo

```ky
from fs import file

content: str = file.open("data.txt", "r").read()
lines: {str} = content.split("\n")
println("líneas: " + lines.len().to_str())
```
