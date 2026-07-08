# Kyle Documentation

> Documentación oficial del lenguaje de programación Kyle.

## Estructura

| Sección | Descripción |
|---------|-------------|
| [01-introduction](01-introduction/) | Visión, filosofía, principios, arquitectura |
| [02-getting-started](02-getting-started/) | Instalación, primeros pasos, testing, debugging |
| [03-language](03-language/) | Referencia completa del lenguaje |
| [04-standard-library](04-standard-library/) | API de la librería estándar |
| [05-runtime](05-runtime/) | Comportamiento interno del runtime |
| [06-compiler](06-compiler/) | Internals del compilador |
| [07-tools](07-tools/) | Herramientas oficiales |
| [08-ecosystem](08-ecosystem/) | Registry, packages, publicación |
| [09-specification](09-specification/) | Especificación formal |
| [10-design](10-design/) | Decisiones de diseño |
| [11-project](11-project/) | Contribución, estilo, releases |
| [12-history](12-history/) | Changelog, migraciones |

## Convenciones

- `^T` = mutable type, `&T` = borrow, `^&T` = mutable borrow
- `T?` = optional type, `T!` = fallible type
- snake_case para funciones y tipos
- `[x]` = implementado, `[ ]` = diseñado no implementado

## Recursos rápidos

| Necesitas | Ir a |
|-----------|------|
| Sintaxis del lenguaje | `03-language/syntax/` |
| Tipos | `03-language/types/` |
| Ownership y memoria | `03-language/memory/` |
| Async y concurrencia | `03-language/concurrency/` |
| Errores | `03-language/error-handling/` |
| API estándar | `04-standard-library/` |
| Compilador (internals) | `06-compiler/` |

Ver también: [TYPES.md](../TYPES.md), [ROADMAP.md](../ROADMAP.md), [AGENTS.md](../AGENTS.md).
