# Compiler Internals

> For contributors who want to understand how the compiler works under the hood.
> Covers the pipeline, MIR instruction set, borrow analysis, and codegen flow.

---

## 1. Architecture Overview

The Kyle compiler is organized as a **9-crate pipeline** driven by
`kyc_driver`:

```
Source → Lexer → Parser → HIR Build → Semantic → MIR Lower → Borrow Analysis → Codegen → Link
         ↑          ↑          ↑           ↑           ↑            ↑            ↑        ↑
      kyc_frontend  │      kyc_hir    kyc_semantic kyc_mir      kyc_mir    kyc_backend  linker
                    │                             (borrow_analysis.rs)
              kyc_core (AST, Span, Types, Diagnostics)
```

### Crate Dependency Graph

```
kyc_cli → kyc_driver → kyc_frontend → kyc_hir → kyc_semantic → kyc_mir → kyc_backend
                ↑                                                           ↓
            kyc_tools                                                  kyc_runtime (C ABI)
```

Each crate is independently testable. The output of each stage is a complete,
self-contained representation consumed by the next.

---

## 2. MIR Instruction Set Reference

MIR (Mid-level IR) is a flat, control-flow-graph-based representation used for
borrow analysis and lowering to LLVM IR.

### Core Instructions

| Instruction | Operands | Description |
| :--- | :--- | :--- |
| `nop` | none | No-op |
| `goto` | `target: BasicBlock` | Unconditional jump |
| `if(cond)` | `cond: Value, then: BB, else: BB` | Conditional branch |
| `return(val)` | `val: Value` | Return from function |
| `call(dest)` | `dest: Value, fn: String, args: [Value]` | Function call |
| `assign(dest)` | `dest: Value, src: Value` | Copy/move value |
| `load(dest)` | `dest: Value, src: Value` | Load from local |
| `store(dest)` | `dest: Value, src: Value` | Store to local |
| `alloc(dest)` | `dest: Value, ty: Type` | Allocate local variable |

### Ownership Instructions

| Instruction | Operands | Description |
| :--- | :--- | :--- |
| `move(dest)` | `dest: Value, src: Value` | Mark src as moved |
| `borrow(dest)` | `dest: Value, src: Value, fn: String` | Mark as borrowed |
| `mutable_borrow(dest)` | `dest: Value, src: Value, fn: String` | Mark as mutably borrowed |

### List/Dict/Str Operations

> ⚠️ Estas son instrucciones **internas del MIR** (lenguaje intermedio del compilador).
> **NO** son sintaxis de Kyle. El código Kyle como `items.add(x)` se traduce a
> estas instrucciones internamente durante la compilación.

| Instruction | Operands | Description |
| :--- | :--- | :--- |
| `list_new(dest)` | `dest: Value` | Create empty list |
| `list_push(list, val)` | `list: Value, val: Value` | Append element |
| `list_pop(dest, list)` | `dest: Value, list: Value` | Pop last element |
| `list_len(dest, list)` | `dest: Value, list: Value` | Get length |
| `dict_new(dest)` | `dest: Value` | Create empty dict |
| `dict_set(dict, k, v)` | `dict: Value, k: Value, v: Value` | Set key-value |
| `dict_get(dest, dict, k)` | `dest: Value, dict: Value, k: Value` | Get by key |
| `dict_len(dest, dict)` | `dest: Value, dict: Value` | Get entry count |

---

## 3. Borrow Analysis Algorithm

File: `kyc_mir/src/borrow_analysis.rs` (formerly `move_analysis.rs`)

### Copy vs Move Classification

Types are classified as either **Copy** or **Move**:

| Classification | Types | Behavior (assignment) | Behavior (fn param default) |
| :--- | :--- | :--- | :--- |
| **Copy** | `i8`–`i64`, `u8`–`u64`, `f32`, `f64`, `bool`, `char`, `void`, `ptr` | Implicitly duplicated | Copied |
| **Move** | `str`, `[T]`, `{K:V}`, `final class`, `class` | Ownership transfers; source becomes unusable | **Borrowed** (not moved) |

### Dataflow Analysis

The analysis is a **forward dataflow** pass over the MIR CFG:

```
For each basic block:
  1. Start with IN set from predecessors
  2. For each instruction, update OUT set:
     - Assignment of a Move type → mark source as moved
     - Function call → mark args as borrowed (not moved) by default
     - Function call with ^ arg → mark arg as moved
     - `.clone()` → source is NOT moved (borrowed)
  3. At join points: INTERSECT live variables from all predecessors
  4. If a moved variable is used → emit "use-after-move" error
  5. If & is missing for mutable coercion → emit "missing &" error
```

### Parameter Classification

The analysis classifies function parameters into three categories:

| Parameter syntax | Behavior | MIR annotation |
|---|---|---|
| `s: T` | Borrowed (caller retains ownership) | `Borrow` |
| `s: &T` | Mutable borrow (caller retains ownership) | `MutableBorrow` |
| `^s: T` | Ownership transferred (caller loses access) | `Move` |

### Clone Handling

`.clone()` on a Move type:
- The source is **not** moved (it remains usable)
- A new heap allocation is created
- The clone is a fresh, independent value

---

## 4. Codegen Flow

The codegen pipeline converts MIR to an executable binary:

### MIR → LLVM IR (`kyc_backend/src/codegen.rs`)

1. **Function generation**: Each MIR function becomes an LLVM function
2. **Basic block mapping**: Each MIR basic block becomes an LLVM basic block
3. **Instruction lowering**:
   - Arithmetic → LLVM `add`, `sub`, `mul`, etc.
   - List operations → calls to `ky_list_*` runtime functions
   - Dict operations → calls to `ky_dict_*` runtime functions
   - String operations → calls to `ky_str_*` runtime functions
   - Memory allocation → `malloc` / `ky_alloc_*`
4. **Ownership annotation lowering**: Moves/borrows are elided at the LLVM
   level (LLVM IR has no concept of ownership — all values are SSA)

### LLVM IR → Object File

- Uses **inkwell** (safe Rust bindings for LLVM 18)
- Target triple is detected from the host system
- Optimization level:
  - Debug: `None`
  - Release: `Aggressive` (equivalent to `-O3`)

### Object File → Binary

- Links with `kyc_runtime` (static C ABI library)
- Runtime provides: `ky_list_*`, `ky_dict_*`, `ky_str_*`, `ky_print*`,
  `ky_io_*`, `ky_assert_*`, `ky_json_*`, `ky_time_*`
- Linker driver: invokes `cc` (system C compiler) with the object file and
  runtime archive

### Pipeline Entry Point

`kyc_driver/src/lib.rs` — `fn run(config: Config) -> Result<(), Diagnostics>`:

```
1. Parse source → AST (kyc_frontend)
2. Build HIR → HirModule (kyc_hir)
3. Type-check → typed HirModule (kyc_semantic)
4. Lower to MIR → MirModule (kyc_mir)
5. Run borrow analysis → errors or success (kyc_mir)
6. Codegen → LLVM module (kyc_backend)
7. Emit object file → invoke linker → binary (kyc_backend)
```
