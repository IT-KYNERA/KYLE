# path — Rutas de Archivos

> Módulo de manipulación de rutas de archivos.
> Import: `from path import path`

## path: manipulación de rutas

```ky
from path import path

p: path = path("/home/user/file.txt")
dir: str = p.dirname()
base: str = p.basename()
ext: str = p.extension()
existe: bool = p.exists()
es_archivo: bool = p.is_file()
es_directorio: bool = p.is_dir()
```

### Métodos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `path(s)` | `fn(s: str) path` | Crear ruta desde string |
| `p.dirname()` | `fn() str` | Directorio padre |
| `p.basename()` | `fn() str` | Nombre del archivo |
| `p.extension()` | `fn() str` | Extensión (incluye punto) |
| `p.exists()` | `fn() bool` | `true` si existe |
| `p.is_file()` | `fn() bool` | `true` si es archivo |
| `p.is_dir()` | `fn() bool` | `true` si es directorio |
| `p.join(other)` | `fn(other: str) path` | Concatenar rutas |
| `p.to_str()` | `fn() str` | String de la ruta |

### Ejemplo

```ky
from path import path

p: path = path("/data")
p = p.join("images").join("photo.jpg")
if p.exists():
    println("archivo: " + p.to_str())
```
