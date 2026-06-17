# KL Programming Language — Project Context v2.0

## Overview

KL (Kynera Language) — compiled, statically-typed language combining Python readability (indentation blocks), Rust type safety (strong typing, generics, pattern matching), Go simplicity (fast compilation, built-in tooling), and LLVM performance.

## Glossary — Abreviaciones Técnicas

| Sigla | Inglés | Español / Qué es |
|-------|--------|------------------|
| **AST** | Abstract Syntax Tree | Árbol de Sintaxis Abstracta — representación del código como árbol de nodos tras el parsing |
| **MIR** | Mid-level Intermediate Representation | Representación Intermedia de nivel medio — IR propia entre AST y LLVM |
| **IR** | Intermediate Representation | Representación Intermedia — cualquier representación entre fuente y código máquina |
| **LLVM** | Low Level Virtual Machine | Framework de compilación que genera código máquina optimizado |
| **LSP** | Language Server Protocol | Protocolo de Servidor de Lenguaje — comunicación editor ↔ herramienta de lenguaje |
| **ABI** | Application Binary Interface | Interfaz Binaria — cómo se llaman funciones y se organizan datos en binario |
| **GC** | Garbage Collector | Recolector de Basura — gestión automática de memoria |
| **CFG** | Control Flow Graph | Grafo de Flujo de Control — representación de caminos de ejecución |
| **DCE** | Dead Code Elimination | Eliminación de Código Muerto — optimización que remueve código no ejecutado |
| **FFI** | Foreign Function Interface | Interfaz de Funciones Externas — llamar código C desde KL |
| **CLI** | Command Line Interface | Interfaz de Línea de Comandos — el binario `klc` |
| **LHS/RHS** | Left/Right Hand Side | Lado izquierdo/derecho de una asignación u operación |

## Pipeline del Compilador

```
Source (.kl)               ← tú escribes esto
    ↓
[Lexer]  (klc_frontend)    ← convierte texto → tokens (palabras, números, operadores)
    ↓
[Parser] (klc_frontend)    ← convierte tokens → AST (árbol de sintaxis)
    ↓
[Semantic] (klc_semantic)  ← resuelve símbolos, chequea tipos, valida contracts  ← FASE 2
    ↓
[MIR] (klc_mir)            ← baja AST a IR intermedia, optimiza                   ← FASE 3
    ↓
[Backend] (klc_backend)    ← genera LLVM IR, LLVM optimiza, genera .o             ← FASE 3
    ↓
[Linker] (klc_backend)     ← linkea con runtime + libc → binario nativo           ← FASE 3
```

## Estado de Implementación (Fase 1: Mostly Complete)

### Fully implemented (9 files, ~3000+ lines)
| Archivo | Líneas | Qué hace |
|---------|--------|----------|
| `klc_core/src/ast.rs` | 1076 | Todos los nodos AST (Program, Decl, Stmt, Expr, Pattern, Type) + pretty-print Display |
| `klc_core/src/span.rs` | 38 | Posición y Span (línea, columna) para errores |
| `klc_core/src/types.rs` | 71 | Sistema de tipos semánticos |
| `klc_core/src/source_map.rs` | 61 | Mapa de código fuente con snippet resolution |
| `klc_frontend/src/token.rs` | 147 | ~50 tipos de token (operadores, keywords, delimitadores) |
| `klc_frontend/src/lexer.rs` | 427 | Tokenizador completo con indentación, números hex/bin, strings, chars |
| `klc_frontend/src/parser.rs` | 812 | Parser recursive descent: decls, stmts, expresiones, 12 niveles de precedencia |
| `klc_driver/src/pipeline.rs` | 23 | Orchestrador lexer→parser→output |
| `klc_runtime/src/panic.rs` | 6 | Manejador de pánico |

### Partially implemented (7 files)
| Archivo | Estado | Lo que falta |
|---------|--------|-------------|
| `klc_cli/src/main.rs` | build/run/check/test son stubs, solo parse funciona | Implementar build, run, check, test reales |
| `klc_core/src/diagnostic.rs` | Diagnostic definido | No hay reporter, ni códigos de error, ni span formatting |
| `klc_runtime/src/gc.rs` | init/alloc/collect definidos | Sin integración Boehm GC real |
| `klc_runtime/src/async_.rs` | Executor struct, worker_count | Sin thread pool, sin work-stealing |
| `klc_runtime/src/task.rs` | Task<T> con id | Sin poll, sin Future impl |
| `klc_runtime/src/channel.rs` | Channel<T> con id/capacity | Sin send/recv |
| `klc_runtime/src/error.rs` | KlError struct | Sin Display, sin métodos |

### Stubs (18 files — todo por hacer en fases futuras)
| Crate | Archivos | Se implementa en |
|-------|----------|------------------|
| `klc_semantic/` | type_checker, scope, symbol_table, contracts | **Fase 2** |
| `klc_mir/` | mir, lower, optimize | **Fase 3** |
| `klc_backend/` | codegen, linker | **Fase 3** |
| `klc_tools/` | lsp, formatter, completion | **Fase 5** |
| `klc_driver/` | build (BuildSystem), config | **Fase 3-4** |
| `klc_core/` | symbol (SymbolTable) | **Fase 2** |
| `runtime/` | memory/, async/, collections/, io/ | **Fase 4** |
| `std/` | core/, math/, json/, io/, net/, time/... | **Fase 4** |

## Project Structure

```
kl/
├── AGENTS.md               ← este archivo
├── Cargo.toml              ← workspace Rust raíz
├── .cargo/config.toml      ← config LLVM (Homebrew)
├── kl.toml                 ← manifest KL
│
├── crates/                 ← 9 crates del compilador
│   ├── klc_core/           ← AST, Span, Types, SourceMap, Diagnostics
│   ├── klc_frontend/       ← Lexer + Parser ← ✅ FASE 1 COMPLETA
│   ├── klc_semantic/       ← Type checker, symbol resolver ← ⏳ FASE 2
│   ├── klc_mir/            ← Mid-level IR, optimizaciones ← ⏳ FASE 3
│   ├── klc_backend/        ← LLVM codegen, linker ← ⏳ FASE 3
│   ├── klc_driver/         ← Pipeline orchestration
│   ├── klc_cli/            ← CLI binary (klc)
│   ├── klc_runtime/        ← GC, async, channels, panic handler ← ⏳ FASE 4
│   └── klc_tools/          ← LSP, formatter, completion ← ⏳ FASE 5
│
├── runtime/                ← KL runtime (Rust) ← ⏳ FASE 4
├── std/                    ← Standard library (KL) ← ⏳ FASE 4
├── docs/                   ← 16 specification documents (frozen)
├── examples/               ← Example .kl programs
├── tests/                  ← Test suite ← ⏳ FASE 1 (PENDING)
├── benchmarks/             ← Benchmarks ← ⏳ FASE 4
└── tools/                  ← Developer scripts
```

## LLVM Configuration

Homebrew LLVM 22.1.7 en `/opt/homebrew/opt/llvm`. Ver `.cargo/config.toml`.

```bash
# SIEMPRE usar Homebrew LLVM, NO el de Apple:
/opt/homebrew/opt/llvm/bin/clang --version
/opt/homebrew/opt/llvm/bin/llvm-config --version

# NUNCA:
clang --version           # ← Apple LLVM
/usr/bin/clang            # ← Apple LLVM
```

## Development Commands

```bash
cargo build --workspace                    # Compila todo
cargo run --bin klc -- parse <file.kl>     # Parsear y dump AST (✅ funciona)
cargo run --bin klc -- build <file.kl>     # Compilar (⏳ stub)
cargo run --bin klc -- run   <file.kl>     # Compilar y ejecutar (⏳ stub)
cargo run --bin klc -- check <file.kl>     # Type-check (⏳ stub)
cargo run --bin klc -- help                 # Ayuda (✅ funciona)
cargo test --workspace                      # Tests (⏳ sin tests aún)
```

## Roadmap (desde aquí)

```
FASE 1 (restante) — Terminar Frontend
├── [ ] Escribir tests unitarios del lexer
├── [ ] Escribir tests unitarios del parser
├── [ ] Escribir tests de integración
├── [ ] Mejorar mensajes de error (span + código + sugerencia)
│
FASE 2 — Semantic Analysis (Q4 2026)
├── [ ] Symbol table + scope resolver
├── [ ] Type checker (inferencia Hindley-Milner)
├── [ ] Module resolver (imports)
├── [ ] Generic monomorphization
├── [ ] Contract / error / optional validation
├── [ ] Diagnostics system
│   Hito: kl check main.kl
│
FASE 3 — Compiler Backend (Q1 2027)
├── [ ] Integrar inkwell (LLVM bindings)
├── [ ] MIR: bajar AST → IR + optimizaciones
├── [ ] LLVM IR generation
├── [ ] Linkear con runtime
│   Hito: kl build main.kl → binario nativo
│
FASE 4 — Std Library & Runtime (Q2 2027)
├── [ ] GC real (Boehm)
├── [ ] Async runtime funcional
├── [ ] Standard library completa
│   Hito: kl run hello.kl
│
FASE 5 — Tooling (Q3-Q4 2027)
├── [ ] Package manager
├── [ ] LSP + formatter
├── [ ] VS Code extension
│
FASE 6 — Self-Hosting (2028)
├── [ ] Compilador escrito en KL
│   Hito: kl build klc
```

## Key Design Decisions (frozen)

| Decisión | Elección |
|----------|----------|
| Bloques | Indentación (4 espacios) |
| Punto y coma | Ninguno — newline termina statements |
| Variables | Mutables por defecto (minúsculas) |
| Constantes | UPPERCASE, inmutables, compile-time |
| Referencia a instancia | `this` (no `self`) |
| Opcionales | `Option<T>` (no `T?`) |
| Propagación errores | `?` (exclusivo para errores) |
| Abstracto | `abs class` / `abs fn` |
| Visibilidad | Convención de nombres (`_` protected, `__` private) |
| Excepciones | Ninguna — errores explícitos con `!` y `match` |
| `let`/`var`/`mut` | Ninguno — mutables por defecto |
| `{}` para bloques | Ninguno — indentación |
| Export | Ninguno — visibilidad por naming |
| String encoding | UTF-8 |
| Integer overflow | Panic en debug, wrapping en release |
| Entry point | `fn main(args: [str]) -> i32` en `src/main.kl` |

## Documentación (16 docs, frozen)

| # | Archivo | Contenido |
|---|---------|-----------|
| 00 | `vision.md` | Filosofía y principios de diseño |
| 01 | `language-specification.md` | Sintaxis completa del lenguaje (1566 líneas) |
| 02 | `formal-grammar.md` | Gramática EBNF formal (1076 líneas) |
| 03 | `ast-specification.md` | Definición de nodos AST (905 líneas) |
| 04 | `type-system.md` | Sistema de tipos y reglas (1038 líneas) |
| 05 | `error-system.md` | Manejo de errores (810 líneas) |
| 06 | `module-system.md` | Módulos, packages, imports (918 líneas) |
| 07 | `standard-library.md` | API de la Std Library (948 líneas) |
| 08 | `async-runtime.md` | Async/await y concurrencia (669 líneas) |
| 09 | `memory-model.md` | Memoria y GC (405 líneas) |
| 10 | `compiler-architecture.md` | Pipeline de 9 etapas (296 líneas) |
| 11 | `project-architecture.md` | Estructura del workspace (304 líneas) |
| 12 | `package-manager.md` | Package manager CLI (446 líneas) |
| 13 | `roadmap.md` | Roadmap de 6 fases (438 líneas) |
| 14 | `error-catalog.md` | Catálogo de errores E/W/L (395 líneas) |
| 15 | `abi-specification.md` | ABI y FFI (168 líneas) |
