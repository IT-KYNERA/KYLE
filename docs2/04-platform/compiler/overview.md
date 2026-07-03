# Compiler Overview

## Pipeline

```
Source (.ky)
    │
    ▼
Lexer → Token stream
    │
    ▼
Parser → AST
    │
    ▼
HIR → Desugared AST
    │
    ▼
Semantic Analysis → Typed AST
    │
    ▼
MIR Lowering → Mid-Level IR
    │
    ▼
Borrow Analysis → Ownership check
    │
    ▼
SSA Construction → SSA form
    │
    ▼
LLVM Codegen → LLVM IR
    │
    ▼
Optimization → O3 passes
    │
    ▼
Linking → Native binary (+ runtime)
```

## Crates

| Crate | Purpose |
|-------|---------|
| `kyc_core` | Foundation: AST, types, diagnostics |
| `kyc_frontend` | Lexer + parser |
| `kyc_hir` | HIR desugaring |
| `kyc_semantic` | Type checker, scope, borrow analysis |
| `kyc_mir` | MIR definition, lowering, SSA |
| `kyc_backend` | LLVM codegen + linker |
| `kyc_driver` | Pipeline orchestration |
| `kyc_cli` | CLI binary |
| `kyc_runtime` | Runtime static library |
| `kyc_tools` | LSP, formatter, package manager |
