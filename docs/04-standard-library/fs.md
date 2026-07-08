# fs — Sistema de Archivos

> Módulo de operaciones con archivos.
> Import: `from fs import file`

## file: lectura y escritura de archivos

```ky
from fs import file

# Escritura
f = file.open("/tmp/test.txt", "w")
f.write("hello world")
f.close()

# Lectura
f = file.open("/tmp/test.txt", "r")
content = f.read()
f.close()
println(content)
```

### Modos de apertura

| Modo | Descripción |
|------|-------------|
| `"r"` | Lectura (texto) |
| `"w"` | Escritura (texto, truncar) |
| `"a"` | Append (texto) |
| `"rb"` | Lectura (binario) |
| `"wb"` | Escritura (binario) |

### Métodos

| Método | Descripción |
|--------|-------------|
| `file.open(path, mode)` | Abrir archivo |
| `f.read()` | Leer todo el contenido como string |
| `f.read_bytes(count)` | Leer N bytes |
| `f.write(text)` | Escribir texto |
| `f.write_bytes(bytes)` | Escribir bytes |
| `f.close()` | Cerrar archivo |
| `f.exists()` | `true` si el archivo existe |
| `f.len()` | Tamaño en bytes |

### Ejemplo

```ky
from fs import file

content = file.open("data.txt", "r").read()
lines = content.split("\n")
println("líneas: " + lines.len().to_str())
```
