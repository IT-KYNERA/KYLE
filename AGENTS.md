# Kyle Programming Language — Project Context v4.0

## Overview

Kyle — compiled, statically-typed language combining Python readability (indentation blocks), Rust type safety (strong typing, generics, pattern matching), Go simplicity (fast compilation, built-in tooling), and LLVM performance.

## State — Resumen Ejecutivo

```
Pipeline completo:      Lexer → Parser → Semantic → MIR → Backend → Linker ✅
Runtime + Std Library:  RAII, async, file I/O, string ops, char ops, threads ✅
Package Manager:        manifest, lock, add, remove, info, build, run, test ✅
LSP:                    document symbols, workspace symbols, signature help,
                        find references, code actions ✅
Formatter:              pretty-printer + comment preservation ✅
VS Code:                extension with syntax highlighting, LSP client, commands ✅
Struct ABI:             pass-by-reference (pointer-based) ✅
Phase 3.5 Complete:     closures, methods, enums/match, async/await ✅
Phase 6 P1:             Ternary ✅, Match-expression ✅, Guard ✅, Defer ✅
Self-Hosting (Phase 8): lexer.kl + parser.kl + semantic.kl (deferred post-MVP) ✅
Tests:                  86 tests, 0 failures ✅
```

## Session Log

### Sesión 1 — Fase 4: Runtime string ops + File I/O + Time
| Feature | Archivos | Estado |
|---------|----------|--------|
| Runtime string ops | `klc_runtime/src/string.rs` | ✅ `kl_str_contains`, `to_upper`, `to_lower`, `trim`, `replace`, `concat`, `input` |
| Compiler string op support | `codegen.rs`, `lower.rs`, `symbol_table.rs` | ✅ extern decls, name remapping, builtins |
| `str()` builtin | `lower.rs` | ✅ Cast i32→i64 antes de `kl_i64_to_str` |
| `len()` builtin | `lower.rs` | ✅ retorna I32 |
| Variable type inference | `lower.rs` | ✅ `Expr::Assignment` usa `local_types` map |
| `kl_print`/`kl_println`/etc. | `codegen.rs`, runtime | ✅ len params cambiados a i32 |
| `kl_now()` fix | `klc_runtime/src/io.rs` | ✅ clock_gettime → `SystemTime::now()` (aarch64) |
| File I/O runtime | `klc_runtime/src/io.rs` | ✅ `open`, `read_str`, `write_str`, `close` |
| Time runtime | `klc_runtime/src/io.rs` | ✅ `sleep(ms)`, `now() -> i64` |
| `std/testing.kl` | `std/testing.kl` | ✅ `assert`, `assert_eq`, `assert_str` |
| String test | `string_test.kl` | ✅ Verificado con `kl run` |

### Sesión 2 — Fase 5: Package Manager
| Feature | Archivos | Estado |
|---------|----------|--------|
| Manifest struct | `klc_tools/src/package/manifest.rs` | ✅ serde + read/write |
| Lock file | `klc_tools/src/package/lock.rs` | ✅ serde + read/write |
| Project helper | `klc_tools/src/package/project.rs` | ✅ `find_project_root()`, source paths |
| CLI: add/remove/info | `klc_cli/src/main.rs` | ✅ `kl add dep@ver`, `kl remove dep`, `kl info` |
| CLI: build/run/test (project) | `klc_cli/src/main.rs` | ✅ busca kl.toml, compila src/main.kl |
| CLI: new | `klc_cli/src/main.rs` | ✅ crea src/ + tests/ |
| CLI: init | `klc_cli/src/main.rs` | ✅ alias de new |

### Sesión 3 — Fase 5: LSP improvements
| Feature | Archivos | Estado |
|---------|----------|--------|
| documentSymbol | `klc_tools/src/lsp.rs` | ✅ SymbolInformation flat |
| workspace/symbol | `klc_tools/src/lsp.rs` | ✅ cross-document query |
| signatureHelp | `klc_tools/src/lsp.rs` | ✅ function signature display |

### Sesión 4 — Fase 5: Formatter comment preservation + Span fixes
| Feature | Archivos | Estado |
|---------|----------|--------|
| Lexer token spans | `klc_frontend/src/lexer.rs` | ✅ `make_token()` usa Position real |
| Parser AST spans | `klc_frontend/src/parser.rs` | ✅ 60+ nodos propagan spans desde tokens |
| Formatter comments | `klc_tools/src/formatter.rs` | ✅ `#` antes de decls/stmts via `last_comment_line` |
| fmt CLI command | `klc_cli/src/main.rs` | ✅ `kl fmt <file.kl>` |

### Sesión 5 — Fase 5: LSP findReferences + codeActions + VS Code extension
| Feature | Archivos | Estado |
|---------|----------|--------|
| LSP findReferences | `klc_tools/src/lsp.rs` | ✅ `handle_references` + `find_references_in_source` |
| LSP code actions | `klc_tools/src/lsp.rs` | ✅ `handle_code_action` (E0009 → create var / import) |
| Server capabilities | `klc_tools/src/lsp.rs` | ✅ references_provider + code_action_provider |
| VS Code extension manifest | `vscode-kl/package.json` | ✅ language activation, commands, grammar |
| Syntax highlighting | `vscode-kl/syntaxes/kl.tmLanguage.json` | ✅ keywords, types, builtins, strings, numbers, operators |
| Language config | `vscode-kl/language-configuration.json` | ✅ comments, brackets, auto-closing, indentation |
| LSP client | `vscode-kl/src/extension.ts` | ✅ launches `klc lsp`, commands `kl.run/build/check` |
| CLI lsp command | `klc_cli/src/main.rs` | ✅ `klc lsp` (ya existía) |

### Sesión 6 — Fase 6: Self-Hosting infraestructura (char ops, fixes, lexer.kl)
| Feature | Archivos | Estado |
|---------|----------|--------|
| Runtime char ops | `klc_runtime/src/string.rs` | ✅ `kl_char_at`, `kl_is_digit`, `kl_is_alpha`, `kl_is_alnum`, `kl_is_whitespace`, `kl_is_upper`, `kl_is_lower` |
| Runtime `ord()` | `klc_runtime/src/string.rs` | ✅ `kl_ord(i8) -> i32` |
| Compiler char builtins | `symbol_table.rs`, `lower.rs`, `codegen.rs` | ✅ extern decls, name remapping, return types |
| Fix: hardcoded `if_then` block name | `lower.rs` | ✅ `Stmt::If` usa `ctx.fresh_block()` en vez de `"if_then"` |
| Fix: elif chain block collision | `lower.rs` | ✅ cada elif usa su propio nombre de bloque (`elif_cond_labels[i]`) |
| Fix: string escape sequences | `klc_frontend/src/lexer.rs` | ✅ `lex_string()` procesa `\n`, `\t`, `\"`, etc. |
| Fix: string return from user fn | `lower.rs` | ✅ `fn_returns` map + `MirType::Str` en calls |
| Fix: string concat result type | `lower.rs` | ✅ `MirType::I64` → `MirType::Str` para que `string_locals` funcione |
| Fix: `Stmt::Break` lowering | `lower.rs` | ✅ `Unreachable` → `Br(loop_end)` via `break_targets` stack |
| Lexer escrito en Kyle | `examples/lexer.kl` | ✅ tokeniza `x = 1 + 2\n` → 7 tokens correctos |
| Tests | - | ✅ 118 tests, 0 failures |

### Sesión 7 — Documentación: docs sync con estado real del compilador
| Feature | Archivos | Estado |
|---------|----------|--------|
| Roadmap actualizado | `docs/13-roadmap.md` | ✅ Fase 4 ✓, Fase 5 ✓, Fase 6 en progreso |
| Language spec v2.0 | `docs/01-language-specification.md` | ✅ string escapes, char literals, builtins, break |
| Std library spec v2.0 | `docs/07-standard-library.md` | ✅ builtins reales, testing API, time top-level |
| Formal grammar | `docs/02-formal-grammar.md` | ✅ char_literal, escape_sequence, character production |
| Compiler architecture | `docs/10-compiler-architecture.md` | ✅ MIR pipeline real, klc_tools, ownership pass |
| Error catalog | `docs/14-error-catalog.md` | ✅ repo URL fixed, lint rules marked 🔶 |

### Sesión 8 — Fase 6: Char comparison fix, RAII alloc fix, string lists, lexer file I/O
| Feature | Archivos | Estado |
|---------|----------|--------|
| `Type::Char` → `is_numeric()` + `can_assign_to()` | `klc_core/src/types.rs` | ✅ char se trata como numérico para `+`, `==`, `<`, etc. |
| Type checker Eq/Neq diagnostic | `klc_semantic/src/type_checker.rs` | ✅ reporta error si unificación falla |
| Lowering: Cast antes de BinaryOp | `klc_mir/src/lower.rs` | ✅ inserta Cast si operandos tienen distinto ancho |
| Runtime: `kl_read_str` usa `kl_alloc` | `klc_runtime/src/io.rs` | ✅ RAII cleanup no crashea |
| Runtime: string ops usan `kl_alloc` | `klc_runtime/src/string.rs` | ✅ concat, upper, lower, trim, replace, substr |
| Runtime: `kl_input` usa `kl_alloc` | `klc_runtime/src/io.rs` | ✅ RAII cleanup no crashea |
| Codegen: Cast ptr↔int via ptrtoint/inttoptr | `klc_backend/src/codegen.rs` | ✅ string lists funcionan |
| Lowering: `substr` special case con cast i64 | `klc_mir/src/lower.rs` | ✅ args pasados como i64, resultado en string_locals |
| Lowering: `Expr::Index` detecta `List(Str)` | `klc_mir/src/lower.rs` | ✅ retorna Str con inttoptr |
| Lowering: `Expr::List` inferencia de tipo | `klc_mir/src/lower.rs` | ✅ `["a", "b"]` → `List(Str)`, `[1, 2]` → `List(I32)` |
| Lexer real (file I/O) | `examples/lexer.kl` | ✅ lee `examples/hello.kl`, tokeniza con posición |
| Tests | - | ✅ 118 tests, 0 failures |

### Sesión 9 — Fase 6: SSA dominance fix para kl_release (crash en cleanup)
| Feature | Archivos | Estado |
|---------|----------|--------|
| Root cause: SSA dominance violation | `klc_backend/src/codegen.rs`, `klc_mir/src/ownership.rs` | ✅ `last_value_map` almacenaba SSA values de `kl_concat` en basic block del loop body, usados por `kl_release` en basic block no-dominante (return). LLVM generaba código con punteros basura (stack garbage, kernel addresses). |
| Fix codegen: Store call results a alloca | `klc_backend/src/codegen.rs` | ✅ `build_store` después de `build_call` para que el alloca tenga el valor correcto para cross-block reads |
| Fix ownership: Load+Call para kl_release | `klc_mir/src/ownership.rs` | ✅ en vez de `kl_release(MirValue::Local(id))` (usa last_value_map), emite `Load { dest: tmp, src: id }` + `Call kl_release(tmp)` para leer el alloca directamente |
| Lexer sin crash | `examples/lexer.kl` | ✅ cleanup sin errores (15 frees exitosos, 0 punteros corruptos) |
| Tests | - | ✅ 118 tests, 0 failures |

### Sesión 10 — Fase 6: Parser en Kyle + Fix parse_block/parse_if (blank lines entre elif)
| Feature | Archivos | Estado |
|---------|----------|--------|
| Fix: `parse_block` single-line body | `klc_frontend/src/parser.rs` | ✅ al detectar que no hay Newline inicial (`single_line`), para tras 1 statement y consume trailing Newlines |
| Fix: `parse_if` trailing Newlines | `klc_frontend/src/parser.rs` | ✅ consume Newlines entre if-body y elif (soporta blank lines) |
| Parser en Kyle | `examples/parser.kl` | ✅ 1511 líneas, AST recursivo con `AstNode`, pasa Rust frontend test |
| Tests | - | ✅ 84 tests, 0 failures |

### Sesión 11 — Fase 6: RAII per-block release fix + lexer.kl funcionando + kl_list_pop + auto-declare fix
| Feature | Archivos | Estado |
|---------|----------|--------|
| Fix: RAII per-block release temp IDs | `klc_mir/src/ownership.rs` | ✅ temp IDs únicos con orden inverso para inserts correctos |
| Fix: Store handler (eliminado inttoptr logic) | `klc_backend/src/codegen.rs` | ✅ revertido cambio innecesario de Sesión 9 |
| Fix: lexer stale `c` después de newline_pending | `examples/lexer.kl` | ✅ re-read `c` + `start_col` tras procesar indentación |
| Runtime: `kl_list_pop` | `klc_runtime/src/list.rs` | ✅ `i64 kl_list_pop(ptr)` — decrementa len, retorna valor |
| Lowering: `pop()` method call | `klc_mir/src/lower.rs` | ✅ `list.pop()` → `kl_list_pop(list)` (análogo a `add`) |
| Codegen: `kl_list_pop` decl | `klc_backend/src/codegen.rs` | ✅ `i64 kl_list_pop(ptr)` extern declaration |
| Lexer en Kyle | `examples/lexer.kl` | ✅ tokeniza `examples/hello.kl` correctamente con INDENT/DEDENT |
| Fix: type checker auto-declare `ident = expr` | `klc_semantic/src/type_checker.rs` | ✅ `check_stmt` intercepta `Stmt::Expression(Expr::Assignment)` con destino `Identifier`, infiere el tipo del valor y registra la variable en el scope actual |
| Root cause | scope.rs / type_checker.rs | ✅ Scope resolver auto-declara variables dentro del scope de la función, pero `resolve_function` hace `pop_scope()` al terminar, eliminando las variables. El type checker arranca con un símbol table vacío (sin variables auto-declaradas). |
| Verificación | `examples/fibonacci.kl` | ✅ `klc run examples/fibonacci.kl` → `fibonacci(10) = 55` |
| Tests | - | ✅ 84 tests, 0 failures |

## Glossary — Abreviaciones Técnicas

| Sigla | Inglés | Español / Qué es |
|-------|--------|------------------|
| **AST** | Abstract Syntax Tree | Árbol de Sintaxis Abstracta — representación del código como árbol de nodos tras el parsing |
| **MIR** | Mid-level Intermediate Representation | Representación Intermedia de nivel medio — IR propia entre AST y LLVM |
| **IR** | Intermediate Representation | Representación Intermedia — cualquier representación entre fuente y código máquina |
| **LLVM** | Low Level Virtual Machine | Framework de compilación que genera código máquina optimizado |
| **LSP** | Language Server Protocol | Protocolo de Servidor de Lenguaje — comunicación editor ↔ herramienta de lenguaje |
| **ABI** | Application Binary Interface | Interfaz Binaria — cómo se llaman funciones y se organizan datos en binario |
| **RAII** | Resource Acquisition Is Initialization | Adquisición de Recursos es Inicialización — memoria se libera al salir del scope |
| **RC** | Reference Counting | Conteo de Referencias — gestión automática de memoria compartida |
| **CFG** | Control Flow Graph | Grafo de Flujo de Control — representación de caminos de ejecución |
| **DCE** | Dead Code Elimination | Eliminación de Código Muerto — optimización que remueve código no ejecutado |
| **FFI** | Foreign Function Interface | Interfaz de Funciones Externas — llamar código C desde Kyle |
| **CLI** | Command Line Interface | Interfaz de Línea de Comandos — el binario `klc` |
| **LHS/RHS** | Left/Right Hand Side | Lado izquierdo/derecho de una asignación u operación |

## Pipeline del Compilador

```
Source (.kl)               ← tú escribes esto
    ↓
[Lexer]  (klc_frontend)    ← convierte texto → tokens (palabras, números, operadores)  ✅
    ↓
[Parser] (klc_frontend)    ← convierte tokens → AST (árbol de sintaxis)                 ✅
    ↓
[Semantic] (klc_semantic)  ← resuelve símbolos, chequea tipos, valida contracts         ✅
    ↓
[MIR] (klc_mir)            ← baja AST a IR intermedia, optimiza                         ✅
    ↓
[Backend] (klc_backend)    ← genera LLVM IR, LLVM optimiza, genera .o                   ✅
    ↓
[Linker] (klc_backend)     ← linkea con libc → binario nativo                           ✅
```

## Estado de Implementación — Archivos Activos

| Crate | Archivo | Líneas | Estado |
|-------|---------|--------|--------|
| `klc_core` | `ast.rs` | 1076 | ✅ completo |
| `klc_core` | `span.rs` | 38 | ✅ completo |
| `klc_core` | `types.rs` | 71 | ✅ completo |
| `klc_core` | `source_map.rs` | 61 | ✅ completo |
| `klc_core` | `diagnostic.rs` | ~200 | 🔶 le faltan códigos de error |
| `klc_frontend` | `token.rs` | 147 | ✅ completo |
| `klc_frontend` | `lexer.rs` | 809 | ✅ span tracking, Position real, escape strings |
| `klc_frontend` | `parser.rs` | 812 → ~1353 | ✅ span tracking agregado (60+ nodos) |
| `klc_semantic` | `type_checker.rs` | 1380 | ✅ 47 tests |
| `klc_semantic` | `symbol_table.rs` | - | ✅ builtins completos |
| `klc_mir` | `mir.rs` | 312 | ✅ completo |
| `klc_mir` | `lower.rs` | 860 | ✅ string ops, type inference, break targets |
| `klc_mir` | `optimize.rs` | 180 | ✅ 2 tests |
| `klc_mir` | `ownership.rs` | - | ✅ RAII inference pass |
| `klc_backend` | `codegen.rs` | 479 | ✅ LLVM 18.1, inkwell |
| `klc_backend` | `linker.rs` | - | 🔶 no linkea runtime library |
| `klc_driver` | `pipeline.rs` | - | 🔶 module resolver no soporta rutas anidadas |
| `klc_cli` | `main.rs` | - | ✅ build/run/parse/check/mir/fmt |
| `klc_runtime` | `string.rs` | - | ✅ contains, to_upper, to_lower, trim, replace, concat, input |
| `klc_runtime` | `io.rs` | - | ✅ open, read_str, write_str, close, sleep, now |
| `klc_runtime` | `async_.rs` | - | ✅ async runtime |
| `klc_runtime` | `task.rs` | - | ✅ tasks |
| `klc_runtime` | `channel.rs` | - | ✅ channels |
| `klc_runtime` | `error.rs` | - | ✅ error handling |
| `klc_runtime` | `gc.rs` | - | ❌ obsoleto (reemplazar por RAII) |
| `klc_tools` | `lsp.rs` | - | ✅ documentSymbol, workspace/symbol, signatureHelp |
| `klc_tools` | `formatter.rs` | - | ✅ pretty-printer + comment preservation |
| `klc_tools` | `package/manifest.rs` | - | ✅ serde + read/write |
| `klc_tools` | `package/lock.rs` | - | ✅ serde + read/write |
| `klc_tools` | `package/project.rs` | - | ✅ find_project_root, source paths |
| `std` | `core.kl` | - | ✅ util functions |
| `std` | `math.kl` | - | ✅ abs, pow, sqrt, gcd |
| `std` | `io.kl` | - | ✅ I/O wrappers |
| `std` | `testing.kl` | - | ✅ assert, assert_eq, assert_str |

## Project Structure

```
kl/
├── AGENTS.md               ← este archivo
├── Cargo.toml              ← workspace Rust raíz
├── .cargo/config.toml      ← config LLVM (Linux)
├── kl.toml                 ← manifest Kyle
│
├── crates/                 ← 9 crates del compilador
│   ├── klc_core/           ← AST, Span, Types, SourceMap, Diagnostics
│   ├── klc_frontend/       ← Lexer + Parser ✅
│   ├── klc_semantic/       ← Type checker, symbol resolver ✅
│   ├── klc_mir/            ← MIR definition, lowering, optimizations ✅
│   ├── klc_backend/        ← LLVM codegen (inkwell), linker ✅
│   ├── klc_driver/         ← Pipeline orchestration
│   ├── klc_cli/            ← CLI binary (klc)
│   ├── klc_runtime/        ← RAII runtime, async, channels, panic handler ⏳
│   └── klc_tools/          ← LSP, formatter, completion ⏳
│
├── runtime/                ← Kyle runtime (Rust) ⏳
├── std/                    ← Standard library (Kyle) ⏳
├── docs/                   ← 16 specification documents (mantener al día)
├── examples/               ← Example .kl programs
├── tests/                  ← Test suite ⏳
├── benchmarks/             ← Benchmarks ⏳
└── tools/                  ← Developer scripts
```

## LLVM Configuration

LLVM 18.1.3 via apt (`llvm-18-dev` + `libpolly-18-dev` + `libzstd-dev`). Ver `.cargo/config.toml`.

```bash
# Sistema (Linux aarch64):
/usr/bin/llvm-config --version   # → 18.1.3
```

## Development Commands

```bash
cargo build --workspace                    # Compila todo
cargo run --bin klc -- parse <file.kl>     # Parsear y dump AST ✅
cargo run --bin klc -- build <file.kl>     # Compilar a binario nativo ✅
cargo run --bin klc -- run   <file.kl>     # Compilar y ejecutar ✅
cargo run --bin klc -- check <file.kl>     # Type-check ✅
cargo run --bin klc -- mir   <file.kl>     # Parsear y dump MIR ✅
cargo run --bin klc -- fmt   <file.kl>     # Formatear código ✅
cargo run --bin klc -- help                 # Ayuda ✅
cargo test -p klc_core -p klc_frontend -p klc_semantic -p klc_mir -p klc_runtime -p klc_tools  # 86 tests, 0 failures ✅
```

## Roadmap (v5.0 — MVP Focus)

```
FASE 1 + 2 + 3 + 3.5 + 4 + 5 (complete) — Full pipeline end-to-end ✅

FASE 6 — Language Completion (🔶 Current — prioridades P0-P5)
│   Completar TODA la sintaxis: que todo genere código funcionando.
│
├─ 🟥 P0 (ALTA) — End-to-end language features (4/5 done)
│   For loops ✅, Generics structs ✅, Generics functions ✅,
│   Error handling (!/?) ✅, String interpolation ✅, Optional chaining (?.) ❌
│
├─ 🟧 P1 (ALTA) — Secondary features
│   Defer ✅, Guard ✅, Type aliases ✅, Dict/Map, Spread ✅, Range slicing ✅,
│   Ternary ✅, Match-expression ✅, const fn
│
├─ 🟦 P3 (MEDIA) — Standard library
│   collections, str ops, time, json
│
├─ 🟪 P4 (BAJA) — Tooling polish
│   LSP completion/hover/goto-def, debug info, optimization levels
│
└─ 🟩 P5 (BAJA) — Robustness & testing
    LLVM verification, error messages, 100+ tests, CI pipeline

│   Hito: kl run any_project.kl → works reliably

FASE 7 — Cross-Platform Support (⏸️ Next)
│   Portar a Windows (x64), Linux (x64+ARM), macOS (Intel+ARM).
│   5 cambios localizados, ~1-2 días de trabajo.
│   Hito: klc build + klc run en las 3 plataformas

FASE 8 — Self-Hosting (⏸️ Deferred)
│   Hito: kl build klc

FASE 9 — Production Ecosystem (📅 Future)

### Sesión 19 — Roadmap restructure: MVP focus, self-hosting deferred to Phase 7
| Feature | Archivos | Estado |
|---------|----------|--------|
| Roadmap rewritten | `docs/13-roadmap.md` | ✅ Phase 6 → MVP Completion, Phase 7 → Self-Hosting (deferred) |
| Status doc rewritten | `docs/16-status.md` | ✅ Accurate gap analysis for MVP, explicit Phase 6 priorities |
| Vision doc updated | `docs/00-vision.md` | ✅ "Current phase: Phase 6 — MVP Completion" |
| Language spec updated | `docs/01-language-specification.md` | ✅ Roadmap section replaced with new phase table |
| Std library doc updated | `docs/07-standard-library.md` | ✅ "Phase 6/7" → "Phase 6 (MVP Completion)" |
| AGENTS.md updated | `AGENTS.md` | ✅ New roadmap, session log entry, v4.0 |
| Self-hosting decision | — | ✅ **Self-hosting deferred to post-MVP.** Rewriting compiler in Kyle happens only after the language is stable and usable for real projects. Rust stays as implementation language. |

### Sesión 19 — Cross-platform analysis + syntax reference + roadmap restructure
| Feature | Archivos | Estado |
|---------|----------|--------|
| Syntax reference (español) | `docs/17-syntax-reference.md` | ✅ 38 secciones, status marks (✅/🔶/❌/📄) |
| Cross-platform audit | `linker.rs`, `pipeline.rs`, `io.rs`, `cli` | 🔶 Solo macOS ARM — ver plan P2 |
| Roadmap actualizado con P0-P5 | `docs/13-roadmap.md` | ✅ Prioridades reales por estado de implementación |
| AGENTS.md actualizado | `AGENTS.md` | ✅ Nuevo roadmap, docs table, hallazgos cross-platform |

### Sesión 20 — Reestructuración de fases: Language Completion → Cross-Platform → Self-Hosting
| Feature | Archivos | Estado |
|---------|----------|--------|
| Nueva estructura de fases | `docs/13-roadmap.md` | ✅ Phase 6=Language Completion, Phase 7=Cross-Platform, Phase 8=Self-Hosting, Phase 9=Production |
| Cross-platform como fase separada | `docs/13-roadmap.md` | ✅ Movido de P2 dentro de Phase 6 a Phase 7 independiente |
| Self-hosting movido a Phase 8 | `docs/13-roadmap.md` | ✅ Eliminada toda urgencia de self-hosting |

### Phase 4/5 Bugfixes
- `codegen.rs`: MirValue::Param(id) devolvía 0 siempre → ahora resuelve al parámetro LLVM real
- `lower.rs`: str() pasaba i32 a kl_i64_to_str (espera i64) → añadido Cast i32→i64
- `lower.rs`: Stmt::Variable/TypedVariable no emitían Store para literales → arreglado
- `optimize.rs`: DCE eliminaba Store si solo Return lo usaba → añadido collect_terminator_refs
- `symbol_table.rs`: println faltaba de builtins → añadido
- `lexer.rs`: make_token() usaba Span::dummy() → ahora usa posición real (line, column, offset)
- `parser.rs`: todos los AST nodos usaban Span::dummy() → ahora propagan spans desde tokens
- `formatter.rs`: conservación de comentarios usando last_comment_line tracking + source_lines
- `parser.rs`: `parse_block` ahora para tras 1 statement si no hay Newline (single-line bodies)
- `parser.rs`: `parse_if` consume Newlines entre if-body y elif/else (blank lines entre branches)
- `type_checker.rs`: auto-declara `ident = expr` con el tipo inferido del valor (para que `str(result)` funcione)
- `type_checker.rs`: auto-declara `ident = expr` con el tipo inferido del valor (para que `str(result)` funcione)

## Key Design Decisions (frozen)

| Decisión | Elección |
|----------|----------|
| Bloques | Indentación (4 espacios) |
| Punto y coma | Ninguno — newline termina statements |
| Variables | Inmutables por defecto, `mut` para mutable (lowercase) |
| Constantes | UPPERCASE, siempre inmutables, compile-time, sin `mut` |
| Referencia a instancia | `this` (no `self`) |
| Opcionales | `Option<T>` (no `T?`) |
| Propagación errores | `?` (exclusivo para errores) |
| Abstracto | `abs class` / `abs fn` |
| Visibilidad | Convención de nombres (`_` protected, `__` private) |
| Excepciones | Ninguna — errores explícitos con `!` y `match` |
| `let`/`var` | Ninguno — `mut` keyword directo |
| `{}` para bloques | Ninguno — indentación |
| Export | Ninguno — visibilidad por naming |
| String encoding | UTF-8 |
| Integer overflow | Panic en debug, wrapping en release |
| Entry point | `fn main(args: [str]) -> i32` en `src/main.kl` |

## Documentación (17 docs)

| # | Archivo | Contenido |
|---|---------|-----------|
| 00 | `vision.md` | Filosofía y principios de diseño |
| 01 | `language-specification.md` | Sintaxis completa del lenguaje (~1650 líneas) |
| 02 | `formal-grammar.md` | Gramática EBNF formal (~1085 líneas) |
| 03 | `ast-specification.md` | Definición de nodos AST (905 líneas) |
| 04 | `type-system.md` | Sistema de tipos y reglas (1038 líneas) |
| 05 | `error-system.md` | Manejo de errores (810 líneas) |
| 06 | `module-system.md` | Módulos, packages, imports (918 líneas) |
| 07 | `standard-library.md` | API de la Std Library (948 líneas) |
| 08 | `async-runtime.md` | Async/await y concurrencia (669 líneas) |
| 09 | `memory-model.md` | Memoria RAII + Compiler-Inferred Ownership |
| 10 | `compiler-architecture.md` | Pipeline de 9 etapas (296 líneas) |
| 11 | `project-architecture.md` | Estructura del workspace (304 líneas) |
| 12 | `package-manager.md` | Package manager CLI (446 líneas) |
| 13 | `roadmap.md` | Roadmap de 9 fases (MVP focus) |
| 14 | `error-catalog.md` | Catálogo de errores E/W/L (395 líneas) |
| 15 | `abi-specification.md` | ABI y FFI (168 líneas) |
| 16 | `status.md` | **Estado verificado del implementation gap** (fuente de verdad) |
| 17 | `syntax-reference.md` | **Sintaxis completa en español con marcas de estado** |

## Session Log (append)

### Sesión 12 — Phase 3.5: StructLiteral + Method Dispatch + Ownership Fixes
| Feature | Archivos | Estado |
|---------|----------|--------|
| `parse_params` optional types | `parser.rs` | ✅ `name` or `name: Type` supported |
| `[T]` list type syntax | `parser.rs` | ✅ `[str]` parsed as `List<T>` |
| `Expr::StructLiteral` AST node | `ast.rs`, `parser.rs` | ✅ `Counter { field: value }` syntax |
| StructLiteral lowering | `lower.rs` | ✅ FieldPtr+Store for struct fields |
| Constructor param binding | `lower.rs` | ✅ params bound to locals |
| Method `this` param dedup | `lower.rs` | ✅ skip explicit `this` param if first |
| Ownership: don't release return values | `ownership.rs` | ✅ concat results used in Return skipped |
| Method dispatch test | `examples/method_test.kl` | ✅ `(4, 6)` from immutable methods |
| parser.kl infinite loop fix | `examples/parser.kl` | ✅ `advance()` on error in expect/expect_keyword/expect_identifier |
| Tests | - | ✅ 84 tests, 0 failures |

### Sesión 14 — Phase 6: Codegen SSA dominance + str() type fix + struct_defs two-pass + parser.kl build exitoso
| Feature | Archivos | Estado |
|---------|----------|--------|
| Fix: SSA dominance violation codegen | `klc_backend/src/codegen.rs` | ✅ `load_value` now prefers alloca over `last_value_map` for cross-block correctness; all dest-producing instructions store to alloca (UnaryOp, Cast) |
| Fix: `str()` result type | `klc_mir/src/lower.rs` | ✅ `alloc_local("_strptr", MirType::I64)` → `MirType::Str` |
| Fix: struct_defs con campos vacíos | `klc_mir/src/lower.rs` | ✅ Two-pass struct definition scan: first register names, then fill fields with full struct_defs map |
| Fix: PropertyAccess lookup fallback | `klc_mir/src/lower.rs` | ✅ When struct type has empty fields, look up real fields from `ctx.struct_defs` |
| parser.kl BUILD + RUN | — | ✅ `klc build examples/parser.kl` → "Build complete" + exit 0 |
| semantic.kl source-level errors | `examples/semantic.kl` | ❌ `peek()` returns char but declares `str`; needs source fix |
| Tests | - | ✅ 84 tests, 0 failures |

### Sesión 15 — Phase 6: Struct pass-by-reference ABI + semantic.kl funcionando completo
| Feature | Archivos | Estado |
|---------|----------|--------|
| Root cause: structs pass-by-value (value semantics) | — | ✅ Descubrimiento fundamental: todos los structs se pasan por valor, `advance(p)` NO modifica el Parser original. Causa raíz de infinite loops en parser.kl y semantic.kl. |
| Fix codegen: struct params como ptr (ABI) | `klc_backend/src/codegen.rs` | ✅ `declare_function` cambia parámetros struct a `ptr` LLVM; `ref_param_struct_types` trackea allocas que almacenan punteros; `FieldPtr` carga ptr del alloca antes de GEP; `Return` dereferencia ref params; `Call` pasa alloca pointer para struct locals |
| Fix lowering: struct args sin Load | `klc_mir/src/lower.rs` | ✅ En call args y method dispatch, detecta `Expr::Identifier` de tipo Struct y usa el local original (sin emitir `Load` que copia) |
| Fix CLI: forward args a binary | `klc_cli/src/main.rs` | ✅ `cmd_run` pasa `args[3..]` al binario compilado (proyecto y file mode) |
| semantic.kl funcional | `examples/semantic.kl` | ✅ Tokeniza, parsea, y type-checks `fibonacci.kl` → "parsed", "checked", "ok", exit 0 |
| parser.kl funcional | `examples/parser.kl` | ✅ Tokeniza y parsea `hello.kl` correctamente (sin infinite loop) |
| Tests | - | ✅ 84 tests, 0 failures |

### Sesión 16 — Phase 3.5: Match con enum variants + enum construction
| Feature | Archivos | Estado |
|---------|----------|--------|
| Enum register in struct_defs | `klc_mir/src/lower.rs` | ✅ Enums registered as `{disc: I32, payload: I64}` tagged union |
| Enum variant index pre-scan | `klc_mir/src/lower.rs` | ✅ `enum_variants` map `enum_name → {variant_name → index}` |
| `Decl::Enum` in main lowering loop | `klc_mir/src/lower.rs` | ✅ No-op (type already registered) |
| `Pattern::EnumVariant` in match lowering | `klc_mir/src/lower.rs` | ✅ Discriminant check + payload binding via FieldPtr |
| Enum construction in `Expr::FunctionCall` | `klc_mir/src/lower.rs` | ✅ `Option.Some(v)` → tagged union creation |
| Enum construction in `Expr::PropertyAccess` | `klc_mir/src/lower.rs` | ✅ `Option.None` → tagged union without payload |
| Fix: struct local allocated LAST in enum construction | `klc_mir/src/lower.rs` | ✅ So `ctx.next_local - 1` returns the struct, not a temp |
| Fix: `eat_identifier` accepts `None`, `True`, `False` | `klc_frontend/src/parser.rs` | ✅ Enum variants with keyword names parse correctly |
| Fix: `parse_enum` safety check for empty name | `klc_frontend/src/parser.rs` | ✅ Prevents infinite loop |
| Fix: type checker `bind_pattern` for match arms | `klc_semantic/src/type_checker.rs` | ✅ Pattern variables registered in type checker's symbol table |
| Fix: i32→i64 cast for `println(i32)` | `klc_mir/src/lower.rs` | ✅ Non-string print args widened to i64 for `kl_println_int` |
| enum_test end-to-end | `examples/enum_test.kl` | ✅ `Option.Some(42)` → match → `v=42` PASS |
| enum_test2 end-to-end | `examples/enum_test2.kl` | ✅ `Some(42)` + `None` ambos funcionan → PASS |
| Tests | - | ✅ 84 tests, 0 failures |

### Sesión 17 — Phase 3.5: Closures end-to-end (I32 params + I32 return)
| Feature | Archivos | Estado |
|---------|----------|--------|
| Closure: MIR `FnAddr` + `CallIndirect` | `klc_mir/src/mir.rs` | ✅ FnAddr stores fn pointer, CallIndirect calls through it |
| Closure: Parser `(x) => body` | `klc_frontend/src/parser.rs` | ✅ LParen backtracking to detect closure |
| Closure: Scope resolution | `klc_semantic/src/scope.rs` | ✅ Push scope, bind params |
| Closure: Type checker | `klc_semantic/src/type_checker.rs` | ✅ I32 params, infer return type |
| Closure: Lowering `Expr::Closure` | `klc_mir/src/lower.rs` | ✅ Creates unique `_closure_N` function + `FnAddr` |
| Closure: Lowering `CallIndirect` | `klc_mir/src/lower.rs` | ✅ Detects closure-typed local, emits `CallIndirect` |
| Closure: Codegen `FnAddr` | `klc_backend/src/codegen.rs` | ✅ Stores function pointer via `as_pointer_value()` |
| Closure: Codegen `CallIndirect` | `klc_backend/src/codegen.rs` | ✅ `build_indirect_call` with dynamic params |
| Fix: Closure functions lost before collection | `klc_mir/src/lower.rs` | ✅ Second collection pass after declaration lowering |
| Fix: `try_as_basic_value()` incorrect API | `klc_backend/src/codegen.rs` | ✅ Usa `ValueKind::Basic(result)` pattern existente |
| closure_test | `examples/closure_test.kl` | ✅ `(x) => x*2`, `double(21)` → 42 PASS |
| closure_test2 | `examples/closure_test2.kl` | ✅ `(a,b) => a+b`, `add(100,1)` → 101 PASS |
| Tests | - | ✅ 86 tests, 0 failures |

### Sesión 18 — Phase 3.5: Async/Await end-to-end (thread-based)
| Feature | Archivos | Estado |
|---------|----------|--------|
| Runtime FFI: `kl_spawn_thread` | `klc_runtime/src/thread.rs` | ✅ `extern "C" fn` que spawns `thread::spawn` con `extern "C" fn(i64)->i64` |
| Runtime FFI: `kl_join_thread` | `klc_runtime/src/thread.rs` | ✅ Joins thread handle, retorna `i64` result |
| MIR: `AsyncSpawn` instruction | `klc_mir/src/mir.rs` | ✅ `MirInst::AsyncSpawn { dest, function_name, arg }` |
| MIR: `AsyncAwait` instruction | `klc_mir/src/mir.rs` | ✅ `MirInst::AsyncAwait { dest, handle }` |
| Lowering: `Expr::Async` | `klc_mir/src/lower.rs` | ✅ Crea función `_async_N` + emite `AsyncSpawn` |
| Lowering: `Expr::Await` | `klc_mir/src/lower.rs` | ✅ Emite `AsyncAwait` + cast a tipo esperado |
| Codegen: extern decls | `klc_backend/src/codegen.rs` | ✅ `kl_spawn_thread(ptr, i64) -> i64`, `kl_join_thread(i64) -> i64` |
| Codegen: `AsyncSpawn` | `klc_backend/src/codegen.rs` | ✅ Llama `kl_spawn_thread` con fn pointer + arg |
| Codegen: `AsyncAwait` | `klc_backend/src/codegen.rs` | ✅ Llama `kl_join_thread` con handle |
| Ownership + Optimizer | `klc_mir/src/ownership.rs`, `optimize.rs` | ✅ Match arms nuevos |
| Fix: `p.len` field access vs `len(p)` | `klc_mir/src/lower.rs` | ✅ Type check antes de interceptar como `kl_list_len` |
| Fix: `kl_alloc(64)` i32→i64 | `klc_mir/src/lower.rs` | ✅ `MirConstant::I64(64)` en vez de `I32(64)` |
| Fix: LLVM module verification | `klc_driver/src/pipeline.rs` | ✅ `verify()` + dump IR en fallo |
| async_test end-to-end | `examples/async_test.kl` | ✅ `async 42` → spawn → `await task` → join → `42` PASS |
| parser.kl build | `examples/parser.kl` | ✅ Build exitoso post-fixes |
| Tests | - | ✅ 86 tests, 0 failures |

### Sesión 21 — Phase 6: Generic structs lowering (monomorphization v1)
| Feature | Archivos | Estado |
|---------|----------|--------|
| Root cause: generics parseados pero ignorados en semantic + lowering | — | ✅ `type_params` en FunctionDecl/StructDecl nunca se usaban; `ast_type_to_mir` para `AstType::Generic` solo manejaba `list` |
| Lowerer: `generic_struct_templates` RefCell | `klc_mir/src/lower.rs` | ✅ Almacena StructDecl genéricos (con type_params no vacíos) para monomorfización tardía |
| Pre-scan: structs genéricos saltados de `struct_defs` | `klc_mir/src/lower.rs` | ✅ Pass 1/2 del pre-scan omite structs con type_params |
| Helper: `is_type_ref` | `klc_mir/src/lower.rs` | ✅ Detecta si un AstType referencia un type param específico |
| Helper: `mir_type_to_string` | `klc_mir/src/lower.rs` | ✅ Serializa MirType para name mangling |
| Helper: `make_concrete_name` | `klc_mir/src/lower.rs` | ✅ Crea nombre único: `Pair__i32_str` |
| Helper: `ast_type_to_mir_with_subst` | `klc_mir/src/lower.rs` | ✅ Convierte AstType → MirType con sustitución de type params |
| `Expr::StructLiteral`: monomorfización on-the-fly | `klc_mir/src/lower.rs` | ✅ Inferencia de type params desde field values, crea struct concreto, registra en struct_defs |
| Fix: borrow closure en `generic_struct` check | `klc_mir/src/lower.rs` | ✅ `then()` clona StructDecl para liberar RefCell borrow |
| generic_struct end-to-end | `examples/generic_struct.kl` | ✅ `Pair<i32,i32>`, `Pair<str,str>`, `Pair<i32,str>` → todos funcionan |
| Tests | - | ✅ 86 tests, 0 failures |

### Sesión 22 — Phase 6: Generic functions monomorphization
| Feature | Archivos | Estado |
|---------|----------|--------|
| Root cause: generic functions skipped in lowering | `klc_mir/src/lower.rs` | ✅ `Decl::Function` con `type_params` se ignoraba completamente — sin especialización |
| Lowerer: `generic_function_templates` + `specialized_mir_functions` | `klc_mir/src/lower.rs` | ✅ Almacena templates y funciones MIR especializadas |
| Pre-scan: funciones genéricas saltadas de lowering directo | `klc_mir/src/lower.rs` | ✅ `lower_program` omite funciones con type_params, se especializan lazy |
| Helper: `extract_generic_bindings` | `klc_mir/src/lower.rs` | ✅ Match de AstType param contra MirType arg para inferir type params |
| Helper: `infer_function_type_params` | `klc_mir/src/lower.rs` | ✅ Infiere todos los type params de una llamada concreta |
| Helper: `mir_type_to_ast_type` | `klc_mir/src/lower.rs` | ✅ Convierte MirType → AstType para sustitución en AST |
| Helper: `substitute_ast_type` | `klc_mir/src/lower.rs` | ✅ Sustituye type params por AstTypes concretos en todo el árbol |
| Helper: `clone_and_specialize_function` | `klc_mir/src/lower.rs` | ✅ Clona FunctionDecl sustituyendo params, return type, y body types |
| Helper: `substitute_stmt_types` | `klc_mir/src/lower.rs` | ✅ Walk de statements para sustituir type params en variable declarations |
| Helper: `pre_register_generic_type` | `klc_mir/src/lower.rs` | ✅ Pre-registra structs concretos en struct_defs ANTES de lower_function |
| Fix: `ast_type_to_mir` Generic case | `klc_mir/src/lower.rs` | ✅ `else` branch ahora crea nombre concreto (`Pair__i32_str`) en vez de `args[0]` |
| Call handler: detección + especialización on-the-fly | `klc_mir/src/lower.rs` | ✅ En `Expr::FunctionCall`, si target es genérico → infiere type args, especializa, emite call |
| generic_fn end-to-end | `examples/generic_fn.kl` | ✅ `first([10,20,30])` → 10, `make_pair(1, "hello")` → field access OK |
| Tests | - | ✅ 86 tests, 0 failures |

### Sesión 23 — Phase 3.5 cerrada: constructor fix + docs audit
| Feature | Archivos | Estado |
|---------|----------|--------|
| Root cause: `this.count = start` tail expression | `klc_mir/src/lower.rs` | ✅ Parser produce `Expr::Binary { op: Assign, left: PropAccess }` no `Expr::Assignment` |
| Fix: `tail_is_field_assign` now detects both patterns | `klc_mir/src/lower.rs` | ✅ Añadido match arm para `Expr::Binary { operator: BinaryOp::Assign }` |
| class_method_test.kl | `examples/class_method_test.kl` | ✅ `Counter(10).increment()+increment()+add(5)` → 17 |
| enum_test2.kl memory corruption | `examples/enum_test2.kl` | ✅ 5/5 runs determinista (fix colateral del constructor) |
| Phase 3.5 audit: closures, methods, enums, async | — | ✅ Todo verificado end-to-end |
| Phase 1-5 y 3.5 marcados completados en docs | `docs/13-roadmap.md`, `docs/16-status.md`, `AGENTS.md` | ✅ For loops ✅, Generics ✅, Error handling (? operator) ✅ |
| 23 examples verificados | `examples/*` | ✅ 22/23 pasan (test_math.kl falla por module search paths) |
| Tests | - | ✅ 86 tests, 0 failures |

### Sesión 24 — Phase 6: Ternary operator (cond ? a : b)
| Feature | Archivos | Estado |
|---------|----------|--------|
| AST: `Expr::Ternary` | `crates/klc_core/src/ast.rs` | ✅ New variant |
| Parser: ternary + error prop conviven | `crates/klc_frontend/src/parser.rs` | ✅ `parse_binary` handles `?` as ternary (prec=2); removed from `parse_postfix`; fallback a error-prop si no hay `:` |
| Type checker | `crates/klc_semantic/src/type_checker.rs` | ✅ Cond debe ser bool, then/else unifican |
| Scope resolver | `crates/klc_semantic/src/scope.rs` | ✅ Walk 3 sub-exprs |
| Lowering | `crates/klc_mir/src/lower.rs` | ✅ CondBr → then/else/merge blocks con result_alloca |
| Formatter | `crates/klc_tools/src/formatter.rs` | ✅ `cond ? then : else` |
| Nested ternary | — | ✅ Right-assoc (`a ? b ? c : d : e`) |
| String ternary | — | ✅ `cond ? "yes" : "no"` |
| Error prop compat | — | ✅ `foo()?` sigue funcionando |
| Examples | `examples/ternary.kl` | ✅ Adult, 42, 3 |

### Sesión 25 — Phase 6: Match-expression + Guard fix + Defer lowering + Docs update
| Feature | Archivos | Estado |
|---------|----------|--------|
| AST: `Expr::MatchExpr` | `crates/klc_core/src/ast.rs` | ✅ New variant with expression, arms, span |
| Parser: match in expression context | `crates/klc_frontend/src/parser.rs` | ✅ `parse_match_expr()` in `parse_primary`; `TokenKind::Match` in `current_is_expr_start` |
| Type checker: `Expr::MatchExpr` | `crates/klc_semantic/src/type_checker.rs` | ✅ Infers expression, checks arms, unifies return types |
| Scope resolver: match expression | `crates/klc_semantic/src/scope.rs` | ✅ Walk expression + arms + guards |
| Lowering: `Expr::MatchExpr` | `crates/klc_mir/src/lower.rs` | ✅ CondBr pattern chain with result_alloca; Wildcard/Identifier/Literal patterns |
| Formatter: `Expr::MatchExpr` | `crates/klc_tools/src/formatter.rs` | ✅ Nested match output with indentation |
| Guard fix: CondBr not a no-op | `crates/klc_mir/src/lower.rs` | ✅ CondBr: true→continue, false→body (early return) |
| Defer lowering: LIFO at return | `crates/klc_mir/src/lower.rs` | ✅ `deferred_exprs` stack; emitted before explicit/implicit return in reverse order |
| Defer test | `examples/defer_test.kl` | ✅ LIFO order verified |
| Guard test | `examples/guard_test.kl` | ✅ CondBr verified |
| Docs updated | `docs/13-roadmap.md`, `16-status.md`, `17-syntax-reference.md`, `01-language-specification.md` | ✅ Match-expression ✅, Guard ✅, Defer ✅ |
| Tests | - | ✅ 86 tests, 0 failures |
| Type alias lowering | `crates/klc_mir/src/lower.rs` | ✅ `TYPE_ALIAS_CACHE` thread_local + resolution in `ast_type_to_mir` (User catch-all) |
| Chained aliases | `examples/type_alias_test.kl` | ✅ `type Years = Age` → `type Age = i32` resuelve a i32 |
| Docs updated (2) | `docs/13-roadmap.md`, `16-status.md`, `17-syntax-reference.md`, `AGENTS.md` | ✅ Type aliases ✅ |
| Range slicing: `..` lexer + parser | `crates/klc_frontend/lexer.rs`, `parser.rs` | ✅ `DotDot` token emitido por lexer, parseado como `BinaryOp::Range`, detectado dentro de `[]` → `RangeSlice` |
| Range slicing: lowering + runtime | `klc_mir/src/lower.rs`, `klc_runtime/src/list.rs` | ✅ `Expr::RangeSlice` → `Call kl_list_slice` |
| Range slicing: codegen extern | `klc_backend/src/codegen.rs` | ✅ `kl_list_slice(ptr, i64, i64) -> ptr` |
| Range slicing test | `examples/slice_test.kl` | ✅ `list[0..3]` → `[0,1,2]` verificado |
| Docs updated (3) | `docs/13-roadmap.md`, `16-status.md`, `17-syntax-reference.md`, `AGENTS.md` | ✅ Range slicing ✅ |
| Spread: `...` lexer token | `crates/klc_frontend/token.rs`, `lexer.rs` | ✅ `DotDotDot` token + lexer `...` → `DotDotDot` |
| Spread: parser in list literals | `crates/klc_frontend/parser.rs` | ✅ `LBracket` loop detecta `DotDotDot` → `Expr::Spread` |
| Spread: lowering + runtime | `klc_mir/src/lower.rs`, `klc_runtime/src/list.rs` | ✅ spread en listas → `Call kl_list_extend` |
| Spread: codegen extern | `klc_backend/src/codegen.rs` | ✅ `kl_list_extend(ptr, ptr)` |
| Spread test | `examples/spread_test.kl` | ✅ `[...a, 4, 5]` → `[1,2,3,4,5]` |
| Docs updated (4) | todos | ✅ Spread ✅ |
| Tests | - | ✅ 86 tests, 0 failures |
