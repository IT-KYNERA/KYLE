# Compiler Internals

> For contributors who want to understand how the compiler works under the hood.
> Covers the pipeline, MIR instruction set, move analysis, and codegen flow.

---

## 1. Architecture Overview

The Kyle compiler is organized as a **9-crate pipeline** driven by
`klc_driver`:

```
Source → Lexer → Parser → HIR Build → Semantic → MIR Lower → Move Analysis → Codegen → Link
         ↑          ↑          ↑           ↑           ↑            ↑            ↑        ↑
      klc_frontend  │      klc_hir    klc_semantic klc_mir      klc_mir    klc_backend  linker
                    │                             (move_analysis.rs)
              klc_core (AST, Span, Types, Diagnostics)
```

### Crate Dependency Graph

```
klc_cli → klc_driver → klc_frontend → klc_hir → klc_semantic → klc_mir → klc_backend
                ↑                                                           ↓
            klc_tools                                                  klc_runtime (C ABI)
```

Each crate is independently testable. The output of each stage is a complete,
self-contained representation consumed by the next.

---

## 2. MIR Instruction Set Reference

MIR (Mid-level IR) is a flat, control-flow-graph-based representation used for
move analysis and lowering to LLVM IR.

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

### Move-Specific Instructions

| Instruction | Operands | Description |
| :--- | :--- | :--- |
| `move(dest)` | `dest: Value, src: Value` | Mark src as moved |
| `borrow(dest)` | `dest: Value, src: Value, fn: String` | Mark as borrowed by function |

### List/Dict/Str Operations

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

## 3. Move Analysis Algorithm

File: `klc_mir/src/move_analysis.rs`

### Copy vs Move Classification

Types are classified as either **Copy** or **Move**:

| Classification | Types | Behavior |
| :--- | :--- | :--- |
| **Copy** | `i8`–`i64`, `u8`–`u64`, `f32`, `f64`, `bool`, `char`, `void`, `ptr` | Implicitly duplicated on assignment |
| **Move** | `str`, `[T]`, `{K:V}`, `final class`, `class` | Ownership transfers; source becomes unusable |

### Dataflow Analysis

The analysis is a **forward dataflow** pass over the MIR CFG:

```
For each basic block:
  1. Start with IN set from predecessors
  2. For each instruction, update OUT set:
     - Assignment of a Move type → mark source as moved
     - Function call with Move args → mark args as moved
     - `.clone()` → source is NOT moved (borrowed)
     - Borrowing function call → source is NOT moved
  3. At join points: INTERSECT live variables from all predecessors
  4. If a moved variable is used → emit "use-after-move" error
```

### Borrowing Functions

Certain functions are designated as **borrowing** — they read a value but do
not take ownership:

- `print(val)`, `println(val)`
- `len(val)`, `strlen(val)`
- `str_eq(a, b)`, `str_ne(a, b)`
- `list_len(l)`, `list_push(l, v)`, `list_get(l, i)`, `list_set(l, i, v)`

These are hardcoded in the move analysis. The compiler skips the move
for arguments passed to borrowing functions.

### Clone Handling

`.clone()` on a Move type:
- The source is **not** moved (it remains usable)
- A new heap allocation is created
- The clone is a fresh, independent value

---

## 4. Codegen Flow

The codegen pipeline converts MIR to an executable binary:

### MIR → LLVM IR (`klc_backend/src/codegen.rs`)

1. **Function generation**: Each MIR function becomes an LLVM function
2. **Basic block mapping**: Each MIR basic block becomes an LLVM basic block
3. **Instruction lowering**:
   - Arithmetic → LLVM `add`, `sub`, `mul`, etc.
   - List operations → calls to `kl_list_*` runtime functions
   - Dict operations → calls to `kl_dict_*` runtime functions
   - String operations → calls to `kl_str_*` runtime functions
   - Memory allocation → `malloc` / `kl_alloc_*`
4. **Move annotation lowering**: Moves are elided at the LLVM level (LLVM IR
   has no concept of ownership — all values are SSA)

### LLVM IR → Object File

- Uses **inkwell** (safe Rust bindings for LLVM 18)
- Target triple is detected from the host system
- Optimization level:
  - Debug: `None`
  - Release: `Aggressive` (equivalent to `-O3`)

### Object File → Binary

- Links with `klc_runtime` (static C ABI library)
- Runtime provides: `kl_list_*`, `kl_dict_*`, `kl_str_*`, `kl_print*`,
  `kl_io_*`, `kl_assert_*`, `kl_json_*`, `kl_time_*`
- Linker driver: invokes `cc` (system C compiler) with the object file and
  runtime archive

### Pipeline Entry Point

`klc_driver/src/lib.rs` — `fn run(config: Config) -> Result<(), Diagnostics>`:

```
1. Parse source → AST (klc_frontend)
2. Build HIR → HirModule (klc_hir)
3. Type-check → typed HirModule (klc_semantic)
4. Lower to MIR → MirModule (klc_mir)
5. Run move analysis → errors or success (klc_mir)
6. Codegen → LLVM module (klc_backend)
7. Emit object file → invoke linker → binary (klc_backend)
```
