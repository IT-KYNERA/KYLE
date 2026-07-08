# Compiler Architecture

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
Semantic Analysis → Typed AST (type checking, scope)
    │
    ▼
MIR Lowering → Mid-Level IR
    │
    ▼
Borrow Analysis → Ownership check
    │
    ▼
SSA Construction → SSA form (phi nodes, GVN)
    │
    ▼
LLVM Codegen → LLVM IR
    │
    ▼
Optimization → O3 passes (mem2reg, GVN, LICM)
    │
    ▼
Linking → Native binary (+ runtime)
```

## Crates

| Crate | Purpose |
|-------|---------|
| `kyc_core` | Foundation: AST, types, diagnostics |
| `kyc_frontend` | Lexer and parser |
| `kyc_hir` | HIR desugaring |
| `kyc_semantic` | Type checker, scope, borrow analysis |
| `kyc_mir` | MIR definition, lowering, SSA construction |
| `kyc_backend` | LLVM codegen, linker |
| `kyc_driver` | Pipeline orchestration |
| `kyc_cli` | CLI binary |
| `kyc_runtime` | Runtime static library |
| `kyc_tools` | LSP, formatter, package manager |
