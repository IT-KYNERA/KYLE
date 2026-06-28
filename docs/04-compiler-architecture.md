# Compiler Architecture

> How the Kyle compiler is organized: the 9 Rust crates, the 7-stage
> pipeline, and the runtime it links against.

---

## 1. The Pipeline

```
   ┌──────────┐    ┌───────┐    ┌────────┐    ┌──────────┐    ┌─────────┐
   │  Source  │ →  │ Lexer │ →  │ Parser │ →  │ Semantic │ →  │   MIR   │
   │  .kl     │    │       │    │        │    │  + Types │    │ Lowering│
   └──────────┘    └───────┘    └────────┘    └──────────┘    └────┬────┘
                                                                     │
   ┌──────────┐    ┌─────────┐    ┌─────────┐    ┌──────────┐         │
   │  Binary  │ ←  │  Linker │ ←  │  LLVM   │ ←  │Optimize+ │ ←───────┘
   │ kl       │    │         │    │ Codegen │    │Ownership │
   └──────────┘    └─────────┘    └─────────┘    └──────────┘
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
│   ├── klc_semantic/    ← Symbol table, type checker, scope resolver
│   ├── klc_mir/         ← MIR definition, lowering, optimizer, ownership
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
| `klc_semantic` | Name resolution + type checking | `Program` | `Program` (typed) + `SymbolTable` |
| `klc_mir` | Mid-level IR + passes | `Program` | `MirModule` (functions, types, control flow) |
| `klc_backend` | LLVM codegen + linking | `MirModule` | Native executable |
| `klc_driver` | Glue between stages | (none) | `klc::run(source, options) -> Result` |
| `klc_cli` | Command-line tool | argv | Compiled binary, stdout/stderr |
| `klc_runtime` | C-ABI runtime library | (linked at compile time) | Provides memory, I/O, string ops to compiled Kyle code |
| `klc_tools` | Out-of-band tools (LSP, fmt) | Source string | LSP responses, formatted source |

---

## 3. Stage 1: Lexer (`klc_frontend::lexer`)

Converts source text into a stream of tokens.

**Output:** `Vec<Token>` with `TokenKind` and `Span` for each token.

**Recognized token kinds:**

- 38 keywords (`fn`, `class`, `if`, `match`, `mut`, `and`, `or`, ...)
- Literals: integer (decimal, hex `0x`, binary `0b`, with `_` separators), float, string (with `{...}` interpolation), char, `true`, `false`, `None`
- Identifiers (alphanumeric + `_`, starting with letter or `_`)
- Operators: arithmetic, comparison, logical, bitwise, assignment, range, spread, ternary, optional chain, error prop
- Indentation: `INDENT` and `DEDENT` (the lexer implements Python-style indent/dedent based on leading whitespace)
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

---

## 5. Stage 3: Semantic Analysis (`klc_semantic`)

Resolves names to symbols and checks types.

Three sub-phases:

### 5.1 Scope Resolution

Walks the AST building a `SymbolTable` — a stack of scopes, one per block
and one per class. Every name reference is resolved to a `Symbol` that
records its kind (variable, function, class, etc.) and its type.

### 5.2 Type Checking

Verifies that every expression has a well-defined type, that assignments
match, and that function calls pass the right number of types of arguments.

### 5.3 Contract Validation

For each class that declares `class X: Contract`, verifies that `X`
provides all the methods declared in `Contract`. (Generic contracts are
not yet supported.)

**Output:** A typed `Program`, a `SymbolTable`, and a list of
`Diagnostic` (errors and warnings).

---

## 6. Stage 4: MIR Lowering (`klc_mir::lower`)

Converts the typed AST into Kyle's Mid-level Intermediate Representation
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

## 7. Stage 5: MIR Optimization (`klc_mir::optimize`)

Constant-folding and dead-code elimination passes on the MIR.

**Currently implemented:**

- Constant folding: `2 + 3` → `5` at MIR level
- Unreachable-code elimination after unconditional `return` / `break`

**Planned (Phase 10):**

- Loop-invariant code motion
- Strength reduction
- Function inlining

---

## 8. Stage 6: Ownership Pass (`klc_mir::ownership`)

Inserts `kl_retain` / `kl_release` calls to manage refcounting automatically.

**Tracked operations (currently):**

- `kl_concat` results (string concatenation allocates)
- `kl_list_new` / `kl_dict_new` results
- Direct `kl_alloc` results

**Known limitation:** Forwarded values (e.g. `let y = f(x); use(y)`) are
conservatively considered leaks. This will be addressed in Phase 11.

---

## 9. Stage 7: LLVM Codegen (`klc_backend`)

Translates MIR to LLVM IR using `inkwell` (Rust bindings for LLVM 18), then
invokes the LLVM toolchain to compile to an object file, and links the
runtime library to produce the final executable.

**Output:** A native binary in `target/<debug|release>/<name>`.

**Optimization level:** Currently `Default` (LLVM's default). The `--release`
flag is accepted but does not yet switch to `O2`/`O3` — this is planned for
Phase 9.

**Linker:** Uses the system linker (`cc`) for the final link step. The
runtime library `libklc_runtime.a` is linked statically.

---

## 10. The Runtime (`klc_runtime`)

A static C-ABI library linked into every compiled Kyle program. It
provides the primitive operations that the compiler lowers calls to.

| File | Responsibility |
|---|---|
| `memory.rs` | `kl_alloc`, `kl_free`, `kl_retain`, `kl_release` — refcounting heap |
| `string.rs` | `kl_concat`, `kl_strlen`, `kl_str_to_*`, `kl_i64_to_str`, `kl_str_to_i64` |
| `list.rs` | `kl_list_new`, `kl_list_push`, `kl_list_pop`, `kl_list_get`, `kl_list_set`, `kl_list_len`, `kl_list_slice`, `kl_list_extend` |
| `dict.rs` | `kl_dict_new`, `kl_dict_set`, `kl_dict_get`, `kl_dict_len` |
| `io.rs` | `kl_print`, `kl_println`, `kl_print_int`, `kl_input`, `kl_input_with_prompt`, `kl_open`, `kl_read_str`, `kl_write_str`, `kl_close` |
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

## 11. The Standard Library (`std/`)

Eight `.kl` modules, all written in Kyle itself:

| Module | Purpose |
|---|---|
| `core` | `Option<T>`, `Some`, `None`, `unwrap_or`, `is_some`, `is_none` |
| `math` | `absolute`, `pow`, `sqrt`, `gcd`, `min`, `max`, `clamp` |
| `io` | `read_file`, `write_file` (file convenience wrappers) |
| `str` | `starts_with_str`, `ends_with_str`, `capitalize`, `repeat_str` |
| `testing` | `assert`, `assert_eq`, `assert_ne`, `assert_str` |
| `collections` | `list_sum`, `list_product`, `list_max`, `list_min`, `list_range` |
| `json` | `parse`, `stringify` (wrappers around the `json_*` builtins) |
| `time` | `timestamp`, `sleep_ms`, `seconds_since` |

---

## 12. Compilation Modes

| Mode | Trigger | Optimization | Output |
|---|---|---|---|
| Debug | default | None (`-O0`) | `target/debug/<name>` |
| Release | `--release` | Default (planned: `-O2` or `-O3`) | `target/release/<name>` |

Both modes link the same `libklc_runtime.a`. The difference is in the
optimization level passed to LLVM.

---

## 13. Development Commands

```bash
# Build all crates
cargo build --workspace

# Run all 101 unit tests
cargo test -p klc_core -p klc_frontend -p klc_semantic \
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

*Version: v0.2.2 · Last updated: 2026-06-27*
