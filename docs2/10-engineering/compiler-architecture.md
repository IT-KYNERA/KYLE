# Compiler Architecture

## Pipeline

```
Source (.ky)
    │
    ▼
Lexer ──► Token stream
    │
    ▼
Parser ──► AST
    │
    ▼
HIR ──► Desugared AST (T? → Option<T>, etc.)
    │
    ▼
Semantic Analysis ──► Typed AST (type checking, scope resolution)
    │
    ▼
MIR Lowering ──► Mid-Level IR
    │
    ▼
Borrow Analysis ──► Ownership check
    │
    ▼
SSA Construction ──► SSA Form (promoted allocas, phi nodes)
    │
    ▼
LLVM Codegen ──► LLVM IR
    │
    ▼
Optimization ──► O3 passes (mem2reg, GVN, LICM, SCCP)
    │
    ▼
Linking ──► Native binary (+ libkyc_runtime.a)
```

## Crates

| Crate | Purpose |
|-------|---------|
| `kyc_core` | Foundation: AST types, diagnostics, source maps |
| `kyc_frontend` | Lexer + parser |
| `kyc_hir` | HIR desugaring |
| `kyc_semantic` | Type checker, scope resolver, borrow analysis |
| `kyc_mir` | MIR definition, lowering from AST, SSA construction |
| `kyc_backend` | LLVM codegen (via inkwell), linker |
| `kyc_driver` | Pipeline orchestration |
| `kyc_cli` | CLI binary |
| `kyc_runtime` | Runtime static library (alloc, strings, lists, dicts) |
| `kyc_tools` | LSP, formatter, package manager |
