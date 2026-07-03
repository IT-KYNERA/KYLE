# SSA Construction

## Overview

Kyle converts MIR to SSA form before LLVM codegen. This enables standard compiler optimizations: constant propagation, GVN, dead code elimination.

## Mem2Reg

Promotable allocas (simple integer/float types, no escaping) are promoted to SSA values. Non-promotable allocas remain as memory operations.

## Phi nodes

Phi nodes are placed at dominance frontiers using standard algorithms. Each promotable alloca gets phi nodes at join points where multiple definitions reach.

## GVN

Global Value Numbering eliminates redundant computations across blocks. If the same binary operation appears with the same operands, the second result is replaced by the first.
