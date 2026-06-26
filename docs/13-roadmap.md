# Kyle Language Roadmap v7.0 — Production Polish

---

## Overview

Kyle is developed in phases. **Phases 0–6 and 3.5** are complete: full compiler
pipeline (lexer → parser → semantic → MIR → LLVM → native binary), runtime
(RAII, async, string ops, file I/O), standard library basics, tooling
(CLI, LSP, formatter, package manager, VS Code extension), AND all syntax
features generate working code end-to-end.

**Phase 7 — Cross-Platform Support** was completed earlier, making the runtime
I/O cross-platform and the target triple auto-detectable. Distribution pipeline
(GitHub Actions release, install.sh, public repo) is also done.

**Phase 8 — Distribution & Tooling Polish** is the current priority: make Kyle
feel like a professional language — better VS Code experience, proper build
output, LSP autocompletion, logo/branding, snippets, and website.

```
Phase 6:  Language Completion           ← ✅ Complete
Phase 7:  Cross-Platform + Distribution ← ✅ Complete
Phase 8:  Distribution & Tooling Polish ← 🔶 CURRENT
Phase 9:  Self-Hosting                  ← ⏸️ deferred
Phase 10: Production Ecosystem          ← 📅 future
```

**Memory model:** RAII + Compiler-Inferred Ownership (NO garbage collector).

For the precise, verified breakdown of what generates working code vs.
what is still a placeholder, see **`docs/16-status.md`**.

---

## Phase 0: Language Design

### Status: Complete ✅

All 17 specification documents are written. The language syntax, type
system, memory model, and architecture are fully defined.

### Tasks

```text
[x] Define language vision
[x] Write language specification
[x] Define formal grammar
[x] Specify AST structure
[x] Design type system
[x] Design error system
[x] Design module system
[x] Specify standard library
[x] Design async runtime
[x] Define memory model (RAII + Compiler-Inferred Ownership)
[x] Design compiler architecture
[x] Plan project structure
[x] Design package manager
[x] Create this roadmap
[x] All documents frozen, consistent, and finalized
```

---

## Phase 1: Compiler Frontend

### Status: Complete ✅

### Deliverables

```text
Lexer (809 lines, 69 tests) ✅
Parser (1353 lines, recursive descent, indent-based) ✅
AST with all node types (1076 lines) ✅
CLI: klc parse <file.kl> → AST dump ✅
```

### Tasks

```text
[x] Set up Rust workspace (9 crates)
[x] Implement lexer
    - 50+ token types, keywords, operators
    - Literals: int, hex, bin, float, string, char, boolean
    - INDENT/DEDENT (indentation-based blocks)
    - 69 unit tests passing
[x] Implement parser
    - Recursive descent, 12 precedence levels
    - Declarations: import, fn, class, struct, enum, contract, type alias, var/const
    - Statements: if/elif/else, while, for, match, return, break, defer, guard, unsafe, loop, binding-if
    - Expressions: binary, unary, call, property access, closure, async, await, spread, range, loop
    - Types: primitive, user, generic, optional, error
[x] Build AST nodes
    - Program, Decl (9 kinds), Stmt (14 kinds), Expr (18 kinds), Pattern, Type
    - Display impls for all nodes
[x] CLI integration
    - klc parse command with pretty-printed AST dump
```

---

## Phase 2: Semantic Analysis

### Status: Complete ✅

### Deliverables

```text
Type checker Hindley-Milner (1380 lines, 47 tests) ✅
Symbol table + scope resolver ✅
Module resolver (import from .kl files) ✅
Generics (type params, fresh var instantiation) ✅
Contracts (validation, implements keyword) ✅
Error types (! return type, ? operator, E0002) ✅
Optional types (None literal, ?. chain) ✅
Diagnostics system ✅
CLI: klc check <file.kl> → "No errors found" ✅
```

### Tasks

```text
[x] mut keyword (token, parser, mutability enforcement)
[x] Symbol table + scope resolver
[x] Type resolver (primitives, user-defined, generics)
[x] Type inference (Hindley-Milner, constraint solving, unification)
[x] Generics
    - TypeParam in AST, Type::TypeParam in type system
    - Parser: fn foo<T>(x: T) ...
    - Fresh type var instantiation at call sites
[x] Contracts (validator, implements keyword)
[x] Error types (! return type suffix, ? postfix operator)
[x] Optional types (None literal, ?. chain)
[x] Diagnostics system (Error/Warning/Lint codes, span-based)
[x] Module resolver (import resolution, file loading)
```

---

## Phase 3: Compiler Backend

### Status: Complete ✅

### Deliverables

```text
MIR definition + lowering + optimizer (1800+ lines) ✅
LLVM codegen via inkwell 0.9 / LLVM 18.1 ✅
Native linker via clang ✅
CLI: klc build <file.kl> → native binary ✅
CLI: klc run <file.kl> → compile + execute ✅
CLI: klc mir <file.kl> → MIR dump ✅
```

### Tasks

```text
[x] MIR definition
    - MirValue, MirConstant, MirType, MirInst, MirTerminator
    - MirBasicBlock, MirFunction, MirModule
    - Display impls for all MIR types
[x] AST → MIR lowering
    - LowerCtx with locals, blocks, block_counter
    - All statements and expressions
    - Constructor lowering for classes
[x] MIR optimizer
    - Constant folding
    - Dead code elimination
    - Remove unreachable basic blocks
[x] LLVM codegen
    - inkwell integration, LLVM 18.1, opaque pointers
    - Type mapping (MIR → LLVM types)
    - Alloca/Store/Load for locals
    - Binary/Unary operations
    - Call + argument mapping
    - Basic block building with terminators
    - TargetMachine for native object file emission
[x] Native linker
    - clang-based linking of .o → binary
    - Shared library support
[x] Pipeline orchestration
    - Source → MIR → LLVM → .o → binary (end-to-end)
    - Check + MIR + Build subcommands
```

---

## Phase 4: Runtime + Standard Library

### Status: Complete ✅

### Tasks

#### 4.1 — Runtime (Rust)

```text
[x] Core runtime crate (klc_runtime)
    - print/println wrappers (write to stdout) ✅
    - str representation (ptr + length, UTF-8) ✅
    - heap allocation wrappers (malloc/free for RAII) ✅
    - program entry point (_start → main wrapper) ✅
    - exit code handling ✅
    - String ops: contains, to_upper, to_lower, trim, replace, concat, input ✅
    - Char ops: char_at, is_digit, is_alpha, is_alnum, is_whitespace, is_upper, is_lower, ord ✅
    - File I/O: open, read_str, write_str, close ✅
    - Time: sleep(ms), now() -> i64 ✅
    - Thread spawn/join (kl_spawn_thread, kl_join_thread) ✅
[x] Link runtime with klc build
```

#### 4.2 — RAII Ownership Inference

```text
[x] Ownership inference pass (klc_mir/src/ownership.rs) ✅
    - Escape analysis ✅
    - Move inference (memcpy for zero-cost) ✅
    - Refcount inference (Arc/Rc wrappers) ✅
    - Insert retain/release at scope exits ✅
```

#### 4.3 — Compiler String/Char Support

```text
[x] Builtins in symbol_table, lower, codegen ✅
[x] String return from user functions ✅
[x] String concat result type (MirType::Str) ✅
[x] str() cast i32→i64 before kl_i64_to_str ✅
[x] len() returns I32 ✅
[x] Inference variable type (local_types map) ✅
```

#### 4.4 — Async Runtime

```text
[x] Work-stealing thread pool ✅
[x] Task<T> with Future poll mechanism ✅
[x] Channel<T> with send/recv ✅
[x] Async/await lowering end-to-end ✅
    - Expr::Async → spawn ✅
    - Expr::Await → join ✅
    - MirInst::AsyncSpawn / MirInst::AsyncAwait ✅
    - Codegen: kl_spawn_thread / kl_join_thread FFI ✅
```

---

## Phase 5: Tooling & Ecosystem

### Status: Complete ✅

```text
[x] Package manager (klc_tools)
    - kl new/init, add/remove, info, build/run/test ✅
    - Manifest (kl.toml): serde + read/write ✅
    - Lock file: serde + read/write ✅
[x] Language server (LSP)
    - documentSymbol, workspace/symbol, signatureHelp ✅
    - findReferences, codeAction ✅
[x] Code formatter
    - AST pretty-printer (all nodes) ✅
    - Comment preservation ✅
    - klc fmt <file.kl> command ✅
[x] VS Code extension
    - Syntax highlighting (TextMate grammar) ✅
    - Language config ✅
    - LSP client ✅
    - Commands: kl.run, kl.build, kl.check ✅
```

---

## Phase 3.5: Backend Gap Closure

### Status: Complete ✅

Features that parsed and type-checked but generated no code were implemented:

```text
[x] StructLiteral + Method Dispatch + Ownership Fixes ✅
[x] Match with enum variants + enum construction ✅
[x] Closures end-to-end (FnAddr + CallIndirect) ✅
[x] Async/await end-to-end (thread-based spawn/join) ✅
[x] Struct pass-by-reference ABI ✅
    → all generate working code
```

---

## Phase 6: Language Completion 🔶 CURRENT

### Status: 🔶 Current — Priority #1

### Goal

**Que absolutamente TODA la sintaxis del lenguaje genere código funcionando.**
Sin features "a medias". Sin "parsea pero no genera código".
Que cualquier programa escrito en Kyle según la especificación compile y
se ejecute correctamente.

### Why this is first

Sin las features completas, Kyle no sirve para proyectos reales.
Multiplataforma y distribución se hacen después, cuando el lenguaje
en sí mismo esté completo.

### Tasks — Organized by Priority

#### 🟥 P0 — End-to-End Language Features (bloquean el MVP)

```text
[x] For loops — `for x in list:`, `for i in 0..10:` ✅
    - parser ✅ | type-checker ✅ | lowering ✅ | codegen ✅ | runtime ✅
    - For-Else / While-Else ✅

[x] Generics — monomorphization en lowering + codegen ✅
    - Generic structs (Pair<T,U>) + generic functions (first<T>, make_pair<T,U>)

[x] Error handling — `!` return type + `?` operator ✅
    - ? operator con Option<T> funciona (error_test.kl → 42)
    - ! return type syntax parsea + lowering

[x] String interpolation — `"Hello {name}"` ✅
    - parser ✅ | type-checker ✅ | lowering ✅ | codegen ✅ | runtime ✅

[x] Range for loop — `for i in 0..10` ✅
    - Counter loop lowering (sin heap allocation)
    - Continue sin saltar incremento (inc_label block)

[x] For-Else / While-Else ✅
    - Python-style break-flag implementation
```

#### 🟧 P1 — Language Features Secundarias

```text
[x] Defer — lowering + codegen ✅ (LIFO order)
[x] Guard — lowering + codegen ✅ (CondBr)
[x] Type aliases — `type MyInt = i32` ✅ (lowering + codegen)
[x] Dict/Map literals — `{ "key": value }` ✅
    - parser ✅ | type-checker ✅ | lowering ✅ | codegen ✅ | runtime ✅
[x] Spread operator — `[...list, new_elem]` ✅
[x] Range slicing — `items[0..3]` ✅
[x] Ternary operator — `cond ? a : b` ✅
[x] Match como expresión ✅
[x] Optional chaining — `?.` ✅ (property access en Option payload)
[ ] const fn — compile-time evaluation
    - parser ✅ | type-checker ❌ | lowering ❌ | codegen ❌
```

#### 🟦 P3 — Standard Library

```text
[x] std/core.kl — Option<T>, utility functions ✅
[x] std/math.kl — abs, pow, sqrt, gcd, min, max, clamp ✅
[x] std/io.kl — file I/O wrappers ✅
[x] std/testing.kl — assert, assert_eq, assert_str, assert_ne ✅
[x] std/collections.kl — list_sum, list_product, list_max, list_min, list_range ✅
[x] std/json.kl — json_parse + json_stringify (runtime FFI) ✅
[x] std/str.kl — starts_with, ends_with, capitalize, repeat_str ✅
[x] std/time.kl — timestamp, sleep_ms, seconds_since ✅
```

#### 🟪 P4 — Tooling Polish

```text
[x] LSP autocompletion (textDocument/completion) ✅
[x] LSP go-to-definition (textDocument/definition) ✅
[x] LSP hover documentation (textDocument/hover) ✅
[ ] Debugger support — DWARF debug info
[ ] LSP rename (textDocument/rename)
[ ] LSP formatting integration
[ ] LLVM optimization levels (O0, O1, O2, O3)
```

#### 🟩 P5 — Robustness & Testing

```text
[ ] Fix LLVM verification errors for all programs
[ ] Proper error messages for lowering/codegen failures
[ ] Lint warnings — unused variables, dead code
[ ] 100+ integration tests (examples/*.kl run and verify output)
[ ] Fuzz testing for lexer + parser
[ ] Standard library test suite
[ ] CI pipeline (GitHub Actions)
```

### Milestone

```text
klc run ANY_PROJECT.kl → works, no crashes
klc test → full suite passes
klc fmt → formats correctly
All syntax features generate working code
```

---

## Phase 7: Cross-Platform Support ⏸️ NEXT

### Status: ⏸️ Next — after Phase 6

### Goal

Kyle currently runs **only on macOS Apple Silicon (aarch64)**.
Phase 7 makes it work on **Linux (x64 + ARM)**, **Windows (x64 + ARM)**,
and **macOS (Intel + Apple Silicon)**.

### Why this is Phase 7 (not Phase 6)

Phase 6 completes the language. Phase 7 ports it to other platforms.
No tiene sentido portar un lenguaje incompleto.

### Cross-Platform Strategy

Kyle is written in Rust + LLVM — **both are inherently cross-platform**.
No se necesita reescribir el compilador para cada plataforma. Solo
hay que adaptar ~5 puntos localizados:

| Componente | Cambio necesario | Dificultad |
|------------|-----------------|------------|
| Runtime I/O | Reemplazar POSIX raw syscalls por `std::fs::File` + `std::io` | Baja (~100 líneas) |
| Target triple | Usar `TargetMachine::get_default_triple()` en vez de hardcodear | Muy baja (~5 líneas) |
| Linker | Detectar SO con `cfg!(windows)` y usar linker/platform correcto | Baja (~20 líneas) |
| Extensión .exe | Usar `std::env::consts::EXE_EXTENSION` | Muy baja (~3 líneas) |
| LLVM paths | Configurar rutas de LLVM por plataforma en `.cargo/config.toml` | Muy baja (~10 líneas) |
| VS Code path | Buscar `klc.exe` en Windows | Muy baja (~5 líneas) |

**Platformas target (6 targets):**

| Platforma | Triple Rust | Prioridad |
|-----------|-------------|-----------|
| macOS Apple Silicon | `aarch64-apple-darwin` | ✅ Already works |
| macOS Intel | `x86_64-apple-darwin` | Alta |
| Linux x64 | `x86_64-unknown-linux-gnu` | Alta |
| Linux ARM | `aarch64-unknown-linux-gnu` | Media |
| Windows x64 | `x86_64-pc-windows-msvc` | Alta |
| Windows ARM | `aarch64-pc-windows-msvc` | Baja |

### Tasks

```text
[ ] Runtime I/O — abstraer POSIX syscalls
    - Actual: klc_runtime/src/io.rs usa open/read/write/close/nanosleep POSIX
    - Solución: reemplazar con std::fs::File + std::io::{Read, Write}
    - Rust stdlib es cross-platform, no requiere cambios de lógica

[ ] Target triple — auto-detección
    - Actual: pipeline.rs hardcodea "arm64-apple-macosx"
    - Solución: Target::initialize_all() + get_default_triple()

[ ] Linker — soporte multiplataforma
    - Actual: hardcodea "clang" + nombre de archivo sin extensión
    - Solución: detectar SO, usar link.exe en Windows

[ ] CLI — extensión .exe
    - Actual: produce binario sin extensión
    - Solución: usar std::env::consts::EXE_EXTENSION

[ ] LLVM paths — config por plataforma
    - Actual: .cargo/config.toml solo tiene ruta Linux aarch64
    - Solución: agregar secciones condicionales por target

[ ] VS Code — detección Windows
    - Actual: extension.ts busca "klc" sin extensión
    - Solución: detectar plataforma, probar klc.exe

[ ] Test en Linux x64 (CI)
[ ] Test en Windows x64 (CI)
[ ] Test en macOS Intel (CI)
```

### Milestone

```text
klc build hello.kl → funciona en macOS (Intel + ARM), Linux (x64 + ARM), Windows (x64)
klc run hello.kl → funciona en las 3 plataformas
All tests → pasan en todas las plataformas
CI pipeline con macOS + Linux + Windows
```

---

## Phase 8: Distribution & Tooling Polish 🔶 CURRENT

### Status: 🔶 Current — Priority #1

### Goal

Hacer que Kyle se sienta como un **lenguaje de programación profesional**:
VS Code extension con highlighting completo, autocompletado LSP inteligente,
build output limpio, logo y branding, snippets, y documentación clara.

### Why this is important now

El lenguaje ya compila y ejecuta código. Lo que falta es la **experiencia de
usuario**: que al abrir un archivo .kl en VS Code se vea colorizado con
precisión, que al escribir aparezcan sugerencias útiles, que al construir
un proyecto los archivos binarios no ensucien el source, y que el lenguaje
tenga una identidad visual profesional.

### Tasks — Organized by Priority

#### 🟥 P0 — VS Code Extension Polish

```text
[x] Syntax highlighting: keywords, types, builtins, operators ✅
[x] Char literal highlighting ('a', '\n') ✅
[x] String escape sequences (\n, \t, \x00, \u0000) ✅
[x] UPPERCASE constant highlighting ✅
[x] Language configuration: comments, brackets, auto-closing ✅
[x] Indentation rules (onEnter after `:`) ✅
[x] Folding support (region markers) ✅
[x] Word pattern for selection/double-click ✅
[x] Code snippets: fn, class, enum, match, for, if, while, import, defer, async/await ✅
[x] Extension icon for file type ✅
[x] Settings: kl.klcPath configuration ✅
[x] out/extension.js synced with source TypeScript (LSP client works) ✅
```

#### 🟧 P1 — LSP Autocompletion

```text
[x] All 44 builtins (print, str, len, contains, etc.) ✅
[x] All 8 Decl variants (functions, variables, constants, classes, structs, enums, contracts, type aliases) ✅
[x] 33 keywords with context detail ✅
[x] Prefix filtering (only show matching completions) ✅
[x] Sort ordering (builtins first, then project symbols, then keywords) ✅
[ ] Dot-triggered struct field / method completions
[ ] Scope-aware completions (inner functions, variables)
[ ] Completion resolve provider (documentation on demand)
```

#### 🟦 P2 — Build Output Structure

```text
[ ] Project build → target/debug/main (or target/release/main) ✅
[x] Single-file build → binary next to source, artifacts in .klc-build/ ✅
[x] Proper .gitignore (target/, *.o, *.ll, .klc-build/) ✅
```

#### 🟪 P3 — Branding & Logo

```text
[x] SVG logo (purple "K" on rounded square) ✅
[x] Color palette (purple #6C3FC5, dark #1A1A2E) defined in docs ✅
[ ] PNG icons (128x128 for VS Code marketplace, 16x16 for file icons)
[ ] Logo in VS Code extension (language icon)
```

#### 🟩 P4 — Website (kl-lang.org)

```text
[ ] Landing page with hero, install command, hello world
[ ] Documentation section (language reference, std lib API)
[ ] Downloads page (binaries + VS Code extension)
```

### Milestone

```text
klc run ANY_PROJECT.kl → works, no crashes
VS Code extension → syntax highlighting + LSP + autocompletado + snippets
Build output → target/debug/ y .klc-build/ sin ensuciar source
Logo y branding → identidad visual profesional
kl-lang.org → documentación y descargas
```

---

## Phase 9: Self-Hosting ⏸️ DEFERRED

### Status: ⏸️ Deferred — after Distribution

### Goal

Rewrite the Kyle compiler in Kyle itself, achieving self-hosting.
Only after the language is complete, multiplatform, AND distributable.

### What is already done

```text
[x] Lexer in Kyle — examples/lexer.kl (tokenizes real files) ✅
[x] Parser in Kyle — examples/parser.kl (recursive AST) ✅
[x] Semantic analyzer in Kyle — examples/semantic.kl (type-checks) ✅
```

### What remains

```text
[ ] MIR lowering in Kyle (~2200 lines of Rust to translate)
[ ] Codegen in Kyle (~1100 lines to translate)
[ ] Bootstrap: klc compiles itself
```

### Milestone

```text
kl build klc   # compiler compiles itself
```

---

## Phase 10: Production Ecosystem 📅 FUTURE

### Status: 📅 Future

```text
[ ] Package registry (kl publish / kl search)
[ ] WASM compilation target
[ ] Cross-compilation
[ ] C FFI improvements
[ ] Debugger (GDB/LLDB integration)
[ ] Profiling tools
[ ] Language server: refactors, code lens, inlay hints
[ ] IDE extensions: JetBrains, Neovim, Helix
[ ] Performance tuning
[ ] Async: state-machine based (not thread-based)
[ ] Macros / metaprogramming
[ ] Error messages: Rust-level quality
```

---

## Timeline

```text
Phase 0:   Language Design                      — Complete ✅
Phase 1:   Compiler Frontend                    — Complete ✅
Phase 2:   Semantic Analysis                    — Complete ✅
Phase 3:   Compiler Backend                     — Complete ✅
Phase 4:   Runtime + Builtins                   — Complete ✅
Phase 5:   Tooling & Ecosystem                  — Complete ✅
Phase 3.5: Backend gap closure                  — Complete ✅
Phase 6:   Language Completion                  — Complete ✅
Phase 7:   Cross-Platform + Distribution        — Complete ✅
Phase 8:   Distribution & Tooling Polish        — 🔶 Current
Phase 9:   Self-Hosting                         — ⏸️ Deferred
Phase 10:  Production Ecosystem                 — 📅 Future
```

---

## Release Milestones

### v0.1.0 — Alpha ✅

```text
Lexer + Parser working
AST dump available
```

### v0.2.0 — Alpha ✅

```text
Type checker working
Semantic analysis complete
```

### v0.3.0 — Beta ✅

```text
Code generation working
Native binaries produced
klc build + klc run functional
```

### v0.4.0 — Beta ✅

```text
RAII runtime working
Standard library basics
Hello World → actual stdout output
String ops, char ops, file I/O, time
```

### v0.5.0 — Beta ✅

```text
Async runtime working
Package manager working
Language server working
Code formatter working
VS Code extension working
Closures, methods, enums, match working
Struct pass-by-reference ABI
```

### v0.1.0 — Alpha ✅

```text
Full compiler pipeline (lexer → parser → semantic → MIR → LLVM → binary)
RAII runtime + async + string ops + file I/O
Package manager + LSP + formatter + VS Code extension
All syntax features generate working code
53 example programs + 86 unit tests
Public repo + GitHub Actions release workflow
install.sh (curl | sh) installer
```

### v0.8.0 — Beta (Phase 8 — Current 🔶)

```text
🟥 P0: For loops, Generics, Error handling, Optional chaining, String interpolation,
       Range for loop, For-Else/While-Else — todas generan código
🟧 P1: Defer, Guard, Type aliases, Dict/Map, Spread, Range slicing,
       Ternary, Match-expression — todas generan código
🟦 P3: Standard library core + math + io + testing
🟪 P4: LSP completion + hover + go-to-definition (in progress)
🟩 P5: 100+ integration tests, no crashes
```

### v1.0.0 — Stable (Phase 9 — Self-Hosting)

```text
Self-hosting: kl build klc
Production-ready compiler
Stable standard library API
Full tooling support
kl-lang.org con documentación, ejemplos, descargas
VS Code extension .vsix publicada
LSP autocompletado completo
Multiplataforma (macOS, Linux, Windows)
```

```text
Self-hosting: kl build klc
Production-ready compiler
Stable standard library API
Full tooling support
```

---

## Success Metrics

```text
Parity with Python for readability
Within 2x of C performance for compute
Within 1.5x of Go for compile times
Zero runtime crashes for typed code
Full test suite passing
One-command install
Works on all major platforms
```

---

## Version

```text
Kyle Language Roadmap v6.0 — MVP & Distribution Focus
Last updated: 2026-06-25
```
