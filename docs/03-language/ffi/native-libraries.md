# Native Libraries

> Linkear y usar bibliotecas nativas del sistema desde Kyle.

## Linker flags con `@link`

```ky
@link "curl"               # -lcurl
@link "pthread"            # -lpthread
@link "m"                  # -lm (math)
@link "sqlite3"            # -lsqlite3
@link "pq"                 # -lpq (PostgreSQL)
```

### Frameworks (macOS)

```ky
@link "-framework CoreFoundation"
@link "-framework Security"
@link "-framework SystemConfiguration"
```

### Library paths

```ky
@link "-L/usr/local/lib"
@link "-L/opt/homebrew/lib"
```

## Path de búsqueda

El linker busca bibliotecas en:

1. `-L` paths especificados en `@link`
2. Variables de entorno (`LIBRARY_PATH`, `LD_LIBRARY_PATH`)
3. Paths del sistema (`/usr/lib`, `/usr/local/lib`, `/opt/homebrew/lib`)

## Compilación estática vs dinámica

| Tipo | Ventaja | Desventaja |
|------|---------|-------------|
| Estática (`.a`) | Binario portable | Tamaño grande |
| Dinámica (`.so`/`.dylib`) | Tamaño pequeño | Requiere runtime library |

Por defecto Kyle linkea dinámicamente. Para estático, usa `@link "lib.a"`.

## Ver también

- `c.md` — Llamar funciones C
- `abi.md` — ABI y calling convention
