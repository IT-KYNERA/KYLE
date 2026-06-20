# Kyle Programming Language — Project Context v3.0

## Overview

Kyle — compiled, statically-typed language combining Python readability (indentation blocks), Rust type safety (strong typing, generics, pattern matching), Go simplicity (fast compilation, built-in tooling), and LLVM performance.

## State — Resumen Ejecutivo

```
Pipeline completo: Lexer → Parser → Semantic → MIR → Backend → Linker ✅
Runtime:        RAII, async, file I/O, string ops, time, testing lib ✅
Std Library:    core, math, io en Kyle ✅
Package Mgr:    manifest, lock, add, remove, info, build, run, test ✅
LSP:            document symbols, workspace symbols, signature help,
                find references, code actions ✅
Formatter:      pretty-printer + comment preservation ✅
VS Code:        extension with syntax highlighting, LSP client, commands ✅
Tests:          86 tests, 0 failures ✅
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
| Tests | - | ✅ 86 tests, 0 failures |

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
cargo test -p klc_core -p klc_frontend -p klc_semantic -p klc_runtime -p klc_tools  # 84 tests, 0 failures ✅
```

## Roadmap (actualizado)

```
FASE 1-2 (complete) — Frontend + Semantic Analysis ✅
├── [x] Lexer (69 tests)
├── [x] Parser (recursive descent, indent-based)
├── [x] AST (all node types + Display)
├── [x] Type checker Hindley-Milner (47 tests)
├── [x] Generics, Contracts, Error types, Optionals
│   Hito: kl check main.kl → No errors found

FASE 3 — Compiler Backend (✅ Complete — build → binario nativo)
├── [x] MIR definition (mir.rs — MirValue, MirInst, MirBasicBlock, MirFunction)
├── [x] AST → MIR lowering (lower.rs — all stmts + exprs)
├── [x] Optimizer (optimize.rs — constant folding, DCE, block removal)
├── [x] inkwell integration (LLVM 18.1, opaque pointers)
├── [x] LLVM codegen (codegen.rs — MIR → LLVM IR via TargetMachine)
├── [x] Linker (linker.rs — clang-based native linking)
│   Hito: kl build hello.kl → ./hello → exit 0

FASE 4 — Std Library & Runtime (✅ Complete)
├── [x] Implementar RAII runtime (destructores, refcount)
├── [x] RAII ownership inference pass
├── [x] Async runtime funcional
├── [x] Standard library básica (core, math, io) in Kyle
├── [x] Runtime: string ops (contains, to_upper, to_lower, trim, replace, input)
├── [x] Runtime: file I/O (open, read_str, write_str, close)
├── [x] Runtime: time (sleep, now)
├── [x] std/testing.kl — assert, assert_eq, assert_str
│   Hito: kl run hello.kl → "Hello, World!"

FASE 5 — Tooling (✅ Complete)
├── [x] Package manager: manifest module + kl add/remove/info commands
├── [x] LSP: textDocument/documentSymbol, workspace/symbol, textDocument/signatureHelp
├── [x] LSP: find references, code actions
├── [x] Formatter: AST pretty-printer (all nodes)
├── [x] Formatter: comment preservation (requires AST spans; lexer+parser span tracking fixed)
├── [x] VS Code extension: syntax highlighting, LSP client, build/run/check commands
│   Hito: klc fmt, klc lsp, kl run, kl build — todo funcional

FASE 6 — Self-Hosting (⏳ In Progress)
├── [x] Runtime char ops + ord() builtin
├── [x] Fixes: if_then block collision, elif chain, string escapes, string return type, string concat type, break lowering
├── [x] Lexer escrito en Kyle (examples/lexer.kl) — tokeniza archivos reales
├── [x] Fix: char/int comparison + type widening en lowering
├── [x] Fix: RAII alloc en todas las funciones string runtime (kl_alloc)
├── [x] Codegen Cast ptr↔int via ptrtoint/inttoptr
├── [x] String lists: `["a", "b"]` → List(Str), `tokens[0]` → str
<<<<<<< Updated upstream
├── [x] Parser escrito en Kyle
├── [ ] Compilador completo en Kyle
=======
├── [x] Parser escrito en KL
├── [x] Fix: auto-declared variable type inference (`result = expr` → type checker registra variable)
├── [ ] Compilador completo en KL
>>>>>>> Stashed changes
│   Hito: kl build klc
```

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

## Documentación (16 docs)

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
| 13 | `roadmap.md` | Roadmap de 6 fases (438 líneas) |
| 14 | `error-catalog.md` | Catálogo de errores E/W/L (395 líneas) |
| 15 | `abi-specification.md` | ABI y FFI (168 líneas) |
