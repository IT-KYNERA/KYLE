# Kyle — AI Agent Context

> **Read this first.** Single entry-point for AI agents working on the Kyle codebase.

## Quick start

```bash
cargo build --release --bin ky
cargo test --workspace
ky run examples/hello.ky
```

## Project structure

```
ky/
├── crates/                    → Rust crates (compiler)
│   ├── kyc_core/              → AST types, diagnostics, span
│   │   └── src/ast/           → Expressions, Statements, Declarations (modular)
│   ├── kyc_frontend/          → Lexer + Parser
│   │   └── src/parser/        → Parsing (modular)
│   ├── kyc_hir/               → HIR desugaring
│   ├── kyc_semantic/          → Type checker, scope, borrow analysis
│   │   └── src/type_checker/  → Type inference (modular)
│   ├── kyc_mir/               → MIR lowering, SSA, optimizations
│   │   └── src/lower/         → AST→MIR lowering (modular)
│   ├── kyc_backend/           → LLVM codegen, linker
│   │   └── src/codegen/       → LLVM IR generation (modular)
│   ├── kyc_driver/            → Compilation pipeline
│   ├── kyc_cli/               → CLI binary (`ky`)
│   ├── kyc_runtime/           → Runtime library (Rust)
│   ├── kyc_platform/          → Platform API (fs, time)
│   └── kyc_ui/                → UI framework
├── packages/                  → Official Kyle packages (env, http, sqlite, ui)
├── registry/                  → Published package registry
├── tools/
│   ├── vscode-extension/      → VS Code extension
│   └── scripts/               → install.sh, install.ps1
├── docs/                      → Full documentation
├── benchmarks/                → Performance benchmarks
├── examples/                  → Example .ky projects
├── tests/                     → End-to-end type-check tests
└── runtimes/                  → Alternative runtimes (js, ky)
```

## Architecture

The compiler pipeline follows a layered architecture:
```
source → kyc_frontend → kyc_hir → kyc_semantic → kyc_mir → kyc_backend → binary
```

Each phase is a separate crate with internal module structure.
