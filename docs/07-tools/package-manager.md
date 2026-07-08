# Package Manager (Technical Reference)

> Documentación técnica del gestor de paquetes.
> Para uso básico, ver `02-getting-started/package-manager.md`.

## Comandos

| Comando | Descripción |
|---------|-------------|
| `ky add <pkg>` | Agregar dependencia |
| `ky remove <pkg>` | Eliminar dependencia |
| `ky install` | Instalar dependencias del proyecto |
| `ky publish` | Publicar paquete en registry |

## Formato del paquete

```
package-name/
├── ky.toml          # metadata del paquete
└── src/
    └── lib.ky       # código fuente
```

## Registry URL

```
https://IT-KYNERA.github.io/KYLE/docs
```

## Ver también

- `02-getting-started/package-manager.md` — Guía de uso
- `08-ecosystem/registry.md` — Especificación del registry
