# Compiler Architecture

> How the Kyle compiler is organized: the 9 Rust crates, the 8-stage
> pipeline, and the runtime it links against.

---

## 1. The Pipeline

```
   ┌──────────┐    ┌───────┐    ┌────────┐    ┌────────┐    ┌──────────┐    ┌────────┐
   │  Source  │ →  │ Lexer │ →  │ Parser │ →  │  HIR   │ →  │ Semantic │ →  │  MIR   │
   │  .kl     │    │       │    │        │    │ Build  │    │  + Types │    │Lowering│
   └──────────┘    └───────┘    └────────┘    └────────┘    └──────────┘    └────┬───┘
                                                                                   │
   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐   │
   │  Binary  │ ←  │  Linker  │ ←  │  SSA     │ ←  │  Move    │ ←  │ Optimize │ ←─┘
   │   .kl    │    │          │    │  Codegen │    │ Analysis │    │          │
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
kl/
├── crates/
│   ├── klc_core/        ← AST, span, types, diagnostics
│   ├── klc_frontend/    ← Lexer, parser
│   ├── klc_hir/         ← HIR definition, builder, HIR-level validation
│   ├── klc_semantic/    ← Symbol table, type checker, scope resolver
│   ├── klc_mir/         ← MIR definition, lowering, optimizer, move analysis
│   ├── klc_backend/     ← LLVM 18 codegen (inkwell), linker driver
│   ├── klc_driver/      ← Pipeline orchestrator (klc::run)
│   ├── klc_cli/         ← Command-line interface (the `kl` binary)
│   ├── klc_runtime/     ← C-ABI runtime (memory, string, list, dict, io, async)
│   └── klc_tools/       ← LSP server, formatter, package manager
```

| Crate | Purpose | Inputs | Outputs |
|---|---|---|---|
| `klc_core` | Foundation types | (none) | AST nodes, type system, source maps, diagnostic reporters |
| `klc_frontend` | Lexing + parsing | Source string | `Program` (AST) |
| `klc_hir` | HIR building + validation | `Program` (AST) | `HirModule` (HIR) |
| `klc_semantic` | Name resolution + type checking | `HirModule` | `HirModule` (typed) + `SymbolTable` |
| `klc_mir` | Mid-level IR + passes | `HirModule` | `MirModule` (functions, types, control flow) |
| `klc_backend` | SSA Form + LLVM codegen + linking | `MirModule` | Native executable |
| `klc_driver` | Pipeline orchestrator | (none) | `klc::run(source, options) -> Result` |
| `klc_cli` | Command-line tool | argv | Compiled binary, stdout/stderr |
| `klc_runtime` | C-ABI runtime library | (linked at compile time) | Provides memory, I/O, string ops to compiled Kyle code |
| `klc_tools` | Out-of-band tools (LSP, fmt) | Source string | LSP responses, formatted source |

---

## 3. Stage 1: Lexer (`klc_frontend::lexer`)

Converts source text into a stream of tokens.

**Output:** `Vec<Token>` with `TokenKind` and `Span` for each token.

**Recognized token kinds:**

- 40+ keywords (`fn`, `class`, `if`, `match`, `and`, `or`, `abstract`, `final`, ...)
- Literals: integer (decimal, hex `0x`, binary `0b`, with `_` separators), float, string (with `{...}` interpolation), char, `true`, `false`, `None`
- Identifiers (alphanumeric + `_`, starting with letter or `_`)
- Operators: arithmetic, comparison, logical, bitwise, range, spread, ternary, optional chain, error prop
- Assignment operators: `=` (immutable bind), `:=` (`Walrus`, mutable bind), `::=` (`ConstDecl`, constant declaration)
- Indentation: `INDENT` and `DEDENT` (the lexer implements Python-style indent/dedent based on leading whitespace)
- Modifier keywords: `Abstract`, `Final` (class/variant modifiers)
- Punctuation: `(`, `)`, `[`, `]`, `{`, `}`, `,`, `:` (with no space), `;` (optional)

The lexer does **not** validate syntax — it just tokenizes. Syntax errors
are detected by the parser.

---

## 4. Stage 2: Parser (`klc_frontend::parser`)

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

- `:=` (walrus) and `::=` (const-decl) are parsed as `DeclKind::Variable` with
  mutability/constancy flags on the declaration node
- `abstract class` / `final class` are parsed as modifiers on the class declaration
  (the lexer emits `Abstract` / `Final` tokens before `class`/`variant`)
- `T?` is parsed as `TypeKind::Optional(inner)` — sugar for `Option<T>` that is
  normalized during HIR building

---

## 5. Stage 3: HIR Build (`klc_hir`)

Converts the parser's AST into the High-level Intermediate Representation
(HIR). The HIR is a desugared, simplified tree that is easier for the
semantic analyzer and subsequent passes to consume.

**Output:** `HirModule { items: Vec<HirItem> }`

**Desugarings performed:**

- `T?` is normalized to `Option<T>` (the internal representation)
- `:=` (walrus) and `::=` (const-decl) are lowered to immutable variable
  declarations with mutability/constancy flags on the `HirBinding`
- `abstract class` / `final class` modifiers are stored as flags on the
  `HirClass` node
- String interpolation (`"hello {name}"`) is decomposed into a sequence
  of literal and interpolation fragments
- Tuple unpacking in assignments is normalized

**Validation:** The HIR builder also checks for basic well-formedness
constraints not checked by the parser (e.g. duplicate declarations in the
same scope, invalid modifier combinations).

---

## 6. Stage 4: Semantic Analysis (`klc_semantic`)

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

## 7. Stage 5: MIR Lowering (`klc_mir::lower`)

Converts the typed HIR into Kyle's Mid-level Intermediate Representation
(MIR). MIR is a simpler, more uniform form that is easier to analyze and
optimize.

**Output:** `MirModule { functions: Vec<MirFunction> }`

**Key design:** MIR is **not** LLVM IR. It is Kyle's own IR, designed
specifically for Kyle's semantics. The backend translates MIR → LLVM IR.

**Special handling in the lowerer:**

- Builtin functions are resolved to runtime calls (`print` → `kl_print`,
  `len` → `kl_strlen`, etc.)
- String interpolation is decomposed into `kl_concat` calls
- `async <expr>` is wrapped in a `kl_spawn_thread` call
- `await <handle>` is lowered to `kl_join_thread`
- Method dispatch on classes is resolved via the `method_table`
- Inheritance is handled via the `class_parent_map` chain walk
- Default constructor is synthesized if the class has none

---

## 8. Stage 6: MIR Optimization (`klc_mir::optimize`)

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

## 9. Stage 7: Move Analysis (`klc_mir::move_analysis`)

Performs move-semantics analysis on the MIR to ensure that no value is
used after it has been moved. Eliminates the need for refcounting.

**What is tracked:**

- Every variable binding has a move state (live / moved / consumed)
- Function call arguments are consumed (moved) unless the parameter type
  is `Copy`
- Assignment to a previously bound name (`x = new_val`) moves out the
  old value
- Return values from functions are moved to the caller

**Enforcement:**

- A compile error is emitted if a moved value is referenced again
- The analysis handles all control-flow paths (branches, loops, early
  returns) using a data-flow framework on the MIR control-flow graph

**Future (Phase 11+):**

- Partial moves (moving one field of a class while keeping others alive)
- Destructors (`drop`) for types with cleanup logic

---

## 10. Stage 8: SSA Form Transformation (`klc_mir::ssa`) — NUEVO (Phase 15)

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

## 11. Stage 9: LLVM Codegen (`klc_backend`)

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
The runtime library `libklc_runtime.a` is linked statically.

---

## 12. The Runtime (`klc_runtime`)

A static C-ABI library linked into every compiled Kyle program. It
provides the primitive operations that the compiler lowers calls to.
The runtime uses move semantics. Refcounting functions (kl_retain/kl_release)
remain available for future Rc/Arc use in the stdlib.

| File | Responsibility |
|---|---|
| `memory.rs` | `kl_alloc`, `kl_free`, `kl_retain`, `kl_release` — heap management (reserved for future Rc/Arc) |
| `string.rs` | `kl_concat`, `kl_strlen`, `kl_str_to_*`, `kl_i64_to_str`, `kl_str_to_i64` |
| `list.rs` | `kl_list_new`, `kl_list_push`, `kl_list_pop`, `kl_list_get`, `kl_list_set`, `kl_list_len`, `kl_list_slice`, `kl_list_extend` |
| `dict.rs` | `kl_dict_new`, `kl_dict_set`, `kl_dict_get`, `kl_dict_len` |
| `io.rs` | `kl_print`, `kl_println`, `kl_input`, `kl_input_with_prompt`, `kl_open`, `kl_read_str`, `kl_write_str`, `kl_close` |
| `async_.rs` | `kl_spawn_thread`, `kl_join_thread` — async/await runtime |
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

Eight `.kl` modules, all written in Kyle itself. The public syntax for
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

Both modes link the same `libklc_runtime.a`. The difference is in the
optimization level passed to LLVM.

---

## 15. Development Commands

```bash
# Build all crates
cargo build --workspace

# Run all unit tests
cargo test -p klc_core -p klc_frontend -p klc_hir -p klc_semantic \
          -p klc_mir -p klc_runtime -p klc_tools

# Build the kl binary in release mode
cargo build --release --bin kl

# Type-check a file without building
./target/release/kl check examples/hello.kl

# Run a file
./target/release/kl run examples/hello.kl

# Format source
./target/release/kl fmt src/main.kl
```

---

*Version: v0.4.0 · Last updated: 2026-06-29*
