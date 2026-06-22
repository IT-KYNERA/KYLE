# Kyle — Verified Implementation Status v1.0

> **Source of truth.** This document is generated from direct inspection
> of the codebase (not from aspirations). Every "✅ working" claim below
> was verified by reading the lowering/codegen, and the end-to-end path
> was verified by compiling and running `examples/fibonacci.kl`.
>
> When this document disagrees with another doc, **this one is right**.
> Last audit: 2026-06-21.

---

## At a glance

| Layer | Status |
|-------|--------|
| Lexer + Parser (frontend) | ✅ Complete (70 + tests) |
| Type checker / semantic | ✅ Complete (14 tests) |
| MIR lowering | ✅ For core constructs · 🔶 Placeholders for advanced |
| LLVM codegen | ✅ Functional end-to-end |
| RAII runtime (Rust) | ✅ Complete |
| Builtins (no import needed) | ✅ Working |
| Importable `std/*.kl` modules | ❌ Not yet written |
| Tooling (CLI/LSP/fmt/VS Code) | ✅ Functional |
| Self-hosting (Phase 6) | 🔄 ~15% (lexer.kl + parser.kl done) |

**Total tests:** 86 passing, 0 failing (70 frontend + 14 semantic + 2 MIR).

---

## ✅ What generates working native code today

Verified by reading `crates/klc_mir/src/lower.rs` and running examples.

### Types

```text
i8 i16 i32 i64    u8 u16 u32 u64    f32 f64    bool    char    str
```

All primitives compile. Structs compile (LLVM struct types, field access
via `FieldPtr`). Lists compile as runtime handles (`list<T>`).

### Statements

```text
✅ fn declarations (incl. recursion)         ✅ if / elif / else
✅ while / for / break                       ✅ match (literal & wildcard only — see below)
✅ return                                    ✅ variable / mut / typed / constant
✅ expression statements                     ✅ defer / guard / unsafe (parsed, basic lowering)
✅ binding-if                                ✅ while-bind
```

### Expressions

```text
✅ literals (int/float/str/char/bool/None)   ✅ identifiers
✅ binary ops (arithmetic, comparison, bitwise, logical)
✅ unary ops (neg, not, bitnot)              ✅ function calls
✅ property access (struct fields)           ✅ list literals [a, b, c]
✅ list indexing list[i]                     ✅ string indexing s[i] (→ char_at)
✅ assignment (=, +=, -=, etc.)
```

### Builtins (no `import` needed — backed by `klc_runtime`)

```text
✅ print  println  input                     ✅ len  str
✅ char_at  ord                               ✅ is_digit is_alpha is_alnum
✅ is_whitespace is_upper is_lower           ✅ contains to_upper to_lower trim replace substr
✅ open read_str write_str close             ✅ sleep now
```

### Verified end-to-end

```bash
$ klc run examples/fibonacci.kl
fibonacci(10) = 55

$ klc run examples/lexer.kl    # 461-line Kyle program
# tokenizes examples/hello.kl correctly with INDENT/DEDENT
```

---

## 🔶 Placeholders — parses & type-checks, but NO working codegen

These features are accepted by the parser and type checker, so they
**will not error out**, but the lowered MIR does not produce the correct
runtime behavior. The lowering either evaluates sub-expressions and
discards the result, or handles only a narrow special case.

> **This is the gap Phase 3.5 must close before Phase 6 can finish.**
> Location: `crates/klc_mir/src/lower.rs` (search for the listed `Expr::` arm).

| Feature | AST arm | What happens today | Priority |
|---------|---------|--------------------|----------|
| **Closures** `(x) => x*2` | `Expr::Closure` | Evaluates body once; no callable value is produced | 🔴 High |
| **Method dispatch** `obj.m()` | `Expr::FunctionCall` + `PropertyAccess` | Hardcoded: only `list.add()` and `list.pop()` map to runtime calls; any other method silently does nothing | 🔴 High |
| **Match with enum variants + data** `text(s):` | `Stmt::Match` | Only `Pattern::Literal`, `Pattern::Wildcard`, `Pattern::Identifier` work. Enum-variant and destructuring patterns are skipped. | 🔴 High |
| **Async / await** | `Expr::Async`, `Expr::Await` | Evaluates inner expression; no state machine, no scheduler integration | 🔴 High |
| **String interpolation** `"Hi {name}"` | (lexed) | Not lowered — use `"Hi " + str(name)` today | 🟡 Medium |
| **Tuples** `(1, 2)` + destructuring | `Expr::Tuple` | Evaluates elements; no tuple value is constructed | 🟡 Medium |
| **Dict / object literals** `{k: v}` | `Expr::Dictionary` | Evaluates values; keys discarded, no map produced | 🟡 Medium |
| **Spread** `[...a, ...b]` | `Expr::Spread` | Evaluates inner; no flattening | 🟡 Medium |
| **Optional chaining** `user?.name` | `Expr::OptionalChain` | Evaluates target; property access discarded | 🟡 Medium |
| **Error propagation `?`** | `Expr::ErrorProp` | Evaluates inner; no error check / early return | 🟡 Medium |
| **Range slice** `s[0..5]`, `l[2..]` | `Expr::RangeSlice` | Evaluates start/end; no slice produced (use `substr` for strings) | 🟡 Medium |
| **Class inheritance / vtables** | `Decl::Class` parent | Methods lower as flat functions; no virtual dispatch | 🟡 Medium |
| **Operator overloading** (`add`, `mul`, …) | spec only | Not implemented in lowering | 🟢 Low |
| **const fn / compile-time eval** | spec only | Not implemented | 🟢 Low |
| **`is` type check** | spec only | Not implemented in lowering | 🟢 Low |

---

## ❌ Not present at all

```text
❌ std/ directory — importable .kl modules do not exist yet.
   (Runtime builtins in Rust DO exist and work — see above.)
❌ DWARF debug info — no -g support, cannot debug with LLDB yet.
❌ LLVM optimization passes beyond -O0.
❌ Lint rules (L0001–L0008 are specified, none implemented).
❌ kl doc (HTML documentation generator).
```

---

## Environmental notes (setup)

### LLVM 18 location

`inkwell 0.9` requires LLVM 18.1. The build needs `LLVM_SYS_181_PREFIX`:

```text
Linux (apt):       /usr/lib/llvm-18            (set in .cargo/config.toml)
macOS Apple Si:    /opt/homebrew/opt/llvm@18   (export in shell — Cargo [env] is static)
macOS Intel:       /usr/local/opt/llvm@18
```

On macOS add to `~/.zshrc`:

```bash
export LLVM_SYS_181_PREFIX=/opt/homebrew/opt/llvm@18
```

### Target triple

`crates/klc_driver/src/pipeline.rs::emit_object` currently hardcodes:

```text
"arm64-apple-macosx"        (macOS)
"aarch64-unknown-linux-gnu" (Linux)
```

Porting to x86_64 or Windows requires generalizing this (use
`TargetMachine::get_default_triple()`).

---

## Phase guidance

### Next: Phase 3.5 (close the codegen gap)

Pick from the 🔴 High rows first. Recommended order:

1. **Method dispatch** — needed for any non-trivial Kyle program and for
   the self-hosting compiler's AST node method calls.
2. **Closures** — the self-hosting compiler uses visitor/closure patterns
   heavily.
3. **Match with enum variants** — `Result<T,E>` and `Option<T>` are
   useless without this; error handling depends on it.
4. **Async/await** — lower priority for self-hosting but high for the
   language's value proposition.

### Then: Phase 6 (continue self-hosting)

`examples/lexer.kl` (461 lines) and `examples/parser.kl` (1509 lines)
are done. Remaining components to write in Kyle:

```text
[ ] Semantic analyzer in Kyle
[ ] MIR lowering in Kyle
[ ] Codegen in Kyle (hardest — needs LLVM FFI surface)
[ ] Bootstrap: klc compiles itself
```

### Later: Phase 7 (stdlib + maturity)

```text
[ ] Write std/io.kl, std/math.kl, std/core.kl, std/testing.kl
[ ] DWARF debug info
[ ] Real LLVM optimization passes (-O2/-O3)
[ ] Lint rules
[ ] kl doc
```

---

## How to re-verify this document

```bash
# 1. Build (needs LLVM 18 — see above)
cargo build --workspace

# 2. Run tests (should report 86 passed, 0 failed)
cargo test --workspace

# 3. End-to-end smoke test
cargo run --bin klc -- run examples/fibonacci.kl
# expected output: fibonacci(10) = 55

# 4. Inspect a placeholder lowering
grep -n "Expr::Closure" crates/klc_mir/src/lower.rs
# you'll see: ctx = self.lower_expr(ctx, body); ctx  ← placeholder
```

If any ✅ claim above fails these checks, this document is stale and
must be updated.

---

## Version

```text
Kyle Verified Implementation Status v1.0
Last updated: 2026-06-21
```
