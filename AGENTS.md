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
| **Borrow checker** | ✅ **Complete** |
| **Tooling** | ✅ **Complete** — LSP, VS Code ext, formatter, test framework, package manager |
| **FFI (extern fn, @link, ptr)** | ✅ **Phase 0 done** — Pure Kyle FFI to C libraries |
| **Runtime in Kyle** | ✅ **Complete** |
| **kyc_platform** | ✅ **Complete** |

See [ROADMAP.md](ROADMAP.md) for full implementation plan.

---

## CRITICAL — When Writing Kyle Code

**When you get a syntax error or unexpected behavior:**
1. **STOP trying random syntax**
2. **Check the docs** (see Documentation Map below)
3. The docs are the **canonical source of truth** for all syntax

**Key files to consult:**
- `docs/03-language/` — **Read this for ANY syntax question** (8 subdirectories)
- `docs/03-language/lexical/operators.md` — Quick lookup: keywords, operators
- `docs/04-standard-library/` — Standard library API
- `TEST_CHECKLIST.md` — Verified syntax features with test status

**Critical naming conventions:**
- **snake_case** for everything: functions, types, methods, variables
- **`^T`** = mutable type, `&T` = borrow, `^&T` = mutable borrow
- **Move by default**: `y = x` transfers ownership for non-Copy types
- **No `let`, `var`, `const`, `mut`**: use `name = value` or `name: ^T = value`
- **No `pub`**: use `_name` for protected, `__name` for private
- **No `self`**: use `this.field` for field access
- **Generic params**: uppercase `T` (only exception to snake_case)

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
├── docs/                 # Documentation (158 files, reorganized)
├── vscode-ky/            # VS Code extension
├── examples/             # Example .ky project
├── tests/                # End-to-end type-check test files
└── ROADMAP.md            # Feature roadmap with phases and implementation order
```

---

## Documentation Map

| Section | Files | Content |
|---------|:-----:|---------|
| Section | Files | Content |
|---------|:-----:|---------|
| [01-introduction/](docs/01-introduction/README.md) | 6 | Vision, philosophy, principles, architecture, roadmap, FAQ |
| [02-getting-started/](docs/02-getting-started/README.md) | 9 | Installation, first program, build, testing, debugging, IDE |
| [03-language/](docs/03-language/README.md) | **8 subdirectories** | **Formal language specification** (read for ANY syntax question) |
| [04-standard-library/](docs/04-standard-library/README.md) | 22 | Standard library modules |
| [05-runtime/](docs/05-runtime/README.md) | 6 | Runtime internals (memory, scheduler, panic, startup) |
| [06-compiler/](docs/06-compiler/README.md) | 17 | Compiler pipeline (lexer, parser, MIR, codegen, linker) |
| [07-tools/](docs/07-tools/README.md) | 11 | CLI, formatter, LSP, VSCode, build system, distribution |
| [08-ecosystem/](docs/08-ecosystem/README.md) | 9 | Registry, packages (http, json, sqlite) |
| [09-specification/](docs/09-specification/README.md) | 7 | Grammar, precedence, type system references |
| [10-design/](docs/10-design/README.md) | 2 | ADRs, RFCs (architecture decisions, move semantics) |
| [11-project/](docs/11-project/README.md) | 1 | CI/CD workflows |
| [12-history/](docs/12-history/README.md) | 3 | Changelog, migration guides, deprecated |

### Quick reference links

| You need... | Go to |
|-------------|-------|
| **ANY syntax question** | `docs/03-language/` (8 subdirectories) |
| Quick keyword/operator lookup | `docs/03-language/lexical/` |
| Compiler CLI flags | `docs/07-tools/compiler-cli.md` |
| How to test | `docs/02-getting-started/testing.md` |
| Standard library | `docs/04-standard-library/` |
| Package manager | `docs/02-getting-started/package-manager.md` |
| VS Code extension | `docs/07-tools/vscode.md` |
| Performance tips | `docs/02-getting-started/performance.md` |
| FFI (extern fn, @link, ptr) | `docs/03-language/ffi/abi.md` |
| UI framework (.kyx) | `docs/03-language/syntax/ui-syntax.md` |
| UI roadmap + WASM | `docs/10-design/rfc/0002-ui-architecture.md` |
| Multi-platform install | `docs/07-tools/distribution.md` |
| Runtime internals | `docs/05-runtime/` |
| Compiler pipeline | `docs/06-compiler/` |

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

Esto significa que `from http.server import router` resuelve a:
- `packages/http/src/server.ky` (desarrollo)
- `std/http/server.ky` (instalado)

## Testing

### Rust unit tests (all platforms)

```bash
# Run all tests (157+ across 9 crates)
cargo test --workspace

# Run tests for a specific crate
cargo test -p kyc_semantic
cargo test -p kyc_frontend

# Build (debug)
cargo build --workspace

# Build release
cargo build --release --bin ky

# Cross-compile runtime only (no LLVM needed)
cargo build --target x86_64-pc-windows-gnu -p kyc_runtime --release
cargo build --target aarch64-unknown-linux-gnu -p kyc_runtime --release
```

**Note:** The `ky` binary (compiler) links against LLVM static libraries which are architecture-specific. Only the `kyc_runtime` crate can be freely cross-compiled. CI uses native runners per platform.

### Kyle checks (no `fn main` needed — auto-generated)

```bash
ky check <file.ky>       # Type-check only
ky build <file.ky>        # Compile to binary
ky run <file.ky>          # Compile and run

# Package tests
cd packages/<name> && ky check src/lib.ky
```

### Cross-platform notes

| Platform | Test command | Notes |
|----------|-------------|-------|
| **macOS** | `cargo test --workspace` | Native. LLVM via Homebrew. |
| **Linux** | `cargo test --workspace` | Native. LLVM via apt. |
| **Windows (MSVC)** | `cargo test --workspace` | Requires VS Build Tools for `link.exe`. |
| **Windows (GNU)** | `cargo test --workspace` | Requires MinGW-w64. Use target `x86_64-pc-windows-gnu`. |

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

LLVM 18.1 required across all platforms.

| Platform | Install command | Env var |
|----------|----------------|---------|
| **macOS** | `brew install llvm@18` | `LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)` |
| **Linux (Debian/Ubuntu)** | `sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev` | `LLVM_SYS_181_PREFIX=/usr/lib/llvm-18` |
| **Windows (Chocolatey)** | `choco install llvm --version=18.1.8` | `LLVM_SYS_181_PREFIX=C:\Program Files\LLVM` |
| **Windows (portable)** | Download `LLVM-18.1.8-win64.zip` + extract | `LLVM_SYS_181_PREFIX=C:\path\to\LLVM-18.1.8-win64` |

**Note:** `LLVM_SYS_181_PREFIX` tells `inkwell` (Rust LLVM bindings) where to find LLVM 18 libraries. Without this env var, the build will fail.

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

| file | Field | Example |
|------|-------|---------|
| `Cargo.toml` | `version = "0.X.X"` | `version = "0.5.2"` |
| `AGENTS.md` | `Version: v0.X.X` (line 232) | `Version: v0.6.0` |
| `install.sh` | `VERSION="v0.X.X"` | `VERSION="v0.5.2"` |
| `install.ps1` | `$Version = "v0.X.X"` | `$Version = "v0.6.0"` |
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

### 7. Create GitHub Release + upload assets

The release is created automatically by CI when a tag is pushed, OR you can create it manually:

```bash
# Create release (assets uploaded by CI)
gh release create v0.X.X \
  --title "Kyle v0.X.X" \
  --notes "## Changes

- Bullet list of changes
"
```

**The CI workflow (`.github/workflows/release.yml`) builds and uploads 5 bundles automatically:**

| Bundle | Platform | CI Runner |
|--------|----------|-----------|
| `ky-macos-arm64.tar.gz` | macOS Apple Silicon | `macos-latest` |
| `ky-macos-x64.tar.gz` | macOS Intel | `macos-13` |
| `ky-linux-arm64.tar.gz` | Linux ARM64 | `ubuntu-24.04-arm` |
| `ky-linux-x64.tar.gz` | Linux x86_64 | `ubuntu-24.04` |
| `ky-windows-x64.zip` | Windows x86_64 | `windows-2025` |

Each bundle contains (flat structure, no top-level dir):
```
ky (or ky.exe)
libkyc_runtime.a
LICENSE
```

### 8. Push the tag (triggers CI)

```bash
git fetch --tags origin
git tag v0.X.X
git push origin v0.X.X
```

This triggers `.github/workflows/release.yml` which:
1. Creates the release in GitHub
2. Compiles `ky` + `kyc_runtime` for all 5 platforms in parallel
3. Generates flat bundles + SHA-256 checksums
4. Uploads assets to the release

**No local build needed.** CI handles all cross-compilation via native runners.

### 9. Verify the release

```bash
# Simulate a clean install (macOS / Linux)
cd /tmp && rm -rf verify_release
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh | sh

# Windows (PowerShell)
iwr -Uri "https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.ps1" | iex
```

Test on each platform:

```bash
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

| Problem | Cause | Fix |
|---------|-------|-----|
| **Download 404** | Asset not uploaded or wrong name | Check CI logs; re-run failed CI jobs |
| **Wrong version** | Binary not rebuilt after Cargo.toml update | `cargo clean -p kyc_cli && cargo build --release --bin ky` |
| **Package not found** | Tarball not rebuilt or GitHub Pages stale | Rebuild tarball, push again |
| **Tests fail** | Code regression | Fix tests, recommit, rebuild |
| **Windows CI fails** | LLVM 18 installation | Check `windows-2025` runner; update LLVM download URL in `release.yml` |

---

## What NOT to Do

1. **Do not guess syntax** — check `docs/03-language-reference/` first
2. **Do not add new syntax features** without checking the docs
3. **Do not reintroduce `mut`, `let`, `var`, `const`** — use `^T` or `:=`
4. **Do not reintroduce `Option<T>` as public syntax** — use `T?`
5. **Do not use `struct`** — use `final class`
6. **Do not write C/C++ code** — the compiler and runtime are pure Rust
7. **Do not skip tests** — CI must pass before any merge

---

## Cross-Platform Development Guide

### Setting up a development environment

#### macOS (Apple Silicon / Intel)

```bash
# 1. Install LLVM 18
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Clone and build
git clone https://github.com/IT-KYNERA/KYLE.git
cd KYLE
cargo build --release --bin ky
cargo build --release -p kyc_runtime

# 4. Test
cargo test --workspace
ky run examples/hello.ky
```

#### Linux (Ubuntu ARM64 / x86_64)

```bash
# 1. Install LLVM 18
sudo apt update && sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev
export LLVM_SYS_181_PREFIX=/usr/lib/llvm-18

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Clone and build
git clone https://github.com/IT-KYNERA/KYLE.git
cd KYLE
cargo build --release --bin ky
cargo build --release -p kyc_runtime

# 4. Test
cargo test --workspace
ky run examples/hello.ky
```

#### Windows (x86_64)

```powershell
# 1. Install LLVM 18 (Option A — Chocolatey, run PowerShell as Admin)
choco install llvm --version=18.1.8
$env:LLVM_SYS_181_PREFIX = "C:\Program Files\LLVM"

# Or Option B — portable zip (no admin needed)
Invoke-WebRequest -Uri "https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/LLVM-18.1.8-win64.zip" -OutFile "$env:TEMP\llvm-18.zip"
Expand-Archive -Path "$env:TEMP\llvm-18.zip" -DestinationPath "$env:USERPROFILE\llvm-18"
$env:LLVM_SYS_181_PREFIX = "$env:USERPROFILE\llvm-18\LLVM-18.1.8-win64"

# 2. Install Rust (https://rustup.rs)
#    Default MSVC toolchain is recommended (requires Visual Studio Build Tools)
#    For MinGW: rustup toolchain install stable-x86_64-pc-windows-gnu

# 3. Clone and build
git clone https://github.com/IT-KYNERA/KYLE.git
cd KYLE
cargo build --release --bin ky
cargo build --release -p kyc_runtime

# 4. Test
cargo test --workspace
.\target\release\ky.exe run examples\hello.ky
```

**Note:** The MSVC toolchain requires `link.exe` from Visual Studio Build Tools.
If using MinGW (`x86_64-pc-windows-gnu`), install `mingw-w64` and the linker will default to GCC.

### Building cross-platform bundles

The `ky` binary links against LLVM static libraries, which are architecture-specific.
Only `kyc_runtime` (pure Rust) can be freely cross-compiled.

| Target | `ky` binary | `kyc_runtime` |
|--------|-------------|---------------|
| Same as host | ✅ Native build | ✅ Native build |
| Different architecture | ❌ Needs LLVM for target | ✅ Cross-compile with `cargo-zigbuild` |
| Different OS | ❌ Needs CI runner | ✅ Cross-compile with `cargo-zigbuild` |

**Recommended workflow:**
1. Push tag → CI builds all 5 platforms natively (`.github/workflows/release.yml`)
2. OR build locally for host platform, use CI for the rest

### Known issues per platform

| Platform | Issue | Status |
|----------|-------|--------|
| **macOS** | None | ✅ Fully supported |
| **Linux** | None | ✅ Fully supported |
| **Windows** | Socket ops use `std::net` handle table (not raw `libc` fd) | ✅ Implemented |
| **Windows** | VS Build Tools required for `link.exe` (MSVC toolchain) | ⚠️ Documented |
| **Windows** | WebSocket over Windows sockets not tested on real Windows | 🔜 Needs testing |
| **Windows** | CI runner `windows-2025` has LLVM 20 pre-installed; workflow installs LLVM 18 separately | ⚠️ Works with zip extraction |

### Distribution model

Each release publishes 5 platform-specific bundles (flat archives with `ky` + `libkyc_runtime.a` + `LICENSE`):

| Platform | Install command |
|----------|----------------|
| macOS / Linux | `curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh \| sh` |
| Windows | `iwr -Uri "https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.ps1" \| iex` |

Both scripts detect the platform, download the correct bundle, verify SHA-256, install, and configure PATH.

See `docs/07-tools/distribution.md` for full details.

---

*Version: v0.6.0 · Last updated: 2026-07-09 — See sections above for release process and cross-platform guide.*

---

## Syntax Status by Document

> All items start as `[x]` — tested and verified.
> See `TEST_CHECKLIST.md` for the complete test suite.

| # | Document | Status | Notes |
|---|----------|--------|-------|
| 1 | `03-language/lexical/literals.md` | [x] | Keywords, literals, comments `#`, escapes `\n \t \r \0` |
| 2 | `03-language/syntax/variables.md` | [x] | `:=` const, `^T` mutable (v0.6), `&T` borrow |
| 3 | `03-language/types/primitive-types.md` | [x] | All types, Copy/Move, `^T`, `&T`, `^&T` |
| 4 | `03-language/syntax/expressions.md` | [x] | Arithmetic, comparisons, bitwise, `as` casts, ranges `..` |
| 5 | `03-language/syntax/statements.md` | [x] | if/elif/else, while, for-in range, match, return |
| 6 | `03-language/syntax/functions.md` | [x] | Parameters (move/borrow/mut borrow), fn pointers, closures |
| 7 | `03-language/types/structs.md` | [x] | `class`, `final class`, StructLiteral, methods, inheritance |
| 8 | `03-language/types/enums.md` | [x] | Enum with variants, match with `Enum.Variant` |
| 9 | `03-language/types/generics.md` | [x] | `class Box<T>`, `fn identity<T>`, `identity<i32>(42)` |
| 10 | `03-language/memory/ownership.md` | [x] | `^T` = mutable, `&T` = borrow, `^&T` = mut borrow, move default |
| 11 | `03-language/syntax/pattern-matching.md` | [x] | `..=` range pattern, `1 | 2` or-pattern, basic match |
| 12 | `03-language/error-handling/result.md` | [x] | `T!`, `ok(v)`/`error(e)` patterns, result match |
| 13 | `03-language/syntax/modules.md` | [x] | `from X import Y`, `import X` |
| 14 | `03-language/ffi/abi.md` | [x] | `@link`, `extern fn` declarations |
| 15 | `03-language/concurrency/async-await.md` | [x] | `async fn`, `async:` block, `await` |


