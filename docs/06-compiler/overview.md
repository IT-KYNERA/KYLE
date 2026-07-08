# Compiler Overview

> Pipeline de compilación de Kyle. Describe cómo el código fuente `.ky` se transforma en binario nativo.

## Pipeline

```
Source (.ky)
    │
    ▼
[1] Lexer → Token stream
    │
    ▼
[2] Parser → AST (Abstract Syntax Tree)
    │
    ▼
[3] HIR → Desugared AST (High-Level IR)
    │
    ▼
[4] Semantic Analysis → Typed AST (type checking, scope resolution)
    │
    ▼
[5] MIR Lowering → Mid-Level IR
    │
    ▼
[6] Borrow Analysis → Ownership validation
    │
    ▼
[7] SSA Construction → SSA form (phi nodes, GVN)
    │
    ▼
[8] MIR Optimization → Constant folding, dead code elimination
    │
    ▼
[9] LLVM Codegen → LLVM IR
    │
    ▼
[10] LLVM Optimization → O3 pipeline (mem2reg, GVN, LICM, inlining)
    │
    ▼
[11] Object Emission → .o file
    │
    ▼
[12] Linking → Binary (linked with libkyc_runtime.a)
```

## Crate structure

| Crate | Purpose | Lines |
|-------|---------|-------|
| `kyc_core` | Foundation: AST types, diagnostics, spans | ~800 |
| `kyc_frontend` | Lexer + Parser | ~3,500 |
| `kyc_hir` | HIR desugaring | ~420 |
| `kyc_semantic` | Type checker, scope resolver, symbol table | ~2,200 |
| `kyc_mir` | MIR lowering, borrow analysis, SSA, optimize | ~10,500 |
| `kyc_backend` | LLVM codegen, linker | ~3,200 |
| `kyc_driver` | Pipeline orchestration | ~570 |
| `kyc_cli` | CLI binary (`ky`) | ~300 |
| `kyc_runtime` | Runtime static library (memory, strings, lists, dicts) | ~3,500 |
| `kyc_tools` | LSP server, formatter, package manager | ~5,000 |
| `kyc_platform` | Platform API (FS, net, time) — en desarrollo | ~500 |

## Orchestration

El pipeline es orquestado por `kyc_driver::pipeline::Pipeline`, que coordina cada fase:

```rust
// Simplified pipeline flow
fn compile(source: &str) -> Result<MirOutput, String> {
    let tokens = lexer::tokenize(source);      // [1] Lexer
    let ast = parser::parse(tokens);            // [2] Parser
    let hir = hir::desugar(ast);                // [3] HIR
    let typed_ast = semantic::analyze(hir);     // [4] Semantic
    let mir = mir::lower(typed_ast);            // [5] MIR
    mir::borrow_analysis::run(&mut mir);        // [6] Borrow
    mir::ssa::transform(&mut mir);              // [7] SSA
    mir::optimize(&mut mir);                    // [8] MIR Opt
    Ok(mir)
}

fn build(mir: MirOutput) -> Result<(), String> {
    let llvm_ir = backend::codegen::generate(&mir);  // [9] LLVM
    backend::optimize_module(&llvm_ir);              // [10] Opt
    backend::emit_object(&llvm_ir);                  // [11] Object
    backend::link(&object_files);                    // [12] Link
}
```

## Entry point

El punto de entrada es `kyc_cli::main` que delega en `Pipeline::build_source()`.

### build_source

```rust
pub fn build_source(source: &str, file_name: &str, output: &Path) -> Result<(), String> {
    // 1. Compile .ky → MIR (lexer → parser → HIR → semantic → MIR → borrow → SSA → optimize)
    let mir_output = Self::compile(source)?;
    
    // 2. Generate LLVM IR from MIR, optimize, emit object, link
    let ir = compile_function(&mir_output)?;
    optimize_module(ir);
    emit_object(ir, output)?;
    link(objects, output, runtime_lib, false, links)
}
```

## Ver también

- `01-introduction/architecture.md` — Visión general del ecosistema Kyle
- Cada fase del pipeline tiene su propio documento en `06-compiler/`
