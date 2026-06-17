# KL Project Architecture Specification v1.0

---

## Workspace Structure

KL uses a Cargo-inspired workspace structure.

```text
kl/
├── Cargo.toml              # Rust workspace root
├── kl.toml                 # KL project configuration
├── .gitignore
├── README.md
│
├── crates/
│   ├── klc_core/           # Core types, AST, Span
│   ├── klc_frontend/       # Lexer, Parser
│   ├── klc_semantic/       # Type checker, resolver
│   ├── klc_mir/            # Mid-level IR, optimizer
│   ├── klc_backend/        # LLVM codegen, linker
│   ├── klc_driver/         # Pipeline orchestration
│   ├── klc_cli/            # CLI binary
│   ├── klc_runtime/        # GC, async, panic handling
│   └── klc_tools/          # LSP, formatter, debugger
│
├── runtime/                # KL runtime source (Rust)
│   ├── memory/
│   ├── async/
│   ├── collections/
│   └── io/
│
├── std/                    # Standard library (KL source)
│   ├── core/
│   ├── math/
│   ├── json/
│   ├── io/
│   ├── net/
│   ├── time/
│   ├── filesystem/
│   ├── collections/
│   ├── crypto/
│   └── testing/
│
├── examples/               # Example KL programs
│
├── tests/                  # Compiler tests
│
├── benchmarks/             # Performance benchmarks
│
├── docs/                   # Language documentation
│
└── tools/                  # Developer tooling
```

---

## Crate Responsibilities

### klc_core

```text
Purpose: Foundation types used by all crates
Depends on: nothing

Contents:
- AST node definitions
- Span (source location)
- SourceMap
- Symbol IDs
- Diagnostic types
- Type representations
```

### klc_frontend

```text
Purpose: Parse KL source into AST
Depends on: klc_core

Contents:
- Lexer (tokenizer, indentation handler)
- Parser (recursive descent)
- Grammar validation
```

### klc_semantic

```text
Purpose: Type check and resolve symbols
Depends on: klc_core

Contents:
- Symbol table
- Scope resolver
- Type checker (inference, unification)
- Module resolver
- Generic monomorphizer
- Contract validator
```

### klc_mir

```text
Purpose: Optimize and transform AST
Depends on: klc_core

Contents:
- MIR definition
- AST to MIR lowering
- Optimization passes
- Control flow analysis
```

### klc_backend

```text
Purpose: Generate native code via LLVM
Depends on: klc_core

Contents:
- LLVM IR generation
- LLVM optimization passes
- Object file generation
- Linker integration
- Debug info generation
```

### klc_driver

```text
Purpose: Orchestrate the compilation pipeline
Depends on: all other crates

Contents:
- Pipeline orchestration
- Build system
- Cache management
- Configuration parsing
```

### klc_cli

```text
Purpose: Command-line interface
Depends on: klc_driver

Contents:
- CLI argument parsing
- Command handlers
- Terminal output
- Progress reporting
```

### klc_runtime

```text
Purpose: Runtime support for compiled KL programs
Depends on: libc, Boehm GC

Contents:
- Garbage collector wrapper
- Async executor (work-stealing scheduler)
- Task system
- Channel implementation
- Mutex and atomics
- Error type support
- Panic handler
```

### klc_tools

```text
Purpose: Developer tooling
Depends on: klc_core, klc_frontend

Contents:
- Language Server Protocol implementation
- Code formatter
- Debugger integration
- Code completion
- Diagnostics
- Refactoring tools
```

---

## Internal Crate Dependencies

```text
klc_cli
  └── klc_driver
        ├── klc_frontend
        │     └── klc_core
        ├── klc_semantic
        │     └── klc_core
        ├── klc_mir
        │     └── klc_core
        └── klc_backend
              └── klc_core
```

---

## Standard Library Structure

```text
std/
├── core/           # Built-in types, runtime
├── io/             # Console, streams
├── math/           # Math functions, constants
├── json/           # Serialization, deserialization
├── net/            # HTTP client, networking
├── time/           # Time, duration, sleep
├── filesystem/     # File read, write, directory ops
├── collections/    # List, dict, set, queue, stack
├── crypto/         # Hashing, UUID, encryption
├── async/          # Task, channel, sync primitives
└── testing/        # Test runner, assertions
```

---

## Runtime Structure

```text
runtime/
├── gc.rs           # Garbage collector wrapper
├── async.rs        # Async executor, scheduler
├── task.rs         # Task type, state machine
├── channel.rs      # Channel implementation
├── mutex.rs        # Mutex for async
├── atomic.rs       # Atomic types
├── error.rs        # Error type implementation
├── panic.rs        # Panic handler
└── lib.rs          # Runtime entry point
```

---

## Development Workflow

### Build

```bash
kl build
```

### Run

```bash
kl run main
```

### Test

```bash
kl test
```

### Format

```bash
kl fmt
```

### Lint

```bash
kl check
```

---

## Project Configuration

File: `kl.toml`

```toml
name = "my_app"
version = "1.0.0"
edition = "1"
authors = ["Kynera"]
license = "MIT"

[compiler]
optimization = "O2"
target = "native"

[dependencies]
web = "1.0"
json = "2.0"

[dev-dependencies]
testing = "1.0"
```

---

## Version

```text
KL Project Architecture Specification v1.0
```
