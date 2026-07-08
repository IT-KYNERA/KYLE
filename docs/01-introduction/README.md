# 01-introduction

> Introducción al lenguaje de programación Kyle.

## Archivos

| Documento | Descripción |
|-----------|-------------|
| `vision.md` | Visión, objetivos y audiencia |
| `philosophy.md` | Filosofía: Python readability, Rust safety, Go simplicity, C perf |
| `principles.md` | Principios de diseño del lenguaje y tooling |
| `architecture.md` | Arquitectura del ecosistema por capas |
| `roadmap.md` | Roadmap de desarrollo y próximos pasos |
| `faq.md` | Preguntas frecuentes |

## Resumen

Kyle es un lenguaje de **bajo nivel con sintaxis de alto nivel**:
- Compilado vía LLVM 18
- Tipado estático fuerte con inferencia
- Ownership y borrow checker (v0.6)
- Move por defecto, `^` para mutable, `&` para borrow
- Sin GC, sin runtime overhead
- Sin `let`/`var`/`mut`/`const`
- Sin `null`, sin excepciones
- snake_case para todo
- Paquetes solo para HTTP/SQLite/Postgres — el resto es nativo
