# KL Compiler Architecture Specification v1.0

---

## Overview

The KL compiler transforms `.kl` source files into native machine code.

### Pipeline

```text
Source Code (.kl)
    │
    ▼
Lexer
    │
    ▼
Tokens
    │
    ▼
Parser
    │
    ▼
AST
    │
    ▼
Semantic Analyzer
    │
    ▼
Type Checker
    │
    ▼
IR (Intermediate Representation)
    │
    ▼
Optimizer
    │
    ▼
LLVM IR
    │
    ▼
LLVM Optimizer
    │
    ▼
Machine Code
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

### 5. IR Generator (klc_mir)

Input: Type-checked AST
Output: MIR (Mid-level IR)

```text
Lower AST to simpler IR
Control flow graph construction
Basic block analysis
```

### 6. Optimizer (klc_mir)

Input: MIR
Output: Optimized MIR

```text
Constant folding
Dead code elimination
Inlining
Escape analysis
Loop optimizations
```

### 7. LLVM Codegen (klc_backend)

Input: Optimized MIR
Output: LLVM IR

```text
Translate MIR to LLVM instructions
Handle GC integration
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

## Compiler Architecture Diagram

```text
┌─────────────────────────────────────────────────────────┐
│                     klc_cli (Binary)                     │
│                  build | run | test                      │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                    klc_driver                            │
│              Pipeline Orchestration                      │
└────┬──────┬──────┬──────┬──────┬──────┬──────┬──────────┘
     │      │      │      │      │      │      │
┌────▼──┐ ┌▼─────┐ ┌▼────┐ ┌▼───┐ ┌▼───┐ ┌▼────┐ ┌▼─────┐
│Lexer  │ │Parser│ │Sem  │ │Type│ │ IR │ │LLVM │ │Linker│
│       │ │      │ │Ana  │ │Chk │ │Gen │ │CG   │ │      │
└───┬───┘ └──┬───┘ └──┬──┘ └──┬─┘ └──┬─┘ └──┬──┘ └──┬───┘
    │        │        │       │      │      │       │
    └────────┴────────┴───────┴──────┴──────┴───────┘
                        │
              ┌─────────▼─────────┐
              │    klc_core       │
              │ AST, Span, Types  │
              └───────────────────┘
```

---

## Dependency Graph

```text
klc_cli → klc_driver

klc_driver → klc_frontend
klc_driver → klc_semantic
klc_driver → klc_mir
klc_driver → klc_backend
klc_driver → klc_runtime

klc_frontend → klc_core
klc_semantic → klc_core
klc_mir      → klc_core
klc_backend  → klc_core
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
KL Compiler Architecture Specification v1.0
```
