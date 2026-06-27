# Kyle Language — Roadmap & Status

> **Source of truth** for implementation status, phase tracking, and release planning.

---

## Phase Overview

```
Phase 0-6:   Language Design + Compiler + Language Completion    ✅ Complete
Phase 7:     Cross-Platform Support                              🔶 Current
Phase 8:     Distribution & Tooling Polish                        🔶 Current
Phase 9:     Backend & Systems (FFI, HTTP, DB, Net)               ⏸️ Next
Phase 10:    Std Library & Ergonomics (Iterators, Collections)   📅 Planned
Phase 11:    Production Hardening (Errors, Debug, Security)      📅 Planned
Phase 12:    Self-Hosting                                         ⏸️ Deferred
Phase 13:    Ecosystem (Registry, WASM, Framework)               📅 Future
```

**Memory model:** RAII + Compiler-Inferred Ownership (no garbage collector).

**Goal for v1.0:** A developer can install Kyle, create a project, connect to
PostgreSQL, build a REST API, serialize JSON, authenticate users, run tests,
and deploy to production — simply and with good performance.

---

## Phase 0: Language Design ✅

All specification documents written and frozen.

- [x] Language vision and philosophy
- [x] Formal grammar (EBNF)
- [x] Type system design
- [x] Error system design
- [x] Module system design
- [x] Memory model (RAII + Ownership Inference)
- [x] Compiler architecture plan

---

## Phase 1: Compiler Frontend ✅

- [x] Lexer (80+ token types, INDENT/DEDENT, escapes, char literals)
- [x] Parser (recursive descent, 12 precedence levels, all declarations)
- [x] AST (all node types: Program, Decl×9, Stmt×14, Expr×25+, Pattern, Type)
- [x] Span tracking (real line/column/offset for all nodes)

---

## Phase 2: Semantic Analysis ✅

- [x] Symbol table + scope resolution
- [x] Type inference (Hindley-Milner)
- [x] Type checking with diagnostics
- [x] Generics (type params, fresh var instantiation)
- [x] Contracts (validation, `impl` keyword)
- [x] Error types (`!` return, `?` operator)
- [x] Optional types (`None`, `?.` chain)
- [x] Pattern binding in match
- [x] Variable auto-declare (`ident = expr`)

---

## Phase 3: MIR + Backend ✅

- [x] MIR definition (MirValue, MirInst, MirFunction, MirModule)
- [x] AST → MIR lowering (all constructs)
- [x] Constant folding + dead code elimination
- [x] Ownership inference pass (RAII retain/release)
- [x] LLVM 18.1 codegen via inkwell (opaque pointers)
- [x] Struct pass-by-reference (pointer ABI)
- [x] Enum tagged unions
- [x] Closures (FnAddr + CallIndirect)
- [x] Async/await (AsyncSpawn + AsyncAwait via thread spawn)
- [x] Object file emission + native linking (clang)
- [x] LLVM IR verification (dumps IR on failure)

---

## Phase 4-5: Runtime, Std Library, Tooling ✅

### Runtime (klc_runtime/)
- [x] print/println, str representation
- [x] kl_alloc / kl_free (RAII heap)
- [x] String ops: contains, to_upper, to_lower, trim, replace, concat, input
- [x] Char ops: char_at, is_digit, is_alpha, is_alnum, is_whitespace, is_upper, is_lower, ord
- [x] File I/O: open, read_str, write_str, close
- [x] Time: sleep, now
- [x] List ops: new, add, get, len, pop, slice, extend, range
- [x] Dict ops: new, set, get, len, free
- [x] Thread spawn/join (kl_spawn_thread, kl_join_thread)
- [x] Async runtime + Channel<T>
- [x] Panic handler + entry_point wrapper

### Standard Library (std/)
- [x] std/core.kl — Option<T>, unwrap_or, is_some, is_none
- [x] std/math.kl — abs, pow, sqrt, gcd, min, max, clamp
- [x] std/io.kl — File read/write wrappers
- [x] std/testing.kl — assert, assert_eq, assert_str, assert_ne
- [x] std/str.kl — starts_with, ends_with, capitalize, repeat_str
- [x] std/collections.kl — list_sum, list_product, list_max, list_min, list_range
- [x] std/json.kl — json_parse + json_stringify (via runtime FFI)
- [x] std/time.kl — timestamp, sleep_ms, seconds_since

### Tooling
- [x] CLI: `kl` (primary) + `klc` (legacy) — build, run, check, parse, mir, fmt, test, new, add, remove, info, lsp
- [x] Package manager: kl.toml manifest, kl.lock, add/remove/info
- [x] Formatter: pretty-printer + comment preservation
- [x] LSP: documentSymbol, workspace/symbol, signatureHelp, findReferences, codeAction, completion, definition, hover, dot-completions, scope-aware completions
- [x] VS Code extension: syntax highlighting, LSP client, commands, snippets, .vsix packaged (kl-0.2.0.vsix)
- [x] CI pipeline (.github/workflows/ci.yml)

---

## Phase 6: Language Completion ✅

All language syntax generates working code end-to-end.

### P0 — Core features
- [x] For loops (list + range + for-else)
- [x] Generics (structs + functions, monomorphization)
- [x] Error handling (`!` / `?` operator)
- [x] String interpolation (concat + `str()`)
- [x] Optional chaining (`?.`)
- [x] While-else

### P1 — Secondary features
- [x] Defer (LIFO lowering)
- [x] Guard (early return via CondBr)
- [x] Type aliases (with chained resolution)
- [x] Dict/Map literals + indexing + `.len()`
- [x] Spread operator (`...` in list literals)
- [x] Range slicing (`list[start..end]`)
- [x] Ternary operator (`cond ? a : b`)
- [x] Match-expression (expression context)
- [x] const fn (type-checker validation)

### P3 — Standard library
- [x] All 8 std modules (see Phase 4-5 above)

---

## Phase 7: Cross-Platform Support 🔶

```
[ ] Platform       Status
 бок macOS ARM      ✅ Working (linker warning fixed)
 бок Linux ARM      ✅ Working (Docker)
 бок macOS Intel   ❌ Untested
 бок Linux x64      ❌ Untested
 бок Windows x64   ❌ Untested
 бок Windows ARM   ❌ Untested
```

### Tasks
- [x] Runtime I/O cross-platform (std::fs + std::io)
- [x] Target triple auto-detection (LLVM get_default_triple)
- [x] Test on macOS Apple Silicon ✅
- [x] Test on Linux ARM ✅
- [ ] Linker — multiplatform support (detect clang/gcc/link.exe)
- [ ] CLI — `.exe` extension on Windows
- [ ] LLVM paths — platform-specific config
- [ ] VS Code — Windows binary detection
- [ ] Test on Linux x64
- [ ] Test on macOS Intel
- [ ] Test on Windows x64

---

## Phase 8: Distribution & Tooling Polish 🔶

### P0 — VS Code Extension
- [x] Syntax highlighting (TextMate grammar)
- [x] Language config (brackets, auto-closing, indentation)
- [x] LSP client (launches `kl lsp`)
- [x] Commands (run/build/check via terminal)
- [x] Snippets (20 snippets)
- [x] Extension icon (PNG 128×128 + 16×16)
- [x] .vsix packaged (kl-0.2.0.vsix, 14 files)

### P1 — LSP Completion
- [x] Basic completion (44 builtins, 8 Decl types, 33 keywords, prefix filter)
- [x] Dot-triggered completion (struct.field, object.method, str/list/dict methods)
- [x] Scope-aware completion (params, auto-declared, block walks)
- [x] Go-to-definition (8 declaration types)
- [x] Hover (function sigs + builtin docs + identifier info)
- [x] Error squiggles (diagnostics → VS Code via publishDiagnostics notification)
- [ ] Completion resolve provider
- [x] Compile-on-save (type-check on save via didSave handler)
- [x] LSP rename (textDocument/rename + prepareRename)
- [x] LSP formatting (via formatter, textDocument/formatting handler)

### P2 — Build & Project
- [x] `kl new` creates professional template (src/main.kl with args, tests/, .gitignore, kl.toml)
- [x] Build output to `target/debug/` and `target/release/`
- [x] `--release` flag on `kl build` and `kl run`
- [x] `.gitignore` (target/, *.klc-build/, kl.lock)
- [x] CLI tab completion (`kl completions bash|zsh|fish`)

### P3 — Branding
- [x] SVG logo
- [x] PNG icons (128×128, 16×16)
- [ ] kl-lang.org website
- [ ] Homebrew tap

### P4 — CI
- [x] GitHub Actions CI (build + tests + examples)

---

## Phase 9: Backend & Systems ⏸️ Next

**Goal:** Enable Kyle to build backend services — HTTP APIs, database apps, CLI
tools. This is the phase that makes Kyle **usable for real projects**.

### P0 — FFI (Foreign Function Interface) — CRITICAL
Without FFI, Kyle cannot call C libraries (libpq, sqlite3, system APIs).

- [ ] `extern "C"` block declarations
- [ ] Raw pointer types (`*T`, `*void`)
- [ ] `unsafe` blocks (already parsed, needs lowering)
- [ ] C struct interop (passing structs to/from C functions)
- [ ] Linking with external C libraries (`-l<lib>` flag)

```kl
extern "C":
    fn PQconnectdb(conninfo: str) -> *PGconn

unsafe:
    conn = PQconnectdb("postgresql://localhost/app")
```

### P1 — HTTP Server & Sockets — CRITICAL
- [ ] TCP server/listen (`std/net.kl`)
- [ ] TCP client connect
- [ ] HTTP server (GET, POST, PUT, PATCH, DELETE)
- [ ] HTTP routing + path params
- [ ] HTTP middleware (chain of handlers)
- [ ] Async I/O for concurrent connections

```kl
import http

app = http.Server()
app.get("/users", get_users)
app.listen(8080)
```

### P1 — HTTP Client — CRITICAL
- [ ] `http.get(url)`, `http.post(url, body)`
- [ ] Request/response objects
- [ ] Headers, query params, JSON body

### P2 — Database Drivers — CRITICAL
- [ ] SQLite driver (via FFI to libsqlite3)
- [ ] PostgreSQL driver (via FFI to libpq)
- [ ] Connection pooling
- [ ] Parameterized queries (SQL injection prevention)
- [ ] Transaction support

```kl
import db

pool = db.sqlite.open("app.db")
row = pool.query_one("SELECT * FROM users WHERE id = ?", [id])?
```

### P2 — Environment Variables — ESSENTIAL
- [ ] `env.get("VAR")` runtime function
- [ ] `.env` file parsing
- [ ] Type conversion helpers (`env.get_int`, `env.get_bool`)

### P2 — Process Spawning — ESSENTIAL
- [ ] `process.run(cmd, args)` → stdout/stderr capture
- [ ] Exit code handling
- [ ] Pipe/stream inheritance

```kl
import process
output = process.run("git", ["status"])?
```

### P3 — JSON Auto-Serialization — HIGH
- [ ] `json.stringify(struct_instance)` → automatic schema inference
- [ ] `json.parse::<T>(str)` → automatic struct deserialization
- [ ] No boilerplate annotations needed (zero-cost derive)

```kl
struct User:
    name: str
    age: i32

u = User(name: "Ana", age: 30)
s = json.stringify(u)  # {"name":"Ana","age":30}
```

### P3 — Release Optimization — HIGH
- [ ] `--release` flag activates LLVM O2/O3 (currently always O0/Default)
- [ ] Dead code elimination in release
- [ ] Bounds check removal in release
- [ ] Benchmark: release vs debug performance comparison

### Status: LSP Intelligent Autocompletion ✅ (already done)
- [x] Dot-triggered completions (struct/class/enum/str/list/dict fields + methods)
- [x] Scope-aware completions (params, auto-declared, block walks)

---

## Phase 10: Std Library & Ergonomics 📅 Planned

**Goal:** Make Kyle productive for everyday programming — iterators, functional
operations, richer collections, logging, configuration.

### P0 — Iterators — CRITICAL
- [ ] Iterator protocol (`.iter()` on lists, dicts, strings)
- [ ] Lazy evaluation (no intermediate allocations)
- [ ] Iterator chaining (`.iter().map(...).filter(...).collect()`)

### P0 — Functional Operations — CRITICAL
- [ ] `map(fn)`, `filter(fn)`, `reduce(fn, init)`
- [ ] `find(fn)`, `any(fn)`, `all(fn)`
- [ ] `first()`, `last()`, `skip(n)`, `take(n)`
- [ ] `collect()` → list

```kl
squares = [1, 2, 3, 4, 5].iter().map(x => x * x).collect()
evens = [1, 2, 3, 4, 5].filter(x => x % 2 == 0)
sum = [1, 2, 3].reduce((a, b) => a + b, 0)
```

### P1 — Advanced Collections — HIGH
- [ ] HashMap (hash table with O(1) lookup)
- [ ] HashSet (unique elements)
- [ ] Queue (FIFO)
- [ ] Deque (double-ended)
- [ ] BinaryHeap (priority queue)

### P1 — Logging — HIGH
- [ ] `log.info(msg)`, `log.warn(msg)`, `log.error(msg)`, `log.debug(msg)`
- [ ] Log levels (configurable via env or config file)
- [ ] Structured logging (JSON output option)

### P1 — Configuration — HIGH
- [ ] Config file reading (TOML/JSON)
- [ ] Environment variable integration
- [ ] Secret management (read from env, never log)

### P2 — Contracts with Generic Constraints — MEDIUM
- [ ] `fn sort<T: Comparable>(items: [T])` — type bound syntax
- [ ] Constraint checking at monomorphization
- [ ] Built-in traits: Comparable, Hashable, Iterable

### P2 — Filesystem Utilities — MEDIUM
- [ ] `fs.Path` — path manipulation (join, parent, stem, extension)
- [ ] `fs.exists(path)`, `fs.is_dir(path)`, `fs.is_file(path)`
- [ ] `fs.mkdir(path)`, `fs.rmdir(path)`, `fs.copy(src, dst)`
- [ ] `fs.read_dir(path)` → list of entries

### P2 — Const Evaluation — MEDIUM
- [ ] Real `const fn` execution at compile time
- [ ] Const generic parameters
- [ ] Compile-time assertions

### P3 — Password Hashing — MEDIUM
- [ ] Argon2 (via FFI or runtime implementation)
- [ ] BCrypt (via FFI)
- [ ] Salt generation utilities

### P3 — JWT — MEDIUM
- [ ] JWT generation (HS256, RS256)
- [ ] JWT validation + decode
- [ ] Claims struct serialization

---

## Phase 11: Production Hardening 📅 Planned

**Goal:** Make Kyle trustworthy for production — debugging, error quality,
testing, security.

### P0 — Error Messages — CRITICAL
- [ ] Structured error codes (E0001, E0002, ...)
- [ ] Exact span + caret pointing to the error
- [ ] Suggestions ("did you mean...?")
- [ ] Color output (terminal-aware)
- [ ] Error catalog documentation

### P0 — Debugging (DWARF) — CRITICAL
- [ ] Emit DWARF debug info in LLVM codegen
- [ ] `gdb` / `lldb` breakpoint support
- [ ] Variable inspection in debugger
- [ ] `--debug` flag (default) vs `--release` (strips debug info)

### P1 — Testing Framework — HIGH
- [ ] `kl test` compiles and runs tests (currently only type-checks)
- [ ] Test runner with assertions + output
- [ ] 100+ integration tests (examples verified via CI)
- [ ] Fuzz testing for lexer + parser
- [ ] Standard library test suite
- [ ] Runtime tests (RAII, async, channels)

### P1 — Query Builder / Lightweight ORM — HIGH
- [ ] Type-safe SQL query builder
- [ ] CRUD operations
- [ ] Basic migrations
- [ ] Schema inference from structs

### P2 — TLS/SSL — MEDIUM
- [ ] TLS for HTTP server (HTTPS)
- [ ] TLS for HTTP client
- [ ] Certificate management

### P2 — WebAssembly Compilation — MEDIUM
- [ ] `kl build --target wasm` → .wasm output
- [ ] WASI support (file system, network)
- [ ] Browser interop (JS FFI)

---

## Phase 12: Self-Hosting ⏸️ Deferred

Rewrite the Kyle compiler in Kyle itself. Deferred until the language is stable
and the std library is rich enough.

- [x] Lexer in Kyle (examples/lexer.kl) ✅
- [x] Parser in Kyle (examples/parser.kl) ✅
- [x] Semantic analyzer in Kyle (examples/semantic.kl) ✅
- [ ] MIR lowering in Kyle
- [ ] LLVM codegen in Kyle
- [ ] Self-compilation test (`kl build compiler.kl` produces working `kl`)

---

## Phase 13: Ecosystem 📅 Future

- [ ] Package registry (`kl publish`, `kl install`, `kl search`)
- [ ] Official backend web framework
- [ ] Frontend framework (via WASM)
- [ ] Community templates and examples
- [ ] Documentation website (kl-lang.org)
- [ ] Package signing + verification

---

## Implementation Status Matrix

### Language Features (End-to-End)

| Feature | Status | Notes |
|---------|--------|-------|
| Variables (let/mut/const) | ✅ | Immutable by default |
| Functions | ✅ | With generics, contracts, error types |
| If/elif/else | ✅ | |
| While / While-else | ✅ | |
| Loop | ✅ | Infinite loop with break |
| For (list) / For (range) / For-else | ✅ | |
| Match (stmt + expr) | ✅ | With patterns + guards |
| Return / Break | ✅ | |
| Defer | ✅ | LIFO lowering |
| Guard | ✅ | Early return via CondBr |
| Unsafe | ✅ | Parsed (FFI lowering pending Phase 9) |
| Struct | ✅ | With generics, pass-by-reference |
| Enum | ✅ | With payloads, tagged union |
| Class | ✅ | With constructors, methods, inheritance |
| Method dispatch | ✅ | `this` param dedup |
| Closure | ✅ | `(x) => body`, FnAddr + CallIndirect |
| Async/await | ✅ | Thread-based (kl_spawn_thread/kl_join_thread) |
| Generics (structs + functions) | ✅ | Monomorphization |
| Error types (! / ?) | ✅ | Option-based lowering |
| Optional chaining (?.) | ✅ | |
| Contracts | ✅ | Validation + impl (no codegen constraints) |
| Type aliases | ✅ | Chained resolution |
| String interpolation | ✅ | Via concat + str() |
| Dict/Map literals | ✅ | Indexing + .len() |
| Spread operator (...) | ✅ | List literals |
| Range slicing (list[a..b]) | ✅ | |
| Ternary (? :) | ✅ | |
| const fn | ✅ | Type-checker validation (no compile-time eval yet) |

### Not Yet Implemented

| Feature | Phase | Priority |
|---------|-------|----------|
| FFI (extern "C" + unsafe lowering) | 9 | P0 Critical |
| HTTP Server + Client | 9 | P1 Critical |
| Database Drivers (SQLite, Postgres) | 9 | P2 Critical |
| ENV variables | 9 | P2 Essential |
| Process spawning | 9 | P2 Essential |
| JSON auto-serialization | 9 | P3 High |
| Release optimization (O2/O3) | 9 | P3 High |
| Iterators (.iter()) | 10 | P0 Critical |
| Functional ops (map/filter/reduce) | 10 | P0 Critical |
| HashMap / HashSet / Queue / Deque | 10 | P1 High |
| Logging | 10 | P1 High |
| Config files + env | 10 | P1 High |
| Contracts with generic constraints | 10 | P2 Medium |
| Filesystem utilities | 10 | P2 Medium |
| Const evaluation (real compile-time) | 10 | P2 Medium |
| Password hashing (Argon2/BCrypt) | 10 | P3 Medium |
| JWT | 10 | P3 Medium |
| Structured error messages | 11 | P0 Critical |
| DWARF debug info | 11 | P0 Critical |
| Test runner (compile + execute) | 11 | P1 High |
| Query Builder / ORM | 11 | P1 High |
| TLS/SSL | 11 | P2 Medium |
| WebAssembly target | 11 | P2 Medium |
| Package registry | 13 | Future |
| Official backend framework | 13 | Future |

### Cross-Platform Support

| Platform | Status |
|----------|--------|
| macOS Apple Silicon (aarch64) | ✅ |
| Linux ARM (aarch64, Docker) | ✅ |
| macOS Intel (x86_64) | ❌ |
| Linux x86_64 | ❌ |
| Windows x86_64 | ❌ |

### Distribution

| Feature | Status |
|---------|--------|
| `kl` binary (primary) | ✅ |
| `klc` binary (legacy) | ✅ |
| install.sh (curl \| sh) | ✅ |
| GitHub Actions CI | ✅ |
| VS Code .vsix | ✅ (kl-0.2.0.vsix) |
| VS Code Marketplace | ❌ |
| Homebrew tap | ❌ |
| kl-lang.org website | ❌ |
| Linux x64 binaries | ❌ |
| Windows binaries | ❌ |

### Testing

| Metric | Value |
|--------|-------|
| Unit tests | 101 (0 failures) |
| Integration tests (examples) | 50+ (all run correctly; error_test returns 1 deliberately) |
| Standard library tests | 0 (Phase 11) |
| Fuzz tests | 0 (Phase 11) |

---

## v1.0 Release Checklist

A developer must be able to:

1. [x] Install Kyle (curl install.sh)
2. [x] Create a project (`kl new myapp`)
3. [ ] Connect to PostgreSQL (Phase 9 P2)
4. [ ] Create a REST API (Phase 9 P1)
5. [ ] Serialize JSON automatically (Phase 9 P3)
6. [ ] Authenticate users with JWT (Phase 10 P3)
7. [x] Run tests (`kl test`)
8. [ ] Deploy to production (Phase 7 + 11)

**Current state:** Phases 0-6 complete, Phase 7-8 nearly done. Phase 9 (Backend
& Systems) is the critical next milestone — it unlocks real-world usage.

---

## Key Design Decisions (frozen)

| Decision | Choice |
|----------|--------|
| Blocks | Indentation (4 spaces) |
| Semicolons | None — newline terminates statements |
| Variables | Immutable by default, `mut` for mutable |
| Constants | UPPERCASE, compile-time, no `mut` |
| Instance reference | `this` (not `self`) |
| Optionals | `Option<T>` (not `T?`) |
| Error propagation | `?` (for errors only) |
| Abstract | `abs class` / `abs fn` |
| Visibility | Convention (`_` protected, `__` private) |
| Exceptions | None — explicit errors with `!` and `match` |
| `let`/`var` | None — `mut` keyword directly |
| `{}` for blocks | None — indentation |
| Export | None — visibility by naming |
| String encoding | UTF-8 |
| Integer overflow | Panic in debug, wrapping in release |
| Entry point | `fn main(args: [str]) -> i32` in `src/main.kl` |
| Memory | RAII + Compiler-Inferred Ownership (no GC) |

---

## Version

```
Roadmap & Status v1.0
Last updated: 2026-06-26
Tests: 101 unit tests, 0 failures
Examples: 50+ examples, all run correctly
Phase 6: COMPLETE — all language features generate working code
Phase 7-8: CURRENT — cross-platform + distribution polish
Phase 9: NEXT — backend & systems (FFI, HTTP, DB) is the critical milestone
```