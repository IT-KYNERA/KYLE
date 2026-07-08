# Principles

## Lenguaje

| Principio | Descripción | Ejemplo |
|-----------|-------------|---------|
| **Sin keywords innecesarios** | No `let`, `var`, `const`, `mut` | `x: i32 = 42` |
| **Tipado fuerte** | Sin coerción implícita | `x = "42" + 1` → error |
| **Move por defecto** | Ownership transfer automático | `y = x` mueve si es no-Copy |
| **Borrow con `&`** | Préstamos explícitos | `f(&x)` presta, no mueve |
| **Mutable con `^`** | Mutabilidad marcada | `x: ^i32 = 0` |
| **Null seguro** | Sin `null`, `T?` para opcionales | `name: str? = none` |
| **Errores como valores** | Sin excepciones, `T!` para fallos | `fn f() i32!` |
| **Sin herencia múltiple** | Solo herencia simple (`::`) | `class Dog :: Animal` |
| **Sin sobrecarga de operadores** | Solo `op_+`, `op_*` etc. | `fn op_+(other: T) T` |

## Compilador

| Principio | Descripción |
|-----------|-------------|
| **Single pass** | Lexer → Parser → HIR → Semantic → MIR → Codegen |
| **SSA opcional** | Deshabilitado por defecto (bugs conocidos) |
| **LLVM O3** | Optimización agresiva en release |
| **LTO** | Link-Time Optimization en release |
| **Sin runtime pesado** | `libkyc_runtime.a` ~3MB |

## Tooling

| Herramienta | Propósito |
|-------------|-----------|
| `ky build` | Compilar a binario |
| `ky run` | Compilar y ejecutar |
| `ky check` | Type-check rápido |
| `ky test` | Ejecutar tests |
| `ky fmt` | Formatear código |
| `ky lsp` | Language server |
| `ky add` / `ky install` | Gestión de paquetes |

## Paquetes

| Tipo | Ejemplos |
|------|----------|
| **Nativos** (built-in) | `str`, `{T}`, `date_time`, `regex`, `json` |
| **Packages** (externos) | `http`, `sqlite`, `postgres` |

## Ver también

- `philosophy.md` — Filosofía del lenguaje
- `architecture.md` — Arquitectura del ecosistema
