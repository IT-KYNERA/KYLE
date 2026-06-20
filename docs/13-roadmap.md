# Kyle Language Roadmap v2.0 — RAII Memory Model

---

## Overview

Kyle is developed in 6 phases. Phases 1–3 (compiler pipeline) are complete. Phase 4 (Runtime + Std Library) is the current focus. Each phase builds on the previous, adding features, stability, and tooling.

**Memory model:** RAII + Compiler-Inferred Ownership (NO garbage collector).

---

## Phase 0: Language Design

### Status: Complete ✅

All 16 specification documents are written and frozen. The language syntax, type system, memory model, and architecture are fully defined.

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
[x] All 16 documents frozen, consistent, and finalized
```

---

## Phase 1: Compiler Frontend

### Status: Complete ✅

### Deliverables

```text
Lexer (427 lines, 69 tests) ✅
Parser (812 lines, recursive descent, indent-based) ✅
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
MIR definition + lowering + optimizer (1093 lines, 2 tests) ✅
LLVM codegen via inkwell 0.9 / LLVM 18.1 (310 lines) ✅
Native linker via clang ✅
CLI: klc build <file.kl> → native binary ✅
CLI: klc run <file.kl> → compile + execute ✅
CLI: klc mir <file.kl> → MIR dump ✅
All examples compile and run (hello, fibonacci, user) ✅
118 tests passing (69 frontend + 47 semantic + 2 MIR) ✅
```

### Tasks

```text
[x] MIR definition
    - MirValue, MirConstant, MirType, MirInst, MirTerminator
    - MirBasicBlock, MirFunction, MirModule
    - Display impls for all MIR types (312 lines)
[x] AST → MIR lowering
    - LowerCtx with locals, blocks, block_counter
    - All statements: variable, return, if/elif/else, while, for, match, defer, guard, unsafe, break, binding-if, constant, typed-variable
    - All expressions: literal, identifier, binary, unary, call, assignment, property access, optional chain, error prop, list, dict, tuple, closure, await, async, spread, range-slice, loop
    - Constructor lowering for classes (601 lines)
[x] MIR optimizer
    - Constant folding (int add/sub/mul)
    - Dead code elimination (unused instructions)
    - Remove unreachable basic blocks
    - 2 unit tests (180 lines)
[x] LLVM codegen
    - inkwell 0.9 integration, LLVM 18.1, opaque pointers
    - Type mapping (MIR → LLVM types)
    - Alloca/Store/Load for locals
    - BinaryOp (add, sub, mul, div, rem, and, or, xor, shl, shr, comparisons)
    - UnaryOp (neg, not, bitnot)
    - Call with argument mapping
    - Basic block building with terminators (br, condbr, ret, unreachable)
    - TargetMachine for native object file emission (310 lines)
[x] Native linker
    - clang-based linking of .o → binary
    - Shared library support (link_shared)
[x] Pipeline orchestration
    - Source → MIR → LLVM → .o → binary (end-to-end)
    - Check + MIR + Build subcommands
```

### Remaining backend work (moved to Phase 4)

```text
[ ] RAII ownership inference pass (determine stack vs move vs refcount)
[ ] Full struct codegen (LLVM struct types, field access, methods)
[ ] Array/List codegen with runtime support
[ ] String codegen with runtime support (ptr + len)
[ ] Pattern matching codegen (switch + branch)
[ ] Codegen for: Closure, Await, Async, PropertyAccess, OptionalChain, RangeSlice
[ ] LLVM optimization passes (O0/O1/O2/O3)
[ ] Debug info (DWARF)
```

---

## Phase 4: Runtime + Standard Library

### Status: Complete ✅

### Goal

Build the RAII runtime and standard library so Kyle programs can do real I/O, use strings, math, collections, and async.

### Milestone

```text
kl run examples/hello.kl   → "Hello, World!"     ✅
klc run string_test.kl     → string ops work     ✅
```

### Tasks

#### 4.1 — Runtime (Rust)

```text
[x] Implement core runtime crate (klc_runtime)
    - print/println wrappers (write to stdout) ✅
    - str representation (ptr + length, UTF-8) ✅
    - heap allocation wrappers (malloc/free for RAII) ✅
    - program entry point (_start → main wrapper) ✅
    - exit code handling ✅
    - String ops: contains, to_upper, to_lower, trim, replace, concat, input ✅
    - Char ops: char_at, is_digit, is_alpha, is_alnum, is_whitespace, is_upper, is_lower, ord ✅
    - File I/O: open, read_str, write_str, close ✅
    - Time: sleep(ms), now() -> i64 ✅
[~] Link runtime with klc build
    - Compile runtime as .a (static library) ✅
    - Pass -L and -l flags to clang linker (manual: klc build --runtime) 🔶
```

#### 4.2 — RAII Ownership Inference

```text
[x] RAII ownership inference pass (klc_mir/src/ownership.rs) ✅
    - Escape analysis: which values escape the current scope ✅
    - Move inference: single-owner values use memcpy (zero-cost) ✅
    - Refcount inference: shared values get Arc/Rc wrappers ✅
    - Insert retain/release instructions at scope exits ✅
```

#### 4.3 — Compiler String/Char Support

```text
[x] Builtin functions registered in symbol_table, lower, codegen ✅
    - print, println, input, len, str, range, char_at ✅
    - is_digit, is_alpha, is_alnum, is_whitespace, is_upper, is_lower, ord ✅
    - contains, to_upper, to_lower, trim, replace ✅
    - open, read_str, write_str, close ✅
    - sleep, now ✅
[x] String return from user functions (fn_returns map) ✅
[x] String concat result type (MirType::Str) ✅
[x] str() cast i32→i64 before kl_i64_to_str ✅
[x] len() returns I32 ✅
[x] Inference variable type (local_types map) ✅
```

#### 4.4 — Standard Library

```text
[x] std/core.kl: utility functions ✅
[x] std/math.kl: abs, pow, sqrt, gcd ✅
[x] std/io.kl: I/O wrappers ✅
[x] std/testing.kl: assert, assert_eq, assert_str ✅
[~] Collections (List, Map, Set) — pendiente de runtime arrays 🔶
[~] Module resolution for stdlib paths — imports desde .kl ✅
```

#### 4.5 — Async Runtime

```text
[x] Async runtime (klc_runtime/src/async_.rs) ✅
    - Work-stealing thread pool ✅
    - Task<T> with Future poll mechanism ✅
    - Channel<T> with send/recv ✅
    - Timer/sleep support ✅
[~] Compiler async lowering — codegen pendiente 🔶
```

---

## Phase 5: Tooling & Ecosystem

### Status: Complete ✅

### Goal

Build developer tooling: package manager, LSP, formatter, debugger.

### Milestone

```text
kl new my-project       ← Q3 2026 ✅
kl add json             ← Q3 2026 ✅
IDE support (LSP)       ← Q3 2026 ✅
```

### Tasks

```text
[x] Package manager (klc_tools)
    - kl new: project scaffolding ✅
    - kl init: alias de new ✅
    - kl add: add dependency @version ✅
    - kl remove: remove dependency ✅
    - kl info: show package info ✅
    - kl build: compila src/main.kl desde proyecto ✅
    - kl run: compila y ejecuta desde proyecto ✅
    - kl test: ejecuta tests desde proyecto ✅
    - Manifest (kl.toml): serde + read/write ✅
    - Lock file: serde + read/write ✅
    - Project helper: find_project_root, source paths ✅
[x] Language server (LSP) — klc_tools/src/lsp.rs ✅
    - textDocument/documentSymbol ✅
    - workspace/symbol ✅
    - textDocument/signatureHelp ✅
    - textDocument/findReferences ✅
    - textDocument/codeAction (E0009 → create var / import) ✅
    - Server capabilities: references_provider + code_action_provider ✅
[x] Code formatter — klc_tools/src/formatter.rs ✅
    - AST pretty-printer (all nodes) ✅
    - Comment preservation (via AST spans + last_comment_line) ✅
    - klc fmt <file.kl> command ✅
[x] IDE support
    - VS Code extension (vscode-kl/) ✅
    - Syntax highlighting (TextMate grammar) ✅
    - Language config (comments, brackets, auto-closing, indentation) ✅
    - LSP client (launches klc lsp) ✅
    - Commands: kl.run, kl.build, kl.check ✅
[ ] Debugger integration
    - DWARF debug info generation 🔶
    - LLDB / GDB support 🔶
[ ] Documentation generator
    - kl doc: generate HTML docs 🔶
```

---

## Phase 6: Self-Hosting

### Status: 🔄 Current — In Progress

### Goal

Rewrite the Kyle compiler in Kyle itself. Postponed until the language is stable and the compiler is feature-complete.

### Milestone

```text
kl build klc   # compiler compiles itself  ← 2026 Q4
```

### Tasks

```text
[x] Write lexer in Kyle — examples/lexer.kl ✅ (200+ lines, tokeniza correctamente)
[x] Fix compiler bugs for self-hosting
    - if_then block naming collision → fresh_block() ✅
    - elif chain block collision → elif_cond_labels vector ✅
    - string escape sequences → lex_string procesa \n, \t, \" etc. ✅
    - string return from user functions → fn_returns map ✅
    - string concat result type → MirType::Str ✅
    - Stmt::Break lowering → Br(loop_end) via break_targets stack ✅
[ ] Write parser in Kyle
[ ] Write semantic analyzer in Kyle
[ ] Write MIR lowering in Kyle
[ ] Write codegen in Kyle
[ ] Bootstrap and self-host
```

---

## Timeline

```text
Phase 0: Language Design        — Complete (Jun 2026)
Phase 1: Compiler Frontend      — Complete (Jul 2026)
Phase 2: Semantic Analysis      — Complete (Aug 2026)
Phase 3: Compiler Backend       — Complete (Sep 2026)
Phase 4: Runtime + Std Library  — Complete (Oct 2026)
Phase 5: Tooling & Ecosystem    — Complete (Nov 2026)
Phase 6: Self-Hosting           — In Progress (Nov 2026 – ...)
```

---

## Release Milestones

### v0.1.0 — Alpha (Complete ✅)

```text
Lexer + Parser working
AST dump available
```

### v0.2.0 — Alpha (Complete ✅)

```text
Type checker working
Semantic analysis complete
```

### v0.3.0 — Beta (Complete ✅)

```text
Code generation working
Native binaries produced
klc build + klc run functional
```

### v0.4.0 — Beta (Complete ✅)

```text
RAII runtime working ✅
Standard library basics ✅
Hello World → actual stdout output ✅
String ops, char ops, file I/O, time ✅
```

### v0.5.0 — Beta (Current 🔄)

```text
Async runtime working ✅
Package manager working ✅
Language server working ✅
Code formatter working ✅
VS Code extension working ✅
Self-hosting preparation 🔄
```

### v1.0.0 — Stable

```text
Production-ready compiler
Stable standard library
Full tooling support
Self-hosting ready
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
Kyle Language Roadmap v3.0
Last updated: 2026-11-19
```
