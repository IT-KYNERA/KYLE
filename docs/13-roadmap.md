# KL Language Roadmap v1.0

---

## Overview

The KL programming language is developed in 5 phases. Each phase builds on the previous, adding features, stability, and tooling.

---

## Phase 0: Language Design (Current)

### Status: Complete

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
[x] Define memory model
[x] Design compiler architecture
[x] Plan project structure
[x] Design package manager
[x] Create this roadmap
[x] Add integer overflow behavior
[x] Specify string encoding (UTF-8)
[x] Define entry point convention
[x] Add attributes/annotations syntax
[x] Add bitwise operators
[x] Specify casting and type alias rules
[x] Add FFI and unsafe blocks
[x] Add defer, guard, string interpolation
[x] Add spread/rest, range slicing, optional chaining
[x] Add operator overloading
[x] Add compile-time evaluation
[x] Add variadic functions, default/named params
[x] Add match guards, break with value
[x] Add while-else/for-else
[x] Add iterator protocol
[x] Create error catalog
[x] Create ABI specification
[x] Replace ^class with abs keyword
[x] Remove export keyword — naming-based visibility only
[x] Replace if let/while let with binding conditions
[x] Final syntax freeze — all 16 documents consistent
```

### Deliverables

```text
/docs directory with all 16 specifications
Complete language design document
Final frozen syntax ready for compiler implementation
Compiler architecture plan
Project structure plan
```

---

## Phase 1: Compiler Frontend

### Status: Not Started

### Goal

Build the compiler frontend: lexer, parser, and AST construction.

### Tasks

```text
[ ] Set up Rust workspace (klc_core, klc_frontend)
[ ] Implement lexer
    - Token types
    - Identifier handling
    - Literal parsing
    - Operator tokens
    - INDENT/DEDENT handling
[ ] Implement parser
    - Recursive descent
    - Expression parsing
    - Statement parsing
    - Declaration parsing
    - Error recovery
[ ] Build AST nodes
    - Program node
    - All declaration nodes
    - All expression nodes
    - All statement nodes
[ ] Write parser tests
    - Valid programs
    - Invalid programs
    - Edge cases
[ ] CLI integration (klc_cli)
    - File reading
    - Parse command
    - AST dump
```

### Tests

```text
Lexer unit tests
Parser unit tests
Integration tests
Golden file tests
```

### Milestone

```text
kl parse main.kl  # produces AST dump
```

---

## Phase 2: Semantic Analysis

### Status: Not Started

### Goal

Add type checking, symbol resolution, and module system.

### Tasks

```text
[ ] Implement symbol table
[ ] Implement scope resolver
[ ] Implement type resolver
    - Primitive types
    - User-defined types
    - Generic types
[ ] Implement type inference
    - Hindley-Milner basics
    - Constraint solving
    - Type unification
[ ] Implement module resolver
    - Import resolution
    - Module graph
    - Circular dependency detection
[ ] Implement generic monomorphization
[ ] Implement contract validation
[ ] Implement error safety validation
[ ] Implement optional safety validation
[ ] Implement diagnostics system
    - Error codes
    - Span-based error messages
    - Warnings
```

### Tests

```text
Type inference tests
Generic resolution tests
Error-handling tests
Contract validation tests
Module resolution tests
```

### Milestone

```text
kl check main.kl  # type-checks without producing binary
```

---

## Phase 3: Compiler Backend

### Status: Not Started

### Goal

Generate native machine code via LLVM.

### Tasks

```text
[ ] Integrate inkwell (LLVM bindings)
[ ] Implement LLVM IR generation
    - Type mapping
    - Function generation
    - Control flow
    - Memory operations
[ ] Implement MIR (optional)
    - AST lowering
    - Basic optimizations
[ ] Implement LLVM optimization passes
    - O0, O1, O2, O3
[ ] Generate object files
[ ] Implement linker integration
[ ] Implement runtime integration
    - GC wrapper
    - Panic handler
[ ] Implement debug info (DWARF)
[ ] Write codegen tests
    - Hello World
    - Fibonacci
    - Data structures
```

### Tests

```text
Codegen unit tests
Runtime integration tests
End-to-end compilation tests
```

### Milestone

```text
kl build main.kl     # produces native binary
./main               # runs the binary
```

---

## Phase 4: Standard Library & Runtime

### Status: Not Started

### Goal

Build the standard library and runtime.

### Tasks

```text
[ ] Implement core runtime
    - GC integration
    - Memory management
    - Type metadata
[ ] Build standard library
    - io (print, read)
    - math (all functions)
    - json (parse, stringify)
    - net (HTTP client)
    - time (timers, sleep)
    - filesystem (read, write, mkdir)
    - collections (list, dict, set)
    - crypto (hash, uuid)
[ ] Implement async runtime
    - Work-stealing scheduler
    - Task system
    - Channel implementation
    - Timer implementation
    - Async I/O
[ ] Implement testing framework
    - Test discovery
    - Assertion library
    - Benchmark runner
[ ] Write standard library tests
```

### Tests

```text
Standard library unit tests
Runtime tests
Async runtime tests
Integration tests
```

### Milestone

```text
kl run examples/hello.kl     # Hello World works
kl test                      # tests pass
kl run examples/web.kl       # async HTTP works
```

---

## Phase 5: Tooling & Ecosystem

### Status: Not Started

### Goal

Build developer tooling and ecosystem.

### Tasks

```text
[ ] Package manager
    - Registry implementation
    - Publish workflow
    - Dependency resolution
    - Lock file generation
[ ] Language server (LSP)
    - Code completion
    - Diagnostics (red squiggles)
    - Go-to-definition
    - Find references
    - Hover information
    - Code actions
[ ] Code formatter
    - KL code formatting
    - Configurable style
[ ] Debugger integration
    - LLDB support
    - Step-through debugging
    - Variable inspection
[ ] IDE support
    - VS Code extension
    - Syntax highlighting
    - Language server integration
[ ] Documentation generator
[ ] Benchmark suite
[ ] CI/CD templates
```

### Milestone

```text
Full IDE support
Package registry online
Community contributions possible
```

---

## Phase 6: Self-Hosting

### Status: Future

### Goal

Rewrite the KL compiler in KL itself.

### Tasks

```text
[ ] Write lexer in KL
[ ] Write parser in KL
[ ] Write semantic analyzer in KL
[ ] Write codegen in KL
[ ] Bootstrap: compile KL compiler with Rust version
[ ] Verify: compile KL compiler with KL version
[ ] Release: KL compiler written in KL
```

### Milestone

```text
kl build klc  # compiler compiles itself
```

---

## Timeline

```text
Phase 0: Language Design     - Complete (Jun 2026)
Phase 1: Compiler Frontend   - Q3 2026 (Aug-Oct)
Phase 2: Semantic Analysis   - Q4 2026 (Nov-Dec)
Phase 3: Compiler Backend    - Q1 2027 (Jan-Mar)
Phase 4: Std Library/Runtime - Q2 2027 (Apr-Jun)
Phase 5: Tooling & Ecosystem - Q3-Q4 2027 (Jul-Dec)
Phase 6: Self-Hosting        - 2028
```

---

## Release Milestones

### v0.1.0 — Alpha

```text
Lexer + Parser working
AST dump available
```

### v0.2.0 — Alpha

```text
Type checker working
Semantic analysis complete
```

### v0.3.0 — Beta

```text
Code generation working
Hello World compiles
```

### v0.4.0 — Beta

```text
Standard library working
Async runtime working
```

### v0.5.0 — Beta

```text
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
KL Language Roadmap v1.0
```
