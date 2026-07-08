# Build

> Compilar proyectos Kyle con el CLI `ky`.

## Compilar un archivo

```bash
ky build archivo.ky              # compilar (debug)
ky build --release archivo.ky   # compilar con optimizaciones
ky build archivo.ky -o salida   # especificar nombre de salida
```

## Compilar y ejecutar

```bash
ky run archivo.ky               # compila y ejecuta
ky run --release archivo.ky     # compila optimizado y ejecuta
```

## Type-check rápido

```bash
ky check archivo.ky              # solo type checking, sin código objeto
```

## Proyectos

```bash
ky build                         # compilar proyecto actual
ky build --release               # compilar en release
ky check                         # type-check del proyecto
```

## Flags

| Flag | Descripción |
|------|-------------|
| `--release` | Compilación optimizada (O3 + LTO) |
| `-o` | Nombre del binario de salida |
| `--target` | Target triple (ej: `wasm32`) |

## Ver también

- `installation.md` — Instalar Kyle
- `first-program.md` — Primer programa
- `06-compiler/pipeline.md` — Pipeline de compilación
