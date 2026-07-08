# path — Rutas de Archivos

> Módulo de manipulación de rutas de archivos.
> Import: `from path import path`

## path: manipulación de rutas

```ky
from path import path

p = path("/home/user/file.txt")
println(p.dirname())      # "/home/user"
println(p.basename())     # "file.txt"
println(p.extension())    # ".txt"
println(p.exists())       # true
println(p.is_file())      # true
println(p.is_dir())       # false
```

### Métodos

| Método | Descripción |
|--------|-------------|
| `path(s)` | Crear ruta desde string |
| `p.dirname()` | Directorio padre |
| `p.basename()` | Nombre del archivo |
| `p.extension()` | Extensión (incluye punto) |
| `p.exists()` | `true` si existe |
| `p.is_file()` | `true` si es archivo |
| `p.is_dir()` | `true` si es directorio |
| `p.join(other)` | Concatenar rutas |
| `p.to_str()` | String de la ruta |

### Ejemplo

```ky
from path import path

p = path("/data")
p = p.join("images")
p = p.join("photo.jpg")
if p.exists():
    println("archivo: " + p.to_str())
```
