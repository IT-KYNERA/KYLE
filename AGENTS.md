# Kyle — AI Agent Context

> **Read this first.** It is the single entry-point for AI agents working on the Kyle codebase.
> It tells you what Kyle is, where we are, how to test, and **where to find documentation**.

---

## What is Kyle?

A compiled, statically-typed language for backend systems, CLI tools, and full-stack development.
Written in **Rust** (compiler + runtime), compiles via **LLVM 18**.

- Python readability (indentation blocks, no semicolons, no `self`)
- Rust type safety (strong typing, generics, pattern matching, borrow checker)
- Go simplicity (fast compilation, built-in tooling, package manager)
- C performance (native code via LLVM O3 pipeline)

**The compiler and runtime are written in Rust.** Packages (`http`, `json`, `sqlite`) are written in **100% Kyle** using `extern fn` + `@link` for FFI to C libraries.

---

## Current Status

| Area | Status |
|------|--------|
| **Compiler (Fases 1-17)** | ✅ **Complete** — Lexer, parser, semantic, MIR, SSA, LLVM codegen, O3 pipeline |
| **Syntax** | ✅ **Complete** — Generics, ranges, match, op overloading, is, ptr, for-else, static fn, ** |
| **Borrow checker** | ✅ **Complete** — `&T` mutable, `^T` move, field mutability |
| **Tooling** | ✅ **Complete** — LSP, VS Code ext, formatter, test framework, package manager |
| **FFI (extern fn, @link, ptr)** | ✅ **Phase 0 done** — Pure Kyle FFI to C libraries |
| **Runtime in Kyle** | 🔶 **Phase A in progress** — 18/88 functions rewritten in pure Kyle |
| **kyc_platform** | 🔶 **Phase 1 started** — FS (file I/O), Time in Rust crate |

See [ROADMAP.md](ROADMAP.md) for full implementation plan.

---

## CRITICAL — When Writing Kyle Code

**When you get a syntax error or unexpected behavior:**
1. **STOP trying random syntax**
2. **Check the docs** (see Documentation Map below)
3. The docs are the **canonical source of truth** for all syntax

**Key files to consult:**
- `docs/03-language-reference/` — **Read this for ANY syntax question** (15 focused files)
- `docs/06-reference/` — Quick lookup: keywords, operators, flags, CLI commands
- `docs/04-platform/standard-library/` — Available built-in functions

**This file (AGENTS.md) does NOT contain syntax reference.**
Do not guess Kyle syntax — always check the docs.

---

## Project Structure

```
ky/
├── crates/               # Rust crates (compiler + runtime + tools)
│   ├── kyc_core/         # Foundation: AST types, diagnostics
│   ├── kyc_frontend/     # Lexer + parser
│   ├── kyc_hir/          # HIR desugaring
│   ├── kyc_semantic/     # Type checker, scope resolver, borrow analysis
│   ├── kyc_mir/          # MIR lowering, SSA construction, optimizations
│   ├── kyc_backend/      # LLVM codegen (via inkwell), linker
│   ├── kyc_driver/       # Compilation pipeline orchestration
│   ├── kyc_cli/          # CLI binary (`ky`)
│   ├── kyc_runtime/      # Runtime static library (memory, strings, lists, dicts, I/O, threads)
│   ├── kyc_tools/        # LSP server, formatter, package manager
│   └── kyc_platform/     # 🔜 Platform API: FS, networking, time (in progress)
│
├── packages/             # Official Kyle packages (100% Kyle, src/ subdir)
│   ├── http/             # HTTP client + server + router + websocket
│   ├── json/             # JSON parse + stringify
│   └── sqlite/           # SQLite database bindings
│
├── docs/                 # Documentation (72 files, reorganized)
├── vscode-ky/            # VS Code extension
├── examples/             # Example .ky project
├── tests/                # End-to-end type-check test files
└── ROADMAP.md            # Feature roadmap with phases and implementation order
```

---

## Documentation Map

| Section | Files | Content |
|---------|:-----:|---------|
| [01-overview/](docs/01-overview/README.md) | 5 | Vision, philosophy, principles, layered architecture |
| [02-guide/](docs/02-guide/README.md) | 7 | Tutorial: install, first program, testing, debugging, patterns, performance, CI/CD |
| [03-language-reference/](docs/03-language-reference/README.md) | **15** | **Formal language specification** (read for ANY syntax question) |
| [04-platform/](docs/04-platform/README.md) | 17 | Compiler CLI, build system, standard library (8 modules), tools, targets (WASM) |
| [05-packages/](docs/05-packages/README.md) | 4 | Official package specs: HTTP, JSON, SQLite, PostgreSQL |
| [06-reference/](docs/06-reference/README.md) | 4 | Quick lookup: keywords, operators, flags, CLI commands |
| [07-engineering/](docs/07-engineering/README.md) | 5 | Compiler architecture, SSA, optimization pipeline, codegen |
| [08-design/](docs/08-design/README.md) | 3 | ADRs, RFCs (architecture decisions, move semantics) |
| [09-project/](docs/09-project/README.md) | 1 | Changelog |
| [10-history/](docs/10-history/README.md) | 1 | Migration guide |

### Quick reference links

| You need... | Go to |
|-------------|-------|
| **ANY syntax question** | `docs/03-language-reference/` (15 focused files) |
| Quick keyword/operator lookup | `docs/06-reference/` |
| Compiler CLI flags | `docs/06-reference/cli-commands.md` + `docs/06-reference/compiler-flags.md` |
| How to test | `docs/02-guide/testing.md` |
| Standard library functions | `docs/04-platform/standard-library/overview.md` |
| Package manager usage | `docs/05-packages/registry.md` |
| VS Code extension | `docs/04-platform/tools/vscode.md` |
| Performance tips | `docs/02-guide/performance.md` |
| Common patterns | `docs/02-guide/patterns.md` |
| FFI (extern fn, @link, ptr) | `docs/03-language-reference/ffi.md` |

---

## Packages (100% Kyle, no Rust)

| Package | Description | Location |
|---------|-------------|----------|
| `http` | HTTP client via libcurl FFI | `packages/http/` |
| `json` | JSON parse + stringify | `packages/json/` |
| `sqlite` | SQLite database bindings | `packages/sqlite/` |

All packages use `extern fn` + `@link` for FFI. See `docs/03-language-reference/ffi.md`.

---

## Module resolution

El compilador busca módulos en este orden:
1. Relativo al archivo fuente (`./`)
2. `src/` del proyecto raíz
3. `packages/` del proyecto raíz (desarrollo local)
4. `std/` del proyecto raíz (packages instalados vía `ky add`)
5. Caché de packages (`~/.ky/cache/`)

Esto significa que `from http.server import Router` resuelve a:
- `packages/http/src/server.ky` (desarrollo)
- `std/http/server.ky` (instalado)

## Testing

```bash
# Rust unit tests (all crates)
cargo test --workspace

# Build (debug)
cargo build --workspace

# Build release
cargo build --release --bin ky

# Kyle checks (sin fn main — implicit main auto-generado)
ky check <file.ky>       # Type-check only
ky build <file.ky>        # Compile to binary
ky run <file.ky>          # Compile and run

# Package tests
cd packages/<name> && ky check src/lib.ky
```

---

## Development Commands

```bash
ky build <file.ky>        # Compile to binary (auto-genera main si falta)
ky run <file.ky>          # Compile and run (sin fn main necesario)
ky check <file.ky>        # Type-check only (fast)
ky fmt [file/dir]         # Format source
ky test                   # Run test suite
ky new <project>          # Create new project
ky add <dep>[@<ver>]      # Add dependency (GitHub Pages registry)
ky remove <dep>           # Remove dependency
ky install                # Install all dependencies from ky.lock
ky publish                # Publish package (creates tarball in registry/)
ky lsp                    # Start LSP server (for editors)
```

---

## LLVM Configuration

LLVM 18.1 required.

**macOS (Apple Silicon):** `brew install llvm@18 && export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)`
**Linux (Ubuntu ARM):** `sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev`

---

## How to publish a new release

### 1. Pre-flight checklist

Before releasing, verify:
- [ ] `cargo test --workspace` — all tests pass
- [ ] `ky check benchmarks/primes/primes.ky` — no errors
- [ ] `ky run examples/hello.ky` — works correctly
- [ ] GitHub Actions CI is green (build + test)

### 2. Update versions

Update the version in ALL of these files (search for the old version string):

| File | Field | Example |
|------|-------|---------|
| `Cargo.toml` | `version = "0.X.X"` | `version = "0.5.2"` |
| `AGENTS.md` | `Version: v0.X.X` (line 232) | `Version: v0.5.3` |
| `install.sh` | `VERSION="v0.X.X"` | `VERSION="v0.5.2"` |
| `vscode-ky/install-extension.sh` | `TAG="v0.X.X"` | `TAG="v0.5.2"` |
| `vscode-ky/package.json` | `"version": "0.X.X"` | `"version": "0.5.2"` |

### 3. Build release binary

```bash
# Full build (all crates, includes runtime)
cargo build --release --bin ky

# Verify version
./target/release/ky --version    # must show v0.X.X
```

### 4. Rebuild package tarballs (if packages changed)

If you modified any package source (`packages/<name>/src/`), rebuild its tarball:

```bash
for pkg in http json sqlite; do
    if [ -d "packages/$pkg" ]; then
        cd packages/$pkg
        tar czf ../../docs/packages/$pkg/0.1.0/download.tar.gz ky.toml src/
        cd ../..
    fi
done
```

Verify the tarball includes all necessary files:
```bash
tar tzf docs/packages/http/0.1.0/download.tar.gz
# Should show: ky.toml  src/  src/lib.ky  src/server.ky  src/websocket.ky
```

### 5. Rebuild VS Code extension

```bash
cd vscode-ky
npx @vscode/vsce package --out ky-0.X.X.vsix
cd ..
```

### 6. Commit and push

```bash
git add -A
git commit -m "Release v0.X.X: description of changes"
git push origin main
```

### 7. Create GitHub Release

```bash
# Compress the binary (MUST be named ky.gz — install.sh downloads this exact name)
gzip -c target/release/ky > /tmp/ky.gz

# Create the release
gh release create v0.X.X \
  --title "Kyle v0.X.X" \
  --notes "## Changes

- Bullet list of changes
" \
  "/tmp/ky.gz" \
  "vscode-ky/ky-0.X.X.vsix"
```

**Important:** The binary MUST be uploaded as `ky.gz` (the filename on disk, NOT with `#label`). The install script downloads `https://.../releases/download/v0.X.X/ky.gz`. If you use `#label.gz`, GitHub renames the file and the download will 404.

### 8. Push the tag

```bash
git fetch --tags origin
git tag v0.X.X
git push origin v0.X.X
```

### 9. Verify the release

```bash
# Simulate a clean install
cd /tmp && rm -rf verify_release
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh | sh

# Add to PATH and test
export PATH="$HOME/.ky/bin:$PATH"

# Version check
ky --version                     # → must show v0.X.X

# App project
ky new app /tmp/verify_release_app
cd /tmp/verify_release_app
ky check
ky build

# API project (tests package registry)
ky new api /tmp/verify_release_api
cd /tmp/verify_release_api
ky check                         # → must resolve deps, no errors
ky build

# Bare script
ky new bare /tmp/verify_release_bare
cd /tmp/verify_release_bare
ky run *.ky                      # → must print "Hello from ..."

# Cleanup
rm -rf /tmp/verify_release_*
```

### 10. If something fails

- **Download 404**: The asset filename doesn't match install.sh. Upload again with the exact filename.
- **Wrong version shown**: Binary wasn't rebuilt after Cargo.toml update. Run `cargo clean -p kyc_cli && cargo build --release --bin ky`.
- **Package not found**: Tarball wasn't rebuilt or Pages is stale. Rebuild tarball and push again.
- **Tests fail locally**: Fix tests, recommit, rebuild.

---

## What NOT to Do

1. **Do not guess syntax** — check `docs/03-language-reference/` first
2. **Do not add new syntax features** without checking the docs
3. **Do not reintroduce `mut`, `let`, `var`, `const`** — use `&T` or `:=`
4. **Do not reintroduce `Option<T>` as public syntax** — use `T?`
5. **Do not use `struct`** — use `final class`
6. **Do not write C/C++ code** — the compiler and runtime are pure Rust
7. **Do not skip tests** — CI must pass before any merge

---

*Version: v0.5.3 · Last updated: 2026-07-06 — Ver `AGENTS.md` > "How to publish a new release" para proceso completo de release.*

---

## Syntax Status por Documento

Testeado con `ky run` en `/tmp/syntax_tests/`. Cada documento de `docs/03-language-reference/` fue verificado.

| # | Documento | Status | Notas |
|---|-----------|--------|-------|
| 1 | [lexical-structure.md](docs/03-language-reference/lexical-structure.md) | [x] | Keywords, literales, comentarios `#`, escapes `\n \t \r \0`. `\u{XXXX}` unicode no expandido. No hay `/* */` |
| 2 | [variables.md](docs/03-language-reference/variables.md) | [x] | `:=` const, `&T` mutable, `^T` move |
| 3 | [types.md](docs/03-language-reference/types.md) | [ ] | `ptr = 0 as ptr` type mismatch (bug #2). `char = 'a'` mismatch. `{str: i64}` con literal `1` inferido como i32 |
| 4 | [expressions.md](docs/03-language-reference/expressions.md) | [x] | Aritmética, comparaciones, bitwise, `as` casts, rangos `..` |
| 5 | [statements.md](docs/03-language-reference/statements.md) | [x] | if/elif/else, while, for-in range, match, return |
| 6 | [functions.md](docs/03-language-reference/functions.md) | [ ] | Default params sí. `static fn` syntax (expected LParen before Static). `Calc.double()` undefined symbol |
| 7 | [classes.md](docs/03-language-reference/classes.md) | [x] | `final class`, StructLiteral `Point {x:1}`, métodos, `Class :: Parent` herencia |
| 8 | [enums.md](docs/03-language-reference/enums.md) | [x] | Enum con variants, match con `Enum.Variant` |
| 9 | [generics.md](docs/03-language-reference/generics.md) | [x] | `class Box<T>`, `fn identity<T>`, `Box<T> {value: 7}`, `identity<i32>(42)` |
| 10 | [ownership.md](docs/03-language-reference/ownership.md) | [x] | `^T` move semantics, `&T` mutable ref, default borrow-by-default |
| 11 | [pattern-matching.md](docs/03-language-reference/pattern-matching.md) | [ ] | `..=` range pattern no existe. `1 \| 2` or-pattern no funciona. Match básico con `_:` sí |
| 12 | [error-handling.md](docs/03-language-reference/error-handling.md) | [ ] | `-> Type` syntax no existe. `ok(v)`/`error(e)` result match no funciona. `return -1` sí |
| 13 | [modules.md](docs/03-language-reference/modules.md) | [x] | `from X import Y` funciona (con packages instalados). `import X` funciona |
| 14 | [ffi.md](docs/03-language-reference/ffi.md) | [x] | `@link`, `extern fn` funcionan |
| 15 | [concurrency.md](docs/03-language-reference/concurrency.md) | [x] | `async fn` con 1 param, `async:` block, `await`, `ky_parallel_for`, `ky_spawn_thread` |

### Bugs activos (de sintaxis documentada que no funciona)

| Bug | Docs ref | Síntoma | Location |
|-----|----------|---------|----------|
| `static fn` syntax error | functions.md | "expected LParen, found Static" | parser.rs |
| `Calc.double()` not found | functions.md | "undefined symbol 'double'" | lower.rs o scope.rs |
| `char = 'a'` type mismatch | types.md | "expected 'char', found 'i32'" | type_checker.rs |
| `..=` range pattern | pattern-matching.md | "expected Colon, found DotDotEquals" | parser.rs |
| `1 \| 2` or-pattern | pattern-matching.md | No implementado | parser.rs |
| `-> Type` return syntax | error-handling.md | "expected type name, found Arrow" | parser.rs |
| `ok(v)`/`error(e)` result | error-handling.md | "expected pattern, found OkKw" | parser.rs |
| `T?` optional type | types.md | Type mismatch 'str' expects 1 arg, got 2 | type_checker.rs |
| `char` type literal `'a'` | types.md | expected 'char', found 'i32' | type_checker.rs |
| `none` in docs | lexical-structure.md | El keyword es `None` (capital N), no `none` | docs.md |

## Documentation Map

Toda la documentación del lenguaje está en [`docs/`](docs/README.md):

| Para saber... | Ir a... |
|---------------|---------|
| Sintaxis del lenguaje (completa) | `docs/03-language-reference/` (15 archivos) |
| Referencia rápida (keywords, operadores) | `docs/06-reference/` |
| Documentación del compilador (flags, CLI) | `docs/04-platform/` |
| Standard library (builtins) | `docs/04-platform/standard-library/` |
| Paquetes oficiales (http, json, sqlite) | `docs/05-packages/` |
| Benchmarks y rendimiento | `benchmarks/` + `ROADMAP.md` |
| Arquitectura del compilador | `docs/07-engineering/`
