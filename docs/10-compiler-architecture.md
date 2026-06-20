# Kyle Compiler Architecture Specification v1.0

---

## Overview

The Kyle compiler transforms `.kl` source files into native machine code.

### Pipeline

```text
Source Code (.kl)
    │
    ▼
Lexer                          (klc_frontend)
    │
    ▼
Tokens
    │
    ▼
Parser                         (klc_frontend)
    │
    ▼
AST
    │
    ▼
Semantic Analyzer              (klc_semantic)
    │
    ▼
Type Checker                   (klc_semantic)
    │
    ▼
MIR Lowering                   (klc_mir/lower.rs)
    │
    ▼
MIR Optimizer                  (klc_mir/optimize.rs)
    │
    ▼
Ownership Inference            (klc_mir/ownership.rs)
    │
    ▼
LLVM Codegen                   (klc_backend/codegen.rs)
    │
    ▼
LLVM Optimizer                 (LLVM passes)
    │
    ▼
Native Binary                  (klc_backend/linker.rs)
```

---

## Compiler Components

### 1. Lexer (klc_frontend)

Input: Source code string
Output: Token stream

```text
Reads characters
Groups into tokens
Tracks position (line, column)
Handles indentation (INDENT/DEDENT)
Ignores comments
```

### 2. Parser (klc_frontend)

Input: Token stream
Output: AST

```text
Recursive descent parser
Indentation-aware
Produces strongly-typed AST nodes
Error recovery for diagnostics
```

### 3. Semantic Analyzer (klc_semantic)

Input: AST
Output: Annotated AST

```text
Scope resolution
Symbol table construction
Module resolution
Import validation
Name binding
This resolution (class/instance context)
```

### 4. Type Checker (klc_semantic)

Input: Annotated AST
Output: Type-checked AST

```text
Type inference
Type unification
Generic monomorphization
Contract validation
Error safety validation
Optional safety validation
```

### 5. MIR Lowering (klc_mir/lower.rs)

Input: Type-checked AST
Output: MIR (Mid-level IR)

```text
Lower AST to MIR (MirModule, MirFunction, MirBasicBlock)
Control flow graph construction
Builtin function name remapping
String local tracking
Break/continue target resolution
```

### 6. MIR Optimizer (klc_mir/optimize.rs)

Input: MIR
Output: Optimized MIR

```text
Constant folding
Dead code elimination (collect_terminator_refs)
Remove unreachable basic blocks
```

### 6b. Ownership Inference (klc_mir/ownership.rs)

Input: Optimized MIR
Output: Ownership-annotated MIR

```text
Escape analysis
Move inference (memcpy for single-owner)
Refcount inference (Arc/Rc for shared)
Insert retain/release at scope exits
```

### 7. LLVM Codegen (klc_backend)

Input: Optimized MIR
Output: LLVM IR

```text
Translate MIR to LLVM instructions
Handle RAII runtime integration (destructors, refcount)
Handle async state machines
Generate debug info
```

### 8. LLVM Optimizer (klc_backend)

Input: LLVM IR
Output: Optimized LLVM IR

```text
LLVM optimization passes
-O0, -O1, -O2, -O3, -Os
Target-specific optimizations
```

### 9. Linker (klc_backend)

Input: Object files + runtime
Output: Native binary

```text
Link with klc_runtime
Link with libc
Produce executable or library
```

---

## Compiler Crate Diagram

```text
┌─────────────────────────────────────────────────────────────┐
│                     klc_cli (Binary)                        │
│          build | run | test | fmt | lsp | parse             │
└────────┬──────────┬──────────┬──────────────────────────────┘
         │          │          │
┌────────▼──────────▼──────────▼──────────────────────────────┐
│                     klc_driver                               │
│                Pipeline Orchestration                        │
└────┬──────┬──────┬──────┬──────┬──────┬──────┬──────────────┘
     │      │      │      │      │      │      │
┌────▼──┐ ┌▼─────┐ ┌▼────┐ ┌▼───┐ ┌▼───┐ ┌▼────┐ ┌▼──────────┐
│Lexer  │ │Parser│ │Sem  │ │Type│ │ MIR│ │LLVM │ │Linker     │
│       │ │      │ │Ana  │ │Chk │ │    │ │CG   │ │           │
└───┬───┘ └──┬───┘ └──┬──┘ └──┬─┘ └──┬─┘ └──┬──┘ └──┬───────┘
    │        │        │       │      │      │       │
    └────────┴────────┴───────┴──────┴──────┴───────┘
                        │
              ┌─────────▼─────────┐   ┌──────────────────────┐
              │    klc_core       │   │    klc_tools         │
              │ AST, Span, Types, │   │ LSP, Formatter,      │
              │ SourceMap, Diag   │   │ Package Manager      │
              └───────────────────┘   └──────────────────────┘
              ┌───────────────────┐
              │   klc_runtime     │
              │ String, I/O, Time │
              │ Async, RAII       │
              └───────────────────┘
```

---

## Dependency Graph

```text
klc_cli → klc_driver
klc_cli → klc_tools          (lsp, fmt subcommands)

klc_driver → klc_frontend
klc_driver → klc_semantic
klc_driver → klc_mir
klc_driver → klc_backend
klc_driver → klc_runtime

klc_frontend → klc_core
klc_semantic → klc_core
klc_mir      → klc_core
klc_backend  → klc_core
klc_tools    → klc_core
klc_tools    → klc_frontend   (formatter needs AST)
```

---

## Compilation Modes

### Debug Mode

```text
-O0
No optimizations
Full debug info
Bounds checking enabled
Fast compilation
```

### Release Mode

```text
-O2
Full optimizations
No debug info
Bounds checking disabled
Fast execution
```

### Small Mode

```text
-Os
Optimize for binary size
Slower execution
Smaller binaries
```

---

## Incremental Compilation

The compiler caches:

```text
Module AST
Type information
Optimized MIR
Object files
```

Only changed modules are recompiled.

---

## Error Reporting

```text
Span tracking for every token
Human-readable messages
Error codes
Suggestions for fixes
Warning system
```

---

## Compiler Configuration

File: `kl.toml`

```toml
name = "my_project"
version = "1.0.0"
edition = "1"

[compiler]
optimization = "O2"
target = "native"
debug = false

[dependencies]
web = "1.0"
```

---

## Compiler Version

```text
Kyle Compiler Architecture Specification v2.0
Last updated: 2026-11-19
```
