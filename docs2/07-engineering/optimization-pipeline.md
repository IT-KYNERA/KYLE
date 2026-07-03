# Optimization Pipeline

## LLVM Optimization Passes

After code generation, KKyle runs the full LLVM optimization pipeline:

```rust
// pipeline.rs
let pipeline = match optimization {
    OptimizationLevel::Aggressive => "default<O3>",
    OptimizationLevel::Default => "default<O2>",
    _ => "default<O1>",
};
module.run_passes(pipeline, &tm, opts);
```

## Included passes

The `default<O3>` pipeline includes:
- **mem2reg** — promotes allocas to SSA registers
- **GVN** — Global Value Numbering (redundancy elimination)
- **LICM** — Loop Invariant Code Motion
- **SCCP** — Sparse Conditional Constant Propagation
- **Inlining** — function inlining
- **Loop unrolling** — loop optimization
- **Vectorization** — SIMD auto-vectorization

## Kyle-specific optimizations

### SSA Construction (before LLVM)

Before LLVM sees the IR, Kyle performs its own SSA construction:
- Promotable allocas (simple types, no escaping) are promoted to SSA values
- Phi nodes are inserted at dominance frontiers
- GVN runs on the SSA function to eliminate redundant computations

### nsw/nuw flags

All integer arithmetic uses `nsw` (no signed wrap) flags, allowing LLVM to optimize integer expressions more aggressively.

### TBAA metadata

Memory operations carry TBAA (Type-Based Alias Analysis) metadata so LLVM can disambiguate memory accesses.

### inbounds GEPs

All pointer arithmetic uses `inbounds` guarantees, allowing LLVM to optimize loop induction variables.

### noalias parameters

Function pointer parameters are marked `noalias`, enabling better alias analysis.

### readonly/readnone

Runtime functions are annotated with `memory(read)` or `memory(none)` attributes.

### !range metadata

Boolean comparison results carry `!range { i32 0, i32 2 }` metadata.
