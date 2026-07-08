# Package Manager

> Gestor de paquetes de Kyle: `ky add`, `ky remove`, `ky install`.

## Comandos

### Agregar dependencia

```bash
ky add http                    # agrega http desde registry
ky add http@1.0.0             # versión específica
```

### Instalar dependencias

```bash
ky install                     # instala todas las dependencias de ky.lock
```

### Eliminar dependencia

```bash
ky remove http                 # elimina http
```

### Publicar paquete

```bash
ky publish                     # publica en registry local
```

## Registry

Los paquetes se distribuyen via GitHub Pages:

```text
https://IT-KYNERA.github.io/KYLE/docs
```

## Paquetes oficiales

| Package | Descripción |
|---------|-------------|
| `http` | Cliente y servidor HTTP |
| `json` | Parseo y serialización JSON |
| `sqlite` | Base de datos SQLite |

## ky.lock

El archivo `ky.lock` se genera automáticamente al instalar dependencias.
No se debe modificar manualmente.

## Ver también

- `build.md` — Compilación
- `project-layout.md` — Estructura de proyectos
- `08-ecosystem/` — Documentación del ecosistema
