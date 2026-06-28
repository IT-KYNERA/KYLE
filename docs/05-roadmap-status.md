# Roadmap & Status

> The phase-by-phase evolution plan, the implementation matrix, and the v1.0
> release checklist. This is the single source of truth for "what's done, what's
> next, and what we'll ship at v1.0."

---

## 1. Phase Overview

| Phase | Focus | Status |
|---|---|---|
| **1-2** | **Docs + AGENTS.md/README.md** | **Done** |
| **3-4** | **Lexer + Parser** (new syntax: `:=`, `::=`, `T?`, `final class`, `abstract class`) | **Current** |
| **5** | HIR (new crate) + desugaring | Planned |
| **6** | Semantic analysis (updated) | Planned |
| **7** | Move semantics (replaces refcounting) | Planned |
| **8** | Backend release mode | Planned |
| **9** | Async scheduler | Planned |
| **10** | Iterators | Planned |
| **11** | Package manager | Planned |
| **12** | Tooling | Planned |
| **13** | Borrow checker | Planned |
| **14** | Alternative backends | Planned |

**Priority order:** Docs and spec first, then lexer/parser, then semantic
pipeline, then move semantics, then async/iterators, then tooling.

---

## 2. Phase 1-2: Docs + AGENTS.md/README.md

**Status:** Done. All documentation has been rewritten to reflect the new
variable syntax (`=`, `:=`, `::=`), type syntax (`T?`), and class keywords
(`final class`, `abstract class`).

### Key Milestones

- AGENTS.md rewritten with new variable/type/keyword decisions
- Language reference (docs/01-language-reference.md) updated to v0.3.0 with
  full new syntax: `=`, `:=`, `::=`, `T?`, `T!`, `final class`, `abstract class`
- Types/Errors/Memory doc updated for new optional syntax
- Modules/Packages/Tooling doc updated
- Compiler architecture doc updated with new pipeline
- All Rust tests passing (101+)

---

## 3. Phase 3-4: Lexer + Parser

**Status:** Lexer tokens are complete. Parser declaration-level syntax is
complete. Statement-level parsing and advanced features are in progress.

### 3.1 Lexer (Done)

| Token | Status |
|---|---|
| `Walrus` (`:=`) for mutable declaration | Done |
| `ConstDecl` (`::=`) for constant declaration | Done |
| `Abstract` keyword | Done (both `abstract` and legacy `abs`) |
| `Final` keyword (for `final class`) | Done |
| `DotDotEquals` (`..=`) for inclusive ranges | Done |
| `DotDotLess` (`..<`) for half-open ranges | Done |
| `Mut` keyword | Removed |

### 3.2 Lexer Remaining

| Token | Status |
|---|---|
| `At` (`@`) for attribute syntax `#[attr]` | Not yet implemented |
| `///` doc comments (currently `##`) | Not yet implemented |

### 3.3 Parser (Done)

| Construct | Status |
|---|---|
| `name = expr` (immutable declaration) | Done |
| `name := expr` (mutable declaration) | Done |
| `name ::= expr` (constant declaration) | Done |
| `abstract class Name:` | Done |
| `struct Name:` (temporary alias) | Done |
| Type-annotated variables `name: T = expr` | Done |
| Statement-level `=`, `:=`, `::=` | Done |

### 3.4 Parser Remaining

| Construct | Status |
|---|---|
| `final class Name:` with no inheritance | Not yet implemented (Final token exists) |
| Destructuring `(x, y) = expr` | Not yet implemented |
| `if let pattern = expr:` | Not yet implemented |
| `while let pattern = expr:` | Not yet implemented |
| Error recovery (panic mode, multiple errors) | Not yet implemented |

### 3.5 Known Issues

- `__name` visibility stripping in parser is confusing: the leading `__` prefix
  is stripped from the identifier and used to set visibility. This should be
  revisited for clarity.
- `.kl` examples in `examples/` and `examples/kyle-test/` still use old syntax
  (`mut` keyword instead of `:=`, `Option<T>` instead of `T?`, etc.). They need
  to be rewritten.
- `Final` keyword exists in the lexer, but `final class Name:` is not parsed.
  Currently `struct` is kept as a temporary alias.
- `@` token for attributes is missing from lexer.
- Doc comments use `##`; should eventually support `///`.

---

## 4. Phase 5: HIR (New Crate) + Desugaring

**Status:** Planned.

| Item | Status |
|---|---|
| Create `klc_hir` crate | Planned |
| Define HIR types (after desugaring) | Planned |
| Desugar `T?` to internal `Option<T>` | Planned |
| Desugar `:=` to internal mut flag | Planned |
| Pass HIR to semantic analysis | Planned |

---

## 5. Phase 6: Semantic Analysis (Updated)

**Status:** Planned.

| Item | Status |
|---|---|
| `T?` type checking | Planned |
| `:=` mutability checking (replaces `mut`) | Planned |
| `::=` constant evaluation checking | Planned |
| Destructuring type checking | Planned |
| `if let` / `while let` type checking | Planned |
| Abstract method enforcement | Planned |
| Or-patterns and match guard completion | Planned |
| Default params | Planned |
| Properties (get/set) | Planned |

---

## 6. Phase 7: Move Semantics

**Status:** Planned. Replaces compiler-inferred refcounting with a proper
move semantics model.

| Item | Status |
|---|---|
| Define Copy vs Move types | Planned |
| Flow analysis for use-after-move detection | Planned |
| Replace ownership pass (refcounting) with move analysis | Planned |
| Implement `.clone()` for Move types | Planned |
| Keep refcount only for `Rc`/`Arc` in stdlib | Planned |
| Memory safety tests | Planned |

---

## 7. Phase 8: Backend Release Mode

**Status:** Planned.

| Item | Status |
|---|---|
| Connect `--release` to `OptimizationLevel::Aggressive` | Planned |
| Verify `-O2` / `-O3` works | Planned |

---

## 8. Phase 9: Async Scheduler

**Status:** Planned.

| Item | Status |
|---|---|
| `async fn name():` syntax | Planned |
| State machine generation | Planned |
| Work-stealing scheduler (tokio-style) | Planned |
| Maintain `async expr` as task in scheduler | Planned |

---

## 9. Phase 10: Iterators

**Status:** Planned.

| Item | Status |
|---|---|
| Trait `Iterable<T>` in stdlib | Planned |
| `iter()` on lists, dicts, ranges, strings | Planned |
| `map`, `filter`, `fold`, `reduce`, `collect` | Planned |
| Lazy evaluation | Planned |

---

## 10. Phase 11: Package Manager

**Status:** Planned.

| Item | Status |
|---|---|
| `kl.package` / `kl.lock` | Planned |
| Version resolution (semver) | Planned |
| `kl publish` + registry | Planned |
| `kl doc` with `///` comments | Planned |

---

## 11. Phase 12: Tooling

**Status:** Planned.

| Item | Status |
|---|---|
| `#[test]` attribute -> `kl test` | Planned |
| Formatter updated for new syntax | Planned |
| LSP updated for new syntax | Planned |
| VS Code extension updated | Planned |

---

## 12. Phase 13: Borrow Checker

**Status:** Low priority.

| Item | Status |
|---|---|
| `&T` and `&mut T` references | Planned |
| Region inference (no lifetime annotations) | Planned |
| Compatibility with move semantics | Planned |

---

## 13. Phase 14: Alternative Backends

**Status:** Low priority.

| Item | Status |
|---|---|
| Cranelift backend | Planned |
| WASM target | Planned |

---

## 14. Feature Matrix

| Feature | Phase | Status |
|---|---|---|
| Indentation-based blocks (4 spaces) | 1-2 | Done |
| Immutable `name = value` | 1-2 | Done |
| Mutable `name := value` | 1-2 | Done |
| Constant `name ::= value` | 1-2 | Done |
| `T?` optional type syntax | 1-2 | Done |
| `T!` error-returning type | 1-2 | Done |
| `final class` (lightweight, no inheritance) | 3-4 | Pending (token exists) |
| `abstract class` | 3-4 | Done |
| `struct` (temporary alias for `final class`) | 1-2 | Done |
| `i8`/`i16`/`i32`/`i64`/`u8`/`u16`/`u32`/`u64`/`f32`/`f64` | 1-2 | Done |
| `bool`, `str`, `char`, `void`, `any` | 1-2 | Done |
| `ptr` raw pointer type | 3-4 | Pending |
| Integer/float/string/char/boolean literals | 1-2 | Done |
| `None` / `null` literals | 3-4 | Pending (null) |
| Arithmetic, comparison, logical, bitwise operators | 1-2 | Done |
| `..=` inclusive range | 3-4 | Done |
| `..<` half-open range | 3-4 | Done |
| Assignment operators (`+=`, `-=`, etc.) | 1-2 | Done |
| Ternary operator (`cond ? a : b`) | 1-2 | Done |
| Optional chaining (`?.`) | 1-2 | Done |
| Error propagation (`?`) | 1-2 | Done |
| Member access (`.field`, `.method()`, `[]`) | 1-2 | Done |
| Functions (typed params, generic, return type) | 1-2 | Done |
| Default argument values | 5 | Pending |
| Variadic parameters (`...names`) | 5 | Pending |
| Error-returning functions (`-> T!`) | 1-2 | Done |
| Async expression (`async expr`) | 1-2 | Done |
| Const functions (`const fn`) | 5 | Pending |
| Public / protected / private visibility | 1-2 | Done |
| If / elif / else | 1-2 | Done |
| While loops | 1-2 | Done |
| For-in-list, For-in-range | 1-2 | Done |
| While-else, For-else | 5 | Pending |
| Loop, break, continue | 1-2 | Done |
| Match (patterns) | 1-2 | Done |
| Match or-patterns (`a \| b`) | 6 | Pending |
| Match guard (`if cond`) | 6 | Pending |
| Match is-type (`x is T`) | 6 | Pending |
| `if let pattern = expr:` | 3-4 | Pending |
| `while let pattern = expr:` | 3-4 | Pending |
| Destructuring `(x, y) = expr` | 3-4 | Pending |
| Defer | 1-2 | Done |
| Guard | 1-2 | Done |
| Unsafe blocks | 1-2 | Done |
| Enums (generic, with payload) | 1-2 | Done |
| Classes (fields, methods, constructor) | 1-2 | Done |
| Single inheritance + method override | 1-2 | Done |
| Contracts (interfaces) | 1-2 | Done |
| Type aliases | 1-2 | Done |
| Properties (get/set) | 6 | Pending |
| Closures (untyped params) | 1-2 | Done |
| Closure typed params `(x: T) =>` | 5 | Pending |
| Function pointer type `(T) -> U` | 6 | Pending |
| Lists (literal, index, slice, spread) | 1-2 | Done |
| Dicts (string keys, literal, index) | 1-2 | Done |
| Tuples `(a, b, c)` | 5 | Pending |
| Built-in functions | 1-2 | Done |
| Standard library (8 modules) | 1-2 | Done |
| Imports (`import x`, `from x import y`, `import ~x`) | 1-2 | Done |
| Import alias (`import x as y`, `from x import y as z`) | 1-2 | Done |
| RAII / compiler-inferred refcounting | 1-2 | Done |
| String interpolation | 1-2 | Done |
| `int()`, `float()`, `bool()` builtins | 1-2 | Done |
| `range(n)`, `range(a, b)` | 1-2 | Done |
| `this` instance reference | 1-2 | Done |
| `super` keyword | 5 | Pending |
| `as` casting | 6 | Pending |
| `is` operator | 6 | Pending |
| `#[attr]` attributes | 3-4 | Pending |
| `///` doc comments | 3-4 | Pending |
| Labeled loops | 5 | Pending |
| Doc comments (`##`) | 1-2 | Done |

---

## 15. Test Counts

| Suite | Count | Status |
|---|---|---|
| `klc_frontend` unit tests | 80 | All passing |
| `klc_semantic` unit tests | 17 | All passing |
| `klc_mir` unit tests | 4 | All passing |
| `klc_runtime` unit tests | 0 | n/a (C-ABI) |
| `klc_tools` unit tests | 0 | n/a (LSP) |
| `klc_backend` unit tests | 0 | n/a |
| `klc_core` unit tests | 0 | n/a |
| `klc_driver` unit tests | 0 | n/a |
| `klc_cli` unit tests | 0 | n/a |
| Example programs (`examples/*.kl`) | 55+ | All compile (old syntax) |
| End-to-end syntax tests (`examples/kyle-test/tests/`) | 12 | All pass `kl test` |
| Total Rust unit tests | 101 | All passing |

---

## 16. v1.0 Release Checklist

> A release v1.0 will be cut when every item below is checked.

### 16.1 Phases 1-2: Documentation

- [x] AGENTS.md rewritten with new syntax
- [x] Language reference updated to v0.3.0
- [x] Types/Errors/Memory doc updated
- [x] Modules/Packages/Tooling doc updated
- [x] Compiler architecture doc updated

### 16.2 Phase 3-4: Lexer + Parser

- [x] Walrus (`:=`), ConstDecl (`::=`) tokens
- [x] Abstract, Final keyword tokens
- [x] DotDotEquals (`..=`), DotDotLess (`..<`) tokens
- [x] Declaration-level `=`, `:=`, `::=` parsing
- [x] `abstract class Name:` parsing
- [ ] `final class Name:` parsing
- [ ] `@` (At) token for attributes
- [ ] `///` doc comment lexing
- [ ] Destructuring `(x, y) = expr`
- [ ] `if let pattern = expr:`
- [ ] `while let pattern = expr:`
- [ ] Error recovery in parser
- [ ] All .kl examples rewritten with new syntax

### 16.3 Phase 5: HIR + Desugaring

- [ ] `klc_hir` crate exists
- [ ] `T?` desugared to internal `Option<T>`
- [ ] `:=` desugared to internal mut flag
- [ ] HIR passes to semantic analysis

### 16.4 Phase 6: Semantic Analysis

- [ ] `T?` type checking
- [ ] `:=` mutability checking
- [ ] `::=` constant evaluation checking
- [ ] Destructuring type checking
- [ ] `if let` / `while let` type checking
- [ ] Abstract method enforcement
- [ ] Or-patterns and match guard filtering
- [ ] Default params
- [ ] Properties (get/set)

### 16.5 Phase 7: Move Semantics

- [ ] Copy vs Move type distinction
- [ ] Use-after-move detection
- [ ] Refcounting replaced by move analysis
- [ ] `.clone()` for Move types
- [ ] Memory safety tests

### 16.6 Phase 8: Release Mode

- [ ] `--release` uses `OptimizationLevel::Aggressive`
- [ ] `-O2` / `-O3` verified

### 16.7 Phase 9: Async Scheduler

- [ ] `async fn name():` declaration syntax
- [ ] State machine generation
- [ ] Work-stealing scheduler
- [ ] `async expr` integration

### 16.8 Phase 10: Iterators

- [ ] `Iterable<T>` trait in stdlib
- [ ] `iter()` on core types
- [ ] `map`, `filter`, `fold`, `reduce`, `collect`
- [ ] Lazy evaluation

### 16.9 Phase 11: Package Manager

- [ ] `kl.package` / `kl.lock`
- [ ] Semver version resolution
- [ ] `kl publish` + registry
- [ ] `kl doc` with `///`

### 16.10 Phase 12: Tooling

- [ ] `#[test]` attribute -> `kl test`
- [ ] Formatter updated for new syntax
- [ ] LSP updated for new syntax
- [ ] VS Code extension updated

### 16.11 Phase 13: Borrow Checker

- [ ] `&T` and `&mut T` references
- [ ] Region inference
- [ ] Compatibility with move semantics

### 16.12 Phase 14: Alternative Backends

- [ ] Cranelift backend
- [ ] WASM target

### 16.13 Documentation

- [ ] All docs reviewed for accuracy
- [ ] Test matrix 100% green
- [ ] Migration guide for Python / Rust / Go developers

---

## 17. Key Design Decisions (Frozen)

These are **not** subject to change without a major version bump.

| Decision | Choice |
|---|---|
| Block syntax | Indentation, 4 spaces |
| Statement terminator | Newline (no semicolons) |
| Variable mutability | `=` immutable, `:=` mutable, `::=` constant |
| `mut` / `let` / `var` / `const` keywords | None — operator signals mutability |
| Instance reference | `this` (not `self`) |
| Optional type | `T?` (sugar for internal `Option<T>`) |
| Error type | `T!` return type |
| Error propagation | `?` operator |
| Exceptions | None — errors are values |
| `{` `}` for blocks | None — indentation |
| Visibility | Convention: `_` protected, `__` private, none public |
| Class keyword | `final class` (no inheritance), `class` (inheritable) |
| Abstract class | `abstract class` (not `abs class`) |
| String encoding | Null-terminated UTF-8 bytes |
| Integer overflow | Panic in debug, wrapping in release (planned) |
| Entry point | `fn main(args: [str]) -> i32` in `src/main.kl` |
| Memory | Move semantics (planned), Copy types + Clone |
| Compiler implementation | Pure Rust, LLVM 18 via `inkwell` |
| Runtime implementation | Pure Rust with `extern "C"` |
| CLI binary name | `kl` (primary), `klc` (legacy alias) |

---

## 18. Current Release (v0.3.0)

### What was added since v0.2.2

- **New variable syntax**
  - `name = value` — immutable declaration (unchanged)
  - `name := value` — mutable declaration (replaces `mut` keyword)
  - `name ::= value` — constant declaration (replaces UPPERCASE convention)
  - `mut` keyword removed from lexer and parser

- **New type syntax**
  - `T?` postfix syntax for optional types (replaces public `Option<T>`)
  - `T!` stays as error-returning type

- **New keywords and tokens**
  - `Walrus` (`:=`) token for mutable declaration
  - `ConstDecl` (`::=`) token for constant declaration
  - `Abstract` keyword (both `abstract` and legacy `abs`)
  - `Final` keyword (for `final class`)
  - `DotDotEquals` (`..=`) for inclusive ranges
  - `DotDotLess` (`..<`) for half-open ranges

- **Parser updates**
  - Declaration-level `=`, `:=`, `::=` for variable declarations
  - Statement-level `=`, `:=`, `::=` for rebinding
  - `abstract class Name:` support (with `abs` backward compat)
  - `struct` kept as temporary alias for `final class`

- **Documentation**
  - AGENTS.md completely rewritten as master context for AI agents
  - Language reference (v0.3.0) with all new syntax documented
  - Types/Errors/Memory document updated for `T?` and `T!`
  - Modules/Packages/Tooling document updated
  - Compiler architecture document updated
  - Roadmap rewritten with new 14-phase evolution plan

- **Tests**
  - All 101 Rust unit tests passing
  - 12 end-to-end syntax test files passing via `kl test`

### Known Issues (v0.3.0)

- `__name` visibility prefix stripping needs fixing
- `.kl` examples not yet rewritten with new syntax
- `Final` keyword exists in lexer but `final class` not fully parsed yet
- `@` (At) token for attributes not yet implemented
- `///` doc comments not yet implemented (using `##` instead)
- Destructuring, `if let`, `while let` not yet implemented

---

*Version: v0.3.0 · Last updated: 2026-06-28*
