# Kyle Language Roadmap v5.0 — MVP Focus

---

## Overview

Kyle is developed in phases. Phases 0–5 (compiler pipeline, runtime,
tooling, and backend gap closure) plus a fully functional **Phase 3.5**
(closures, method dispatch, match/enum, async/await) are **complete**.

The **current work is Phase 6 — Language Completion**: finish ALL syntax
features so they generate working code end-to-end. No features "a medias".

**Order of execution:**

```
Phase 6:  Language Completion    ← AHORA (sintaxis al 100%)
Phase 7:  Cross-Platform         ← después (Windows + Linux + macOS)
Phase 8:  Self-Hosting           ← después (compilador escrito en Kyle)
Phase 9:  Production Ecosystem   ← futuro (registro, WASM, etc.)
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
[x] All 17 documents frozen, consistent, and finalized
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

### Status: 🔶 Current — Prioridad #1

### Goal

**Que absolutamente TODA la sintaxis del lenguaje genere código funcionando.**
Sin features "a medias". Sin "parsea pero no genera código".
Que cualquier programa escrito en Kyle según la especificación compile y
se ejecute correctamente.

### Why this is first

Sin las features completas, Kyle no sirve para proyectos reales.
Multiplataforma y self-hosting se hacen después, cuando el lenguaje
en sí mismo esté completo.

### Tasks — Organized by Priority

#### 🟥 P0 — End-to-End Language Features (bloquean el MVP)

Estas características ya parsean y type-checkean, pero **no generan código**.
Sin ellas, el lenguaje no es usable.

```text
[x] For loops — `for x in list:`, `for i in range(n):` ✅ COMPLETED
    - parser ✅ | type-checker ✅ | lowering ✅ | codegen ✅ | runtime ✅

[x] Generics end-to-end — monomorphization en lowering + codegen ✅ COMPLETED
    - parser ✅ | type-checker ✅ | lowering ✅ | codegen ✅ | runtime ✅
    - Generic structs (Pair<T,U>) + generic functions (first<T>, make_pair<T,U>)

[x] Error handling — `!` return type + `?` operator ✅ COMPLETED
    - parser ✅ | type-checker ✅ | lowering ✅ | codegen ✅ | runtime ✅
    - `?` operator con Option<T> funciona (error_test.kl → 42)
    - `!` return type syntax parsea pero falta testing end-to-end

[x] String interpolation — `"Hello {name}"` ✅ COMPLETED
    - parser ✅ | type-checker ✅ | lowering ✅ | codegen ✅ | runtime ✅

[x] Optional chaining — `?.` ⚠️ PARCIAL
    - parser ✅ | type-checker ✅ | lowering ✅ (propagación Some/None) | codegen ✅
    - Falta: property access en el payload struct (requiere info de tipo genérico)
```

#### 🟧 P1 — Language Features Secundarias

También parsean pero no generan código. Importantes pero no bloquean el MVP.

```text
[x] Defer — lowering + codegen ✅
    - parser ✅ | type-checker ✅ | lowering ✅ | codegen ✅
[x] Guard — lowering + codegen ✅
    - parser ✅ | type-checker ✅ | lowering ✅ | codegen ✅
[x] Type aliases — `type MyInt = i32` (lowering) ✅
    - parser ✅ | type-checker ✅ | lowering ✅ | codegen ✅
[ ] Dict/Map literals — `{ "key": value }`
    - parser ⚠️ | type-checker ⚠️ | lowering: ❌ | codegen: ❌
[x] Spread operator — `[...list, new_elem]` ✅
    - parser ✅ | type-checker ✅ | lowering ✅ | codegen ✅
[x] Range slicing — `items[0..3]` ✅
    - parser ✅ | type-checker ✅ | lowering ✅ | codegen ✅
[ ] Ternary operator ✅ — `cond ? a : b` (cubre el caso simple de if-como-expresión)
    - parser ✅ | type-checker ✅ | lowering ✅ | codegen ✅
[ ] If-como-expresión (bloques indentados) — diferido; ternary cubre el 90% de casos
[x] Match como expresión ✅
    - parser ✅ | type-checker ✅ | lowering ✅ | codegen ✅
[ ] const fn — compile-time evaluation
    - parser ✅ | type-checker ❌ | lowering: ❌ | codegen: ❌
```

#### 🟦 P3 — Standard Library

```text
[ ] std/core.kl — Option<T>, Result<T, E>, utility functions
    - Prioridad: MEDIA (depende de genéricos P0)
[ ] std/math.kl — abs, pow, sqrt, gcd ✅ (ya existe)
[ ] std/io.kl — file I/O wrappers ✅ (ya existe)
[ ] std/testing.kl — assert, assert_eq, assert_str ✅ (ya existe)
[ ] std/collections.kl — HashMap, Set
    - Prioridad: MEDIA (depende de genéricos P0)
[ ] std/json.kl — JSON parser/generator
    - Prioridad: BAJA
[ ] std/str.kl — split, join, starts_with, etc.
    - Prioridad: MEDIA
[ ] std/time.kl — datetime, duration, formatting
    - Prioridad: BAJA
```

#### 🟪 P4 — Tooling Polish

```text
[ ] LSP autocompletion (textDocument/completion)
[ ] LSP go-to-definition (textDocument/definition)
[ ] LSP hover documentation (textDocument/hover)
[ ] Debugger support — DWARF debug info
[ ] LLVM optimization levels (O0, O1, O2, O3)
```

#### 🟩 P5 — Robustness & Testing

```text
[ ] Fix LLVM verification errors for all programs
    - Type mismatches (i64 vs i32, struct vs ptr)
    - SSA dominance violations
[ ] Proper error messages for lowering/codegen failures
[ ] Lint warnings — unused variables, dead code
[ ] 100+ integration tests (examples/*.kl run and verify output)
[ ] Fuzz testing for lexer + parser
[ ] Standard library test suite
[ ] CI pipeline (GitHub Actions)
```

### Milestone

```text
kl run ANY_PROJECT.kl → works, no crashes
kl test → full suite passes
kl fmt → formats correctly
All syntax features generate working code
```

---

## Phase 7: Cross-Platform Support

### Status: ⏸️ Next — after Phase 6

### Goal

Kyle currently runs **only on macOS Apple Silicon (aarch64)**.
Phase 7 makes it work on **Windows (x64)**, **Linux (x64 + ARM)**,
and **macOS (Intel + Apple Silicon)**.

### Why this is Phase 7 (not Phase 6)

Phase 6 completes the language. Phase 7 ports it to other platforms.
No tiene sentido portar un lenguaje incompleto.

### Tasks

```text
[ ] Runtime I/O — abstraer POSIX syscalls
    - Actual: klc_runtime/src/io.rs usa open/read/write/close/nanosleep POSIX ❌
    - Solución: reemplazar con std::fs::File + std::io::{Read, Write}
    - Rust stdlib es cross-platform, no requiere cambios de lógica
    - Dificultad: BAJA (~100 líneas de reescritura localizada)

[ ] Target triple — auto-detección
    - Actual: pipeline.rs hardcodea "arm64-apple-macosx" / "aarch64-unknown-linux-gnu" ❌
    - Solución: Target::initialize_all() + TargetMachine::get_default_triple()
    - Dificultad: MUY BAJA (~5 líneas de cambio)

[ ] Linker — soporte multiplataforma
    - Actual: linker.rs hardcodea "clang" + ".a" ❌
    - Solución: detectar SO, usar clang.exe + .lib en Windows
    - Dificultad: BAJA (~20 líneas con cfg!(windows))

[ ] CLI — extensión .exe
    - Actual: main.rs produce binario sin extensión ❌
    - Solución: usar std::env::consts::EXE_EXTENSION
    - Dificultad: MUY BAJA (~3 líneas)

[ ] LLVM paths — config por plataforma
    - Actual: .cargo/config.toml solo tiene ruta Linux ❌
    - Solución: agregar [target.'cfg(target_os = ...)'.env] secciones
    - Dificultad: MUY BAJA (~10 líneas en config)

[ ] VS Code — detección Windows
    - Actual: extension.ts busca "klc" sin extensión ❌
    - Solución: detectar plataforma, probar klc.exe
    - Dificultad: MUY BAJA (~5 líneas)
```

### Milestone

```text
klc build hello.kl → funciona en macOS (Intel + ARM), Linux (x64 + ARM), Windows (x64)
klc run hello.kl → funciona en las 3 plataformas
86 tests → pasan en todas las plataformas
CI pipeline con macOS + Linux + Windows
```

---

## Phase 8: Self-Hosting

### Status: ⏸️ Deferred — after Phase 7

### Goal

Rewrite the Kyle compiler in Kyle itself, achieving self-hosting.
Only after the language is complete AND multiplatform.

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

## Phase 9: Production Ecosystem

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
Phase 0:   Language Design              — Complete ✅
Phase 1:   Compiler Frontend            — Complete ✅
Phase 2:   Semantic Analysis            — Complete ✅
Phase 3:   Compiler Backend             — Complete ✅
Phase 4:   Runtime + Builtins           — Complete ✅
Phase 5:   Tooling & Ecosystem          — Complete ✅
Phase 3.5: Backend gap closure          — Complete ✅
Phase 6:   Language Completion          — 🔶 Current (P0→P5)
Phase 7:   Cross-Platform Support       — ⏸️ Next
Phase 8:   Self-Hosting                 — ⏸️ Deferred
Phase 9:   Production Ecosystem         — 📅 Future
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

### v0.6.0 — RC (Phase 6 — Current 🔶)

```text
🟥 P0: For loops, Generics, Error handling, Optional chaining, String interpolation
     → todas generan código end-to-end
🟧 P1: Defer, Guard, Type aliases, Dict/Map, Spread, Range, const fn
     → todas generan código end-to-end
🟦 P3: Standard library complete (collections, str, time, json)
🟪 P4: LSP completion + hover + go-to-definition
🟩 P5: 100+ integration tests, no crashes, CI pipeline
```

### v0.7.0 — RC (Phase 7)

```text
Cross-platform: macOS (Intel+ARM) + Linux (x64+ARM) + Windows (x64)
klc build + klc run funciona en las 3 plataformas
CI pipeline con todas las plataformas
```

### v1.0.0 — Stable

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
```

---

## Version

```text
Kyle Language Roadmap v5.0 — MVP Focus
Last updated: 2026-06-22
```
