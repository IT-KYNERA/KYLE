# Code Generation — LLVM Backend

Kyle uses [inkwell](https://github.com/TheDan64/inkwell) (LLVM 18.1 bindings) for code generation.

## Architecture

The codegen lives in `kyc_backend::codegen` and converts MIR instructions to LLVM IR.

## Key components

### Type mapping

| MIR Type | LLVM Type |
|----------|-----------|
| `MirType::I32` | `i32` |
| `MirType::I64` | `i64` |
| `MirType::F32` | `f32` |
| `MirType::F64` | `f64` |
| `MirType::Bool` | `i1` |
| `MirType::Str` | `ptr` (pointer to heap data) |
| `MirType::Struct(name, fields)` | `{ field_types }` named struct |

### Function compilation

1. **Declare phase**: All functions declared in the module (including runtime externs)
2. **SSA pass**: Promotable functions go through SSA construction → `compile_ssa_function`
3. **Non-SSA pass**: Functions with escaping allocas → `compile_function`
4. **Phi resolution**: Phi node incomings are filled after all blocks are compiled

### Runtime linking

Each Kyle binary is linked with `libkyc_runtime.a`, which provides:
- Memory management (reference-counted allocator)
- String operations (concat, search, conversion)
- List operations (dynamic array)
- Dict operations (hash map)
- Async runtime (thread pool)
- File I/O, JSON, time

### Optimization passes (post-codegen)

After LLVM IR is generated, `run_passes("default<O3>")` is called to optimize the module before object code emission.
