# KL Language Roadmap v2.0 — RAII Memory Model

---

## Overview

KL is developed in 6 phases. Phases 1–3 (compiler pipeline) are complete. Phase 4 (Runtime + Std Library) is the current focus. Each phase builds on the previous, adding features, stability, and tooling.

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

### Status: 🔄 Current — In Progress

### Goal

Build the RAII runtime and standard library so KL programs can do real I/O, use strings, math, collections, and async. This phase unblocks useful programs.

### Milestone

```text
kl run examples/hello.kl   → "Hello, World!"     ← Q3 2026
kl run examples/math.kl    → sqrt(2) = 1.414...   ← Q4 2026
kl test                     → all tests pass      ← Q4 2026
kl run examples/server.kl  → async HTTP works     ← Q1 2027
```

### Tasks

#### 4.1 — Runtime Mínimo (Rust)

```text
[ ] Implement core runtime crate (klc_runtime)
    - print/println wrappers (write to stdout)
    - str representation (ptr + length, UTF-8)
    - heap allocation wrappers (malloc/free for RAII)
    - program entry point (_start → main wrapper)
    - exit code handling
[ ] Link runtime with klc build
    - Compile runtime as .a (static library)
    - Pass -L and -l flags to clang linker
    - Runtime auto-initialization (constructor attribute)
```

#### 4.2 — RAII Ownership Inference

```text
[ ] Design RAII inference pass (klc_backend/src/raii.rs)
    - Escape analysis: which values escape the current scope
    - Move inference: single-owner values use memcpy (zero-cost)
    - Refcount inference: shared values get Arc/Rc wrappers
    - Cycle detection warning
[ ] Implement in MIR-to-MIR transform
    - Annotate MirValue with ownership (Owned, Shared, Ref)
    - Insert retain/release instructions at scope exits
    - Generate destructor calls for owned types
```

#### 4.3 — Codegen Completo

```text
[ ] Struct codegen
    - LLVM struct types (not i32 dummy)
    - Field access via GEP
    - Constructor codegen
    - Method dispatch (vtable for classes)
[ ] Array/List codegen
    - Runtime array struct (ptr + len + capacity)
    - Indexing, iteration, push/pop
[ ] String codegen
    - Runtime str struct (ptr + len)
    - Concatenation, slicing, comparison
    - print/println runtime functions
[ ] Pattern matching codegen
    - Enum discriminant switch
    - Destructuring bindings
    - Guard conditions
[ ] Remaining expression codegen
    - PropertyAccess, OptionalChain, RangeSlice
    - Closure (heap-allocated captures)
    - Await/Async (state machine generation)
```

#### 4.4 — Standard Library

```text
[ ] Core stdlib (std/core/)
    - math.kl: abs, min, max, sqrt, sin, cos, pow, floor, ceil, round
    - io.kl: print, println, read_line, read_file, write_file
    - json.kl: parse, stringify
    - time.kl: now, sleep, timer
    - collections.kl: List<T>, Map<K,V>, Set<T>
[ ] Module resolution for stdlib paths
    - Resolve import math → std/core/math.kl
    - Stdlib search path (--stdlib flag)
```

#### 4.5 — Async Runtime

```text
[ ] Async runtime implementation
    - Work-stealing thread pool
    - Task<T> with Future poll mechanism
    - Channel<T> with send/recv
    - Timer/sleep support
    - Async I/O (non-blocking file/network)
[ ] Codegen for async
    - State machine lowering for async fns
    - Await transform (yield to executor)
```

### Tests

```text
[ ] Runtime unit tests (Rust)
[ ] Stdlib unit tests (KL)
[ ] Async runtime tests
[ ] End-to-end compilation tests
    - Hello World → stdout check
    - Math computation → result check
    - File I/O → content check
```

---

## Phase 5: Tooling & Ecosystem

### Status: Not Started

### Goal

Build developer tooling: package manager, LSP, formatter, debugger.

### Milestone

```text
kl new my-project   ← Q3 2027
kl add json         ← Q3 2027
IDE support (LSP)   ← Q4 2027
```

### Tasks

```text
[ ] Package manager (klc_tools)
    - kl new: project scaffolding
    - kl add: add dependency
    - kl build --release: optimized build
    - Registry API (publish, search)
    - Lock file + dependency resolution
[ ] Language server (LSP)
    - Code completion
    - Diagnostics (red squiggles in real time)
    - Go-to-definition
    - Find references
    - Hover type information
    - Code actions (quick fixes)
[ ] Code formatter
    - Indentation-aware formatting
    - Configurable style (max line length, etc.)
[ ] Debugger integration
    - DWARF debug info generation
    - LLDB / GDB support
    - Step-through, variable inspection
[ ] IDE support
    - VS Code extension
    - Syntax highlighting (TextMate grammar)
    - LSP client integration
[ ] Documentation generator
    - kl doc: generate HTML docs from source
    - Doc comments (## style)
```

---

## Phase 6: Self-Hosting

### Status: Future

### Goal

Rewrite the KL compiler in KL itself.

### Milestone

```text
kl build klc   # compiler compiles itself  ← 2028
```

### Tasks

```text
[ ] Write lexer in KL
[ ] Write parser in KL
[ ] Write semantic analyzer in KL
[ ] Write MIR lowering in KL
[ ] Write codegen in KL
[ ] Bootstrap: compile KL compiler with Rust version
[ ] Self-host: compile KL compiler with KL version
[ ] Release: KL compiler is self-hosting
```

---

## Timeline

```text
Phase 0: Language Design        — Complete (Jun 2026)
Phase 1: Compiler Frontend      — Complete (Jul 2026)
Phase 2: Semantic Analysis      — Complete (Aug 2026)
Phase 3: Compiler Backend       — Complete (Sep 2026)
Phase 4: Runtime + Std Library  — Q3 2026 – Q1 2027
Phase 5: Tooling & Ecosystem    — Q3 2027 – Q4 2027
Phase 6: Self-Hosting           — 2028
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

### v0.4.0 — Beta (Current 🔄)

```text
RAII runtime working
Standard library basics
Hello World → actual stdout output
```

### v0.5.0 — Beta

```text
Async runtime working
Package manager working
Language server working
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
KL Language Roadmap v2.0
Last updated: 2026-09-18
```
