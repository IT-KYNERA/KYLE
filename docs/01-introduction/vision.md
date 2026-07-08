# Vision

Kyle es un lenguaje de programación compilado, estáticamente tipado, diseñado para
sistemas backend, herramientas CLI y desarrollo full-stack con rendimiento nativo,
legibilidad de Python y seguridad de tipos de Rust.

## Objetivos

| Aspecto | Objetivo | Referencia |
|---------|----------|------------|
| Velocidad de compilación | Comparable a Go | ~5s para proyectos medianos |
| Rendimiento runtime | Comparable a C/Rust | Fib: 1.6× C, Matmul: 7.8× C (con listas) |
| Legibilidad | Comparable a Python | Indentación, sin `{}`, sin `;` |
| Seguridad de memoria | Compile-time | Borrow checker, move por defecto |
| Tipado estático | Fuerte e inferido | Sin `null`, sin coerción implícita |

## Audiencia

Kyle está diseñado para:

- **Desarrolladores backend** — APIs, microservicios, CLIs
- **Equipos** que valoran legibilidad tanto como rendimiento
- **Proyectos** que necesitan rendimiento nativo sin complejidad C++
- **Desarrolladores Rust** que quieren menos verbosidad sin sacrificar seguridad
- **Desarrolladores Python/Go** que quieren mejor rendimiento sin cambiar drásticamente

## Filosofía

> "Tan rápido como C, tan legible como Python, tan seguro como Rust."

No es un clon de ningún lenguaje existente. Toma lo mejor de cada uno:
- **Rendimiento** de C (LLVM, native code)
- **Seguridad** de Rust (ownership, borrow checker)
- **Simplicidad** de Go (compilación rápida, tooling integrado)
- **Legibilidad** de Python (indentación, sin ruido sintáctico)

## Ver también

- `philosophy.md` — Filosofía de diseño
- `principles.md` — Principios del lenguaje
- `architecture.md` — Arquitectura del ecosistema
