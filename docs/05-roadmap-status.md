# Roadmap & Status

> The phase-by-phase plan, the implementation matrix, and the v1.0 release
> checklist. This is the single source of truth for "what's done, what's
> next, and what we'll ship at v1.0."

---

## 1. Phase Overview

| Phase | Focus | Status |
|---|---|---|
| **0–6** | Language design, compiler, all syntax features | ✅ Complete |
| **7** | Cross-platform: macOS ARM + Linux ARM | ✅ Complete |
| **8** | Distribution and tooling polish | ✅ Complete |
| **9** | Backend & systems: FFI, HTTP, DB, ENV | 🔜 Next |
| **10** | Std library ergonomics: iterators, collections | 📅 Planned |
| **11** | Production hardening: errors, DWARF, TLS, WASM | 📅 Planned |
| **12** | Self-hosting (compiler in Kyle) | ⏸️ Deferred |
| **13** | Ecosystem: registry, framework, website | 📅 Future |

---

## 2. Phase 0–6: The Language

**Status:** ✅ Complete. Every language feature listed in
[`docs/01-language-reference.md`](01-language-reference.md) under the
"Constructs" sections is implemented, type-checked, and codegenned. The
test suite has 101 passing unit tests.

Key milestones:

- ✅ Lexer + parser for full grammar
- ✅ Type checker with generics, type inference, contracts
- ✅ MIR with control flow, classes, generics
- ✅ LLVM 18 codegen (inkwell)
- ✅ Runtime: memory, string, list, dict, I/O, async
- ✅ 8 standard library modules
- ✅ Formatter, LSP, VS Code extension
- ✅ Package manager (manifest)
- ✅ 50+ working examples

---

## 3. Phase 7: Cross-Platform Support

**Status:** ✅ Complete. Two platform binaries are built and released
automatically via GitHub Actions.

| Platform | Status | Notes |
|---|---|---|
| macOS ARM (Apple Silicon) | ✅ | Tested, primary dev platform |
| Linux ARM (aarch64) | ✅ | Tested, CI runs on `ubuntu-24.04-arm` |
| Linux x64 (x86_64) | 📅 | Planned (Phase 9+) |
| macOS Intel (x86_64) | 📅 | Planned (Phase 9+) |
| Windows x64 | 📅 | Planned (Phase 11+) |

---

## 4. Phase 8: Distribution & Tooling Polish

**Status:** ✅ Complete. Everything in this phase is shipped in v0.2.2.

| Item | Status |
|---|---|
| One-line installer (`install.sh`) | ✅ |
| `kl` and `klc` binary aliases | ✅ |
| GitHub Actions CI on every push | ✅ |
| Tag-based release pipeline | ✅ |
| VS Code `.vsix` packaged in releases | ✅ |
| LSP semantic tokens (variables colored) | ✅ |
| Formatter (`kl fmt`) | ✅ |
| Auto-generated documentation | ✅ (the 6 docs in `docs/`) |
| Comprehensive README | ✅ |

---

## 5. Phase 9: Backend & Systems

**Status:** 🔜 Next. The next major milestone.

| Item | Priority | Status |
|---|---|---|
| FFI (`extern "C"`, `*T` raw pointers, `unsafe:` body lowering) | P0 | ❌ |
| HTTP server + client (`http.Server`, `http.Request`, `http.Response`) | P0 | ❌ |
| Process spawning (`process.spawn`, `process.exec`) | P1 | ❌ |
| Environment variables (`env.get`, `env.set`) | P1 | ❌ |
| File system (`fs.read`, `fs.write`, `fs.walk`) | P1 | ❌ |
| Database drivers (`db.sqlite`, `db.postgres`) | P2 | ❌ |
| Channels (`Channel<T>`, `ch.send`, `ch.recv`) | P2 | ❌ |
| Release optimization (`--release` → `-O2`) | P1 | ❌ |

---

## 6. Phase 10: Std Library Ergonomics

**Status:** 📅 Planned.

| Item | Status |
|---|---|
| Iterator trait + `iter()` method on lists | ❌ |
| `map`, `filter`, `fold`, `reduce` on lists | ❌ |
| `zip`, `enumerate`, `flatten` | ❌ |
| Custom sort (`sort_by`, `sort_with`) | ❌ |
| `Option` and `Result` convenience methods | ❌ |
| Real compile-time evaluation (`const fn`) | ❌ |

---

## 7. Phase 11: Production Hardening

**Status:** 📅 Planned.

| Item | Status |
|---|---|
| Ownership pass full coverage (no leaks) | ❌ |
| Cycle detection in refcounting | ❌ |
| DWARF debug info emission | ❌ |
| Stack traces in panic messages | ❌ |
| TLS / secure sockets | ❌ |
| WASM target | ❌ |
| Coroutine-based async (work-stealing) | ❌ |

---

## 8. Phase 12: Self-Hosting

**Status:** ⏸️ Deferred until Phase 11 is done. The compiler cannot host
itself until the language is fully stable.

| Item | Status |
|---|---|
| Lexer in Kyle (`examples/lexer.kl`) | ✅ exists |
| Parser in Kyle (`examples/parser.kl`) | ✅ exists |
| Semantic analyzer in Kyle (`examples/semantic.kl`) | ✅ exists |
| Replace Rust compiler with Kyle compiler | ❌ |

---

## 9. Phase 13: Ecosystem

**Status:** 📅 Future.

| Item | Status |
|---|---|
| Central package registry | ❌ |
| Web framework (Kyle web) | ❌ |
| Documentation hosting site | ❌ |
| Community Discord / forum | ❌ |

---

## 10. Implementation Matrix (Language Features)

| Feature | Phase | Status |
|---|---|---|
| Indentation-based blocks | 0 | ✅ |
| Immutable / mut / const variables | 0 | ✅ |
| i8/i16/i32/i64/u8/u16/u32/u64/f32/f64 | 0 | ✅ |
| `bool`, `str`, `char`, `void`, `any` | 0 | ✅ |
| Arithmetic, comparison, logical, bitwise operators | 0 | ✅ |
| Functions (typed params, generic, return type) | 0 | ✅ |
| If / elif / else | 0 | ✅ |
| While, for-in-list, for-in-range | 0 | ✅ |
| Loop, break, continue | 0 | ✅ |
| Match (literal, identifier, wildcard, enum patterns) | 0 | ✅ |
| Match as expression | 0 | ✅ |
| Structs (generic) | 0 | ✅ |
| Enums (generic, payload) | 0 | ✅ |
| Classes (fields, methods, constructor) | 0 | ✅ |
| Single inheritance | 0 | ✅ |
| Method override (polymorphism) | 0 | ✅ |
| Public / protected / private visibility | 8 | ✅ |
| Abstract classes | 0 | ✅ (partial — no `abs fn` enforcement) |
| Contracts (interfaces) | 0 | ✅ (no generic constraints) |
| Closures | 0 | ✅ |
| Async / await (expression form) | 0 | ✅ |
| Lists (with spread, index, slice) | 0 | ✅ |
| Dicts (string keys) | 0 | ✅ |
| Error values (`T!`, `?`) | 0 | ✅ |
| Optional chaining (`?.`) | 0 | ✅ |
| Imports (4 forms + relative) | 0 | ✅ |
| Type aliases | 0 | ✅ |
| String interpolation | 0 | ✅ |
| Built-in functions (~30) | 0 | ✅ |
| Standard library (8 modules) | 0 | ✅ |
| RAII / refcounting | 0 | ✅ |
| Pattern: or-patterns (`a \| b`) | 10 | ❌ |
| Pattern: match guard (`if cond`) | 10 | 🔶 parsed, not filtering |
| Pattern: is-type (`x is T`) | 10 | ❌ |
| Properties (get/set) | 9 | ❌ |
| FFI (`extern "C"`) | 9 | ❌ |
| Raw pointer types (`*T`) | 9 | ❌ |
| `async fn name():` (function form) | 10 | ❌ |
| `?:` default operator | 10 | ❌ |
| `Channel<T>` (language-level) | 9 | ❌ |
| Compile-time evaluation | 10 | 🔶 type-checks only |
| `**` power operator (correct lowering) | 9 | 🔶 lowered as mul |
| `+%`, `-%`, `*%` percent operators | 9 | 🔶 no semantic meaning |

---

## 11. Test Counts

| Suite | Count | Status |
|---|---|---|
| `klc_core` unit tests | varies | ✅ |
| `klc_frontend` unit tests | 80 | ✅ |
| `klc_semantic` unit tests | 17 | ✅ |
| `klc_mir` unit tests | 4 | ✅ |
| `klc_runtime` unit tests | 0 | n/a (C-ABI) |
| `klc_tools` unit tests | 0 | n/a (LSP) |
| Example programs (`examples/*.kl`) | 55 | ✅ all compile |
| Standard library test suite | 0 | 📅 Phase 11 |

**Total unit tests:** 101 passing, 0 failing.

---

## 12. v1.0 Release Checklist

> A release v1.0 will be cut when every item below is checked.

### 12.1 Language

- [ ] All `01-language-reference.md` constructs marked ✅
- [ ] `**` power operator correctly lowered
- [ ] Match or-patterns (`a | b`)
- [ ] Match guard `if cond` filtering
- [ ] Compile-time evaluation works for `const fn`
- [ ] No known syntax bugs

### 12.2 Tooling

- [ ] LSP semantic tokens stable
- [ ] Formatter handles all valid code
- [ ] VS Code extension published to marketplace
- [ ] `kl completions` for bash, zsh, fish

### 12.3 Performance

- [ ] Release builds use `-O2` or `-O3`
- [ ] Ownership pass has no known leaks
- [ ] Cycle detection in refcounting
- [ ] Standard library benchmarked against equivalent C/Rust code

### 12.4 Platforms

- [ ] Linux x64 binary released
- [ ] macOS Intel binary released
- [ ] Windows x64 binary released
- [ ] CI runs on all four platforms

### 12.5 Ecosystem

- [ ] At least 3 third-party packages on the registry
- [ ] `kl` aliases both stable
- [ ] Website up with tutorials
- [ ] Community Discord active

### 12.6 Documentation

- [ ] All 6 docs reviewed for accuracy
- [ ] Test matrix 100% green
- [ ] Tutorial series published (beginner → advanced)
- [ ] Migration guide for Python / Rust / Go developers

---

## 13. Key Design Decisions (Frozen)

These are **not** subject to change without a major version bump.

| Decision | Choice |
|---|---|
| Block syntax | Indentation, 4 spaces |
| Statement terminator | Newline (no semicolons) |
| Variable mutability | `mut` keyword required to make mutable |
| Constants | UPPERCASE convention, no keyword |
| Instance reference | `this` (not `self`) |
| Optional type | `Option<T>` (not `T?`) |
| Error type | `T!` return type |
| Error propagation | `?` operator |
| Exceptions | None — errors are values |
| `let` / `var` keywords | None — `mut` keyword directly |
| `{` `}` for blocks | None — indentation |
| Visibility | Convention: `_` protected, `__` private, none public |
| String encoding | Null-terminated UTF-8 bytes |
| Integer overflow | Panic in debug, wrapping in release (planned) |
| Entry point | `fn main(args: [str]) -> i32` in `src/main.kl` |
| Memory | RAII + compiler-inferred refcounting (no GC, no manual free) |
| Compiler implementation | Pure Rust, LLVM 18 via `inkwell` |
| Runtime implementation | Pure Rust with `#[unsafe(no_mangle)] extern "C"` |
| CLI binary name | `kl` (primary), `klc` (legacy alias) |

---

## 14. What's in the Next Release (v0.2.2)

This is what was added since v0.2.1:

- **Bug fixes**
  - `input("prompt")` now accepts 1 argument (was 0)
  - `open(path, mode)` now correctly expects 2 arguments
  - `read_str(fd, count)` now correctly expects 2 arguments
  - `class X: Parent` (single colon) is now accepted by the parser
  - Private methods (`__method`) are now blocked from outside the class
  - Classes without a constructor get a synthetic default constructor
  - Inheritance walks the parent chain for both fields and methods
  - LSP autocompletion now uses the correct character position
  - Error source label changed from `klc` to `kl`

- **New features**
  - 5 new example programs (`input_prompt_test`, `class_greet_test`,
    `inheritance_test`, `polymorphism_test`, `private_method_test`)
  - Private method convention: declare with `__name`, call with `this.name`
  - Method override and dynamic dispatch through parent chain

- **Documentation**
  - 5 docs rewritten with full test matrix and checkboxes
  - 6 docs total: vision, language reference, types/errors/memory, modules,
    architecture, roadmap

---

*Version: v0.2.2 · Last updated: 2026-06-27*
