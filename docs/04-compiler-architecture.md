# Compiler Architecture

> How the Kyle compiler is organized: the 9 Rust crates, the 8-stage
> pipeline, and the runtime it links against.

---

## 1. The Pipeline

```
   ┌──────────┐    ┌───────┐    ┌────────┐    ┌────────┐    ┌──────────┐    ┌────────┐
   │  Source  │ →  │ Lexer │ →  │ Parser │ →  │  HIR   │ →  │ Semantic │ →  │  MIR   │
   │  .ky     │    │       │    │        │    │ Build  │    │  + Types │    │Lowering│
   └──────────┘    └───────┘    └────────┘    └────────┘    └──────────┘    └────┬───┘
                                                                                   │
   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐   │
   │  Binary  │ ←  │  Linker  │ ←  │  SSA     │ ←  │  Borrow  │ ←  │ Optimize │ ←─┘
   │   .ky    │    │          │    │  Codegen │    │ Analysis │    │          │
   └──────────┘    └──────────┘    └─────┬─────┘    └──────────┘    └──────────┘
                                         │
                                    ┌────▼────┐
                                    │   SSA   │
                                    │  Form   │
                                    │  Pass   │
                                    └─────────┘
```

Each stage is implemented as a separate Rust crate and is independently
testable. The output of each stage is a complete, self-contained
representation that the next stage consumes.

---

## 2. The Nine Crates

```
ky/
├── crates/
│   ├── kyc_core/        ← AST, span, types, diagnostics
│   ├── kyc_frontend/    ← Lexer, parser
│   ├── kyc_hir/         ← HIR definition, builder, HIR-level validation
│   ├── kyc_semantic/    ← Symbol table, type checker, scope resolver
│   ├── kyc_mir/         ← MIR definition, lowering, optimizer, borrow analysis
│   ├── kyc_backend/     ← LLVM 18 codegen (inkwell), linker driver
│   ├── kyc_driver/      ← Pipeline orchestrator (klc::run)
│   ├── kyc_cli/         ← Command-line interface (the `kl` binary)
│   ├── kyc_runtime/     ← C-ABI runtime (memory, string, list, dict, io, async)
│   └── kyc_tools/       ← LSP server, formatter, package manager
```

| Crate | Purpose | Inputs | Outputs |
|---|---|---|---|
| `kyc_core` | Foundation types | (none) | AST nodes, type system, source maps, diagnostic reporters |
| `kyc_frontend` | Lexing + parsing | Source string | `Program` (AST) |
| `kyc_hir` | HIR building + validation | `Program` (AST) | `HirModule` (HIR) |
| `kyc_semantic` | Name resolution + type checking | `HirModule` | `HirModule` (typed) + `SymbolTable` |
| `kyc_mir` | Mid-level IR + passes | `HirModule` | `MirModule` (functions, types, control flow) |
| `kyc_backend` | SSA Form + LLVM codegen + linking | `MirModule` | Native executable |
| `kyc_driver` | Pipeline orchestrator | (none) | `klc::run(source, options) -> Result` |
| `kyc_cli` | Command-line tool | argv | Compiled binary, stdout/stderr |
| `kyc_runtime` | C-ABI runtime library | (linked at compile time) | Provides memory, I/O, string ops to compiled Kyle code |
| `kyc_tools` | Out-of-band tools (LSP, fmt) | Source string | LSP responses, formatted source |

---

## 3. Stage 1: Lexer (`kyc_frontend::lexer`)

Converts source text into a stream of tokens.

**Output:** `Vec<Token>` with `TokenKind` and `Span` for each token.

**Recognized token kinds:**

- 40+ keywords (`fn`, `class`, `if`, `match`, `and`, `or`, `abstract`, `final`, ...)
- Literals: integer (decimal, hex `0x`, binary `0b`, with `_` separators), float, string (with `{...}` interpolation), char, `true`, `false`, `None`
- Identifiers (alphanumeric + `_`, starting with letter or `_`)
- Operators: arithmetic, comparison, logical, bitwise, range, spread, ternary, optional chain, error prop
- Assignment operators: `=` (immutable bind), `:=` (`Walrus`, constant declaration), `&T` (mutable type prefix), `^T` (move type prefix)
- Indentation: `INDENT` and `DEDENT` (the lexer implements Python-style indent/dedent based on leading whitespace)
- Modifier keywords: `Abstract`, `Final` (class/variant modifiers)
- Punctuation: `(`, `)`, `[`, `]`, `{`, `}`, `,`, `:` (with no space), `;` (optional)

The lexer does **not** validate syntax — it just tokenizes. Syntax errors
are detected by the parser.

---

## 4. Stage 2: Parser (`kyc_frontend::parser`)

Builds an AST from the token stream.

**Output:** `Program { declarations: Vec<Decl> }`

**Parser type:** Recursive descent with one token of lookahead. Each
declaration and statement is parsed by a dedicated `parse_*` method.

**Indentation-based blocks:** The parser uses the `INDENT`/`DEDENT` tokens
from the lexer to identify block bodies. There is no `end` keyword and no
`{` `}` braces.

**Error recovery:** The parser reports the first syntax error and stops. It
does not attempt to recover and report multiple errors.

**New syntax handled:**

- `:=` (walrus) is parsed as `DeclKind::Variable` with constancy flag for constants
- `&T` is parsed as mutable reference type
- `^T` is parsed as move type (ownership transfer)
- `abstract class` / `final class` are parsed as modifiers on the class declaration
  (the lexer emits `Abstract` / `Final` tokens before `class`/`variant`)
- `T?` is parsed as `TypeKind::Optional(inner)` — sugar for `Option<T>` that is
  normalized during HIR building

---

## 5. Stage 3: HIR Build (`kyc_hir`)

Converts the parser's AST into the High-level Intermediate Representation
(HIR). The HIR is a desugared, simplified tree that is easier for the
semantic analyzer and subsequent passes to consume.

**Output:** `HirModule { items: Vec<HirItem> }`

**Desugarings performed:**

- `T?` is normalized to `Option<T>` (the internal representation)
- `:=` (walrus) is lowered to constant declaration with constancy flag on `HirBinding`
- `&T` types are lowered to mutable reference types in HIR
- `^T` types are lowered to move/ownership types in HIR
- `abstract class` / `final class` modifiers are stored as flags on the
  `HirClass` node
- String interpolation (`"hello {name}"`) is decomposed into a sequence
  of literal and interpolation fragments
- Tuple unpacking in assignments is normalized

**Validation:** The HIR builder also checks for basic well-formedness
constraints not checked by the parser (e.g. duplicate declarations in the
same scope, invalid modifier combinations).

---

## 6. Stage 4: Semantic Analysis (`kyc_semantic`)

Resolves names to symbols and checks types. Operates on the HIR rather
than the raw AST.

Three sub-phases:

### 6.1 Scope Resolution

Walks the HIR building a `SymbolTable` — a stack of scopes, one per block
and one per class. Every name reference is resolved to a `Symbol` that
records its kind (variable, function, class, etc.) and its type.

### 6.2 Type Checking

Verifies that every expression has a well-defined type, that assignments
match, and that function calls pass the right number and types of arguments.

### 6.3 Contract Validation

For each class that declares `class X: Contract`, verifies that `X`
provides all the methods declared in `Contract`. (Generic contracts are
not yet supported.)

**Output:** A typed `HirModule`, a `SymbolTable`, and a list of
`Diagnostic` (errors and warnings).

---

## 7. Stage 5: MIR Lowering (`kyc_mir::lower`)

Converts the typed HIR into Kyle's Mid-level Intermediate Representation
(MIR). MIR is a simpler, more uniform form that is easier to analyze and
optimize.

**Output:** `MirModule { functions: Vec<MirFunction> }`

**Key design:** MIR is **not** LLVM IR. It is Kyle's own IR, designed
specifically for Kyle's semantics. The backend translates MIR → LLVM IR.

**Special handling in the lowerer:**

- Builtin functions are resolved to runtime calls (`print` → `ky_print`,
  `len` → `ky_strlen`, etc.)
- String interpolation is decomposed into `ky_concat` calls
- `async <expr>` is wrapped in a `ky_spawn_thread` call
- `await <handle>` is lowered to `ky_join_thread`
- Method dispatch on classes is resolved via the `method_table`
- Inheritance is handled via the `class_parent_map` chain walk
- Default constructor is synthesized if the class has none

---

## 8. Stage 6: MIR Optimization (`kyc_mir::optimize`)

Constant-folding, dead-code elimination, and inlining passes on the MIR.

**Currently implemented:**

- Constant folding: `2 + 3` → `5` at MIR level
- Unreachable-code elimination after unconditional `return` / `break`
- **Function inlining** (Phase 15): inline small/single-call functions
- **Mem2Reg** (Phase 15): promote non-escaping allocas to SSA values

**Planned:**

- Loop-invariant code motion
- Strength reduction
- GVN (Global Value Numbering) on SSA

---

## 9. Stage 7: Borrow Analysis (`kyc_mir::borrow_analysis`)

Performs borrow/ownership analysis on the MIR. Parameters are **borrowed
by default** (not moved). Only `^` parameters transfer ownership.

**What is tracked:**

- Every variable binding has a state (live / moved / borrowed)
- Function call arguments are **borrowed** unless the parameter is `^T`
- Assignment to a previously bound name (`x = new_val`) moves out the
  old value
- Return values from functions are moved to the caller
- Parameter types: `s: T` = borrow, `s: &T` = mutable borrow, `^s: T` = move

**Enforcement:**

- A compile error is emitted if a moved value is referenced again
- A compile error is emitted if `&` is missing for mutation coercion
- The analysis handles all control-flow paths (branches, loops, early
  returns) using a data-flow framework on the MIR control-flow graph

**Future (Fase 14+):**

- Partial moves (moving one field of a class while keeping others alive)
- Destructors (`drop`) for types with cleanup logic
- Full borrow checker with reference types (`&T`, `^T`)

---

## 10. Stage 8: SSA Form Transformation (`kyc_mir::ssa`) — NUEVO (Phase 15)

Converts the lowered MIR (`MirFunction` with load/store) into **Static Single
Assignment** form (`SsaFunction` with phi nodes). This is the most impactful
optimization in the compiler — it eliminates the allocas and load/store pairs
that prevent LLVM from optimizing aggressively.

**Output:** `SsaModule { functions: Vec<SsaFunction> }`

**Key passes:**

1. **Mem2Reg** — Identify non-escaping allocas (no `field_ptr`/`PtrOffset` to them).
   Replace each `load %X` with the value of the last `store %X` reaching that point.
   Insert phi nodes at join points where multiple definitions converge.

2. **Phi placement** — Use the dominance-frontier algorithm to place minimal phi nodes.

3. **Renaming** — Walk the dominator tree, renaming variables so each assignment
   gets a unique SSA version.

**SSA vs non-SSA:** Functions that use heap-allocated types (strings, lists, dicts,
class instances) or have `field_ptr`/`Memcpy` instructions remain in non-SSA form,
since those allocas escape and cannot be promoted.

**Optimizations that require SSA:**
- GVN (Global Value Numbering)
- Constant propagation across blocks
- Dead store elimination

---

## 11. Stage 9: LLVM Codegen (`kyc_backend`)

Translates **SsaFunction** (or `MirFunction` for non-SSA functions) to LLVM IR
using `inkwell` (Rust bindings for LLVM 18), then invokes the LLVM toolchain to
compile to an object file, and links the runtime library to produce the final
executable.

**SSA codegen:** For SSA functions, the codegen emits LLVM values directly
(phi nodes, arithmetic, calls) without allocating stack slots or emitting
load/store pairs. This lets LLVM apply its full optimization pipeline
(constant propagation, loop optimization, vectorization).

**Non-SSA codegen:** Functions with escaping allocas fall back to the original
alloca+load+store codegen path.

**Output:** A native binary in `target/<debug|release>/<name>`.

**Optimization levels:**
| Mode | LLVM flags | Codegen |
|------|-----------|---------|
| Debug | `-O0` | Non-SSA (alloca) |
| Release | `-O2 -flto=thin` | SSA + Alias Analysis |

**Linker:** Uses the system linker (`cc`) with `-flto=thin` for release builds.
The runtime library `libkyc_runtime.a` is linked statically.

---

## 12. The Runtime (`kyc_runtime`)

A static C-ABI library linked into every compiled Kyle program. It
provides the primitive operations that the compiler lowers calls to.
The runtime uses borrow semantics. Refcounting functions (ky_retain/ky_release)
remain available for future Rc/Arc use in the stdlib.

| File | Responsibility |
|---|---|
| `memory.rs` | `ky_alloc`, `ky_free`, `ky_retain`, `ky_release` — heap management (reserved for future Rc/Arc) |
| `string.rs` | `ky_concat`, `ky_strlen`, `ky_str_to_*`, `ky_i64_to_str`, `ky_str_to_i64` |
| `list.rs` | `ky_list_new`, `ky_list_push`, `ky_list_pop`, `ky_list_get`, `ky_list_set`, `ky_list_len`, `ky_list_slice`, `ky_list_extend` |
| `dict.rs` | `ky_dict_new`, `ky_dict_set`, `ky_dict_get`, `ky_dict_len` |
| `io.rs` | `ky_print`, `ky_println`, `ky_input`, `ky_input_with_prompt`, `ky_open`, `ky_read_str`, `ky_write_str`, `ky_close` |
| `async_.rs` | `ky_spawn_thread`, `ky_join_thread` — async/await runtime |
| `channel.rs` | (planned) `Channel<T>` for inter-thread communication |
| `thread.rs` | OS thread spawning primitives |
| `panic.rs` | Panic handler for `assert` failures |
| `task.rs` | Async task internals |
| `error.rs` | Error reporting helpers |
| `lib.rs` | Public re-exports |

The runtime is written in **pure Rust** with `#[unsafe(no_mangle)]` and
`extern "C"` to expose a C-ABI. There is no C or C++ in the runtime.

---

## 13. The Standard Library (`std/`)

Eight `.ky` modules, all written in Kyle itself. The public syntax for
optional types is `T?` (internally represented as `Option<T>`).

| Module | Purpose |
|---|---|
| `core` | `T?`, `Some`, `None`, `unwrap_or`, `is_some`, `is_none` |
| `math` | `absolute`, `pow`, `sqrt`, `gcd`, `min`, `max`, `clamp` |
| `io` | `read_file`, `write_file` (file convenience wrappers) |
| `str` | `starts_with_str`, `ends_with_str`, `capitalize`, `repeat_str` |
| `testing` | `assert`, `assert_eq`, `assert_ne`, `assert_str` |
| `collections` | `list_sum`, `list_product`, `list_max`, `list_min`, `list_range` |
| `json` | `parse`, `stringify` (wrappers around the `json_*` builtins) |
| `time` | `timestamp`, `sleep_ms`, `seconds_since` |

---

## 14. Compilation Modes

| Mode | Trigger | Optimization | Output |
|---|---|---|---|
| Debug | default | None (`-O0`) | `target/debug/<name>` |
| Release | --release | Aggressive (O2/O3) | target/release/<name> |

Both modes link the same `libkyc_runtime.a`. The difference is in the
optimization level passed to LLVM.

---

## 15. Development Commands

```bash
# Build all crates
cargo build --workspace

# Run all unit tests
cargo test -p kyc_core -p kyc_frontend -p kyc_hir -p kyc_semantic \
          -p kyc_mir -p kyc_runtime -p kyc_tools

# Build the ky binary in release mode
cargo build --release --bin ky

# Type-check a file without building
./target/release/kl check examples/hello.ky

# Run a file
./target/release/kl run examples/hello.ky

# Format source
./target/release/kl fmt src/main.ky
```

---

*Version: v0.4.0 · Last updated: 2026-06-29*
