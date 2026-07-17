# Kyle — AI Agent Context

> **Read this first.** It is the single entry-point for AI agents working on the Kyle codebase.
> It tells you what Kyle is, where we are, how to test, and **where to find documentation**.

---

## Benchmarks

```bash
cd benchmarks && bash run_benchmarks.sh
```

Corre 4 benchmarks (primes, fib, concat, matmul) en 8 lenguajes (C, C++, Rust, C#, Go, Java, Python, Kyle). El script compila todo automáticamente y muestra resultados en ms.

Para agregar un nuevo benchmark:
```bash
mkdir -p benchmarks/mibench
# Crear mibench.c, mibench.cpp, mibench.rs, mibench.go, mibench.py, mibench.java, mibench.ky
# El script detecta automáticamente los archivos existentes
```

Para rebuildear solo Kyle:
```bash
./target/release/ky build benchmarks/primes/primes.ky
cp target/debug/primes benchmarks/primes/primes_ky
```

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
| **Compiler (Fases 1-17)** | ✅ **Complete** |
| **Types (46 total)** | ✅ **Complete** |
| **Borrow checker** | ✅ **Complete** |
| **Cross-platform build** | ✅ **Complete** — macOS ARM, Linux ARM/x64, Windows x64 |
| **Tooling** | ✅ **Complete** — LSP, formatter, test framework, package manager |
| **FFI (extern fn, @link, ptr)** | ✅ **Complete** |
| **Runtime in Kyle** | ✅ **Complete** |
| **kyc_platform** (fs, time) | ✅ **Complete** |
| **u8-u64 codegen** | ✅ **Complete** |
| **HTTP / JSON / SQLite packages** | ✅ **Complete** |
| **VSCode extension** | ✅ Syntax highlighting, snippets (48 kyui), LSP, debugger, testing UI |
| **`--target freestanding`** | ✅ **Complete** | Entry point `_start`, no main wrapper. `ky build freestanding kernel.ky` |
| | |

### UI Framework (v0.8.2) — Estado por funcionalidad

| Funcionalidad | Estado | Detalle |
|--------------|:------:|---------|
| **Web backend** | ✅ **Funcional** | Genera ESM JS, HTML con CSS reset, router automático |
| **`ky new kyui` template** | ✅ **Funcional** | `app.kyx` + `src/views/` + `src/layouts/` + imports |
| **Routing** (`<router>`, `<route>`) | ✅ **Funcional** | Centralizado, rutas por path, wildcard `*` |
| **Layout persistente** (`<layout>`, `<slot>`) | ✅ **Funcional** | Se genera como función render separada |
| **Module resolver** (`from X import Y`) | ✅ **Funcional** | Resuelve imports + auto-detecta tags custom |
| **File picker** (`<file_picker>`) | ✅ **Funcional** | Web: `<input type="file">` + FileReader + `file_data` |
| **Form models** (`<form model=@form>`) | ✅ **Funcional** | `field="name"` binding automático a modelo |
| **Styles** (`style<button> Primary:`) | ✅ **Funcional** | CSS generado correctamente (`color("#...")` → `'#...'`) |
| **navigate() / set_title() / <link>** | ✅ **Funcional** | Built-in en runtime JS |
| **Document title / icon** | ✅ **Funcional** | `document.title` desde `<app title="...">` |
| **CSS reset** | ✅ **Funcional** | `* { margin:0; padding:0 }` en HTML generado |
| **Component tags** (vstack, hstack, etc.) | ✅ **Funcional** | 46 tipos nativos en ComponentTag |
| **Condicionales / loops / match** | ✅ **Funcional** | `@if`, `@for`, `@match` en templates |
| | | |
| **Desktop backend (SDL2)** | 🟡 **Roto** | Sin `SDL_PollEvent`, ventana no responsive, solo 11/46 tags |
| **iOS backend (SwiftUI)** | 🟡 **Roto** | Swift inválido (`.fontWeight(.bold())`, `Color(hex:)`), routes ignorados |
| **WASM target** | ❌ **No probado** | `kyc_runtime_wasm` excluded de tests |
| **Android backend** | ❌ **No existe** | Pendiente |
| **Terminal / TUI backend** | ❌ **No existe** | Pendiente |

See [ROADMAP.md](ROADMAP.md) for full implementation plan.

---

## CRITICAL — CI & Platform Configuration

**DO NOT modify these files unless you fully understand the cross-platform implications.**
These configurations are the result of extensive debugging across 4 platforms and are
known to work. Any change can break Windows, Linux ARM, Linux x64, or macOS ARM builds.

| File | What it does | Risk if modified |
|------|-------------|------------------|
| `.github/workflows/release.yml` | CI build + release for all 4 platforms + VSIX packaging | Breaks CI, broken releases |
| `.github/workflows/ci.yml` | CI test runner | Breaks test CI |
| `.cargo/config.toml` | Per-platform LLVM prefix (empty — CI sets it) | Can override LLVM path |
| `tools/windows/patch-llvm.ps1` | Creates missing LLVM headers + `llvm-config.exe` for Windows | Breaks Windows build |
| `tools/windows/llvm-config.cmd` | Batch fallback for llvm-config queries | Breaks Windows build |
| `crates/kyc_backend/src/linker.rs` | Platform-specific linker flags (CRT, system libs) | Breaks linking on any platform |
| `install.sh` / `install.ps1` | One-command installers | Broken user experience |
| `vscode-ky/src/extension.ts` | VS Code extension binary discovery | Extension stops working |
| `vscode-ky/install-extension.sh` / `.ps1` | Extension installers | Users can't install extension |

**If you must change these:** test on ALL 4 platforms before merging. Use `cargo check`
locally per-platform and let CI verify any changes via a tagged release.

---

## CRITICAL — When Writing Kyle Code

**When you get a syntax error or unexpected behavior:**
1. **STOP trying random syntax**
2. **Check the docs** (see Documentation Map below)
3. The docs are the **canonical source of truth** for all syntax

**Key files to consult:**
- `docs/03-language/` — **Read this for ANY syntax question** (9 subdirectories)
- `docs/03-language/lexical/operators.md` — Quick lookup: keywords, operators
- `docs/03-language/syntax/operator-overloading.md` — Overloading `op_add`, `op_eq`, etc.
- `docs/03-language/syntax/string-interpolation.md` — `"{expr}"` syntax
- `docs/03-language/syntax/error-propagation.md` — `!` operator for errors
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

**Syntax reference for porting the compiler:** see `docs/15-kyle-syntax-reference.md` (this project) or the docs below.

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
├── docs/                 # Documentation (170+ files)
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
| [03-language/](docs/03-language/README.md) | **9 subdirectories** | **Formal language specification** (read for ANY syntax question) |
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
| **ANY syntax question** | `docs/03-language/` (9 subdirectories) |
| Quick keyword/operator lookup | `docs/03-language/lexical/` |
| Compiler CLI flags | `docs/07-tools/compiler-cli.md` |
| How to test | `docs/02-getting-started/testing.md` |
| Standard library | `docs/04-standard-library/` |
| Package manager | `docs/02-getting-started/package-manager.md` |
| VS Code extension | `docs/07-tools/vscode-extension.md` — LSP handlers, grammar, bugs conocidos |
| Performance tips | `docs/02-getting-started/performance.md` |
| FFI (extern fn, @link, ptr) | `docs/03-language/ffi/abi.md` |
| Operator overloading | `docs/03-language/syntax/operator-overloading.md` |
| String interpolation | `docs/03-language/syntax/string-interpolation.md` |
| Error propagation (`!`) | `docs/03-language/syntax/error-propagation.md` |
| UI framework (.kyx + imports) | `docs/03-language/syntax/ui-syntax.md` |
| UI routing | `docs/03-language/ui/routing.md` |
| UI layout + slots | `docs/03-language/ui/composition.md` |
| File picker | `docs/03-language/ui/file-picker.md` |
| Form models | `docs/03-language/ui/state-events.md` (Section 8) |
| VSCode extension | `docs/07-tools/vscode-extension.md` — LSP handlers, grammar, bugs conocidos |
| UI roadmap + WASM | `docs/10-design/rfc/0002-ui-architecture.md` |
| Multi-platform install | `docs/07-tools/distribution.md` |
| Runtime internals | `docs/05-runtime/` |
| Compiler pipeline | `docs/06-compiler/` |
| **Syntax reference (self-hosting)** | **`docs/15-kyle-syntax-reference.md`** |
| **Self-hosting plan** | **`docs/14-self-hosting.md`** |
| **Syntax checklist (368 features)** | **`tests/SYNTAX_CHECKLIST.md`** — 181 ✅ 175 ❓ 12 ❌ |

---

## Packages (100% Kyle, no Rust)

| Package | Description | Location |
|---------|-------------|----------|
| `http` | HTTP client via libcurl FFI | `packages/http/` |
| `json` | JSON parse + stringify | `packages/json/` |
| `sqlite` | SQLite database bindings | `packages/sqlite/` |

All packages use `extern fn` + `@link` for FFI. See `docs/03-language/ffi/abi.md`.

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
| `AGENTS.md` | `Version: v0.X.X` (line 232) | `Version: v0.6.1` |
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

**The CI workflow (`.github/workflows/release.yml`) builds and uploads 4 platform bundles + VSIX automatically:**

| Bundle | Platform | CI Runner |
|--------|----------|-----------|
| `ky-macos-arm64.tar.gz` | macOS Apple Silicon | `macos-latest` |
| `ky-linux-arm64.tar.gz` | Linux ARM64 | `ubuntu-24.04-arm` |
| `ky-linux-x64.tar.gz` | Linux x86_64 | `ubuntu-24.04` |
| `ky-windows-x64.zip` | Windows x86_64 | `windows-2025` |
| `ky-extension.vsix` | VS Code extension | `ubuntu-24.04` |

> **macOS Intel (x64) is no longer supported.** Apple stopped shipping Intel Macs.
> The `macos-13` runner was removed from CI in v0.6.2.

Each bundle contains (flat structure, no top-level dir):
```
ky (or ky.exe)
libkyc_runtime.a (or kyc_runtime.lib on Windows)
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
2. Packages VS Code extension (`.vsix`)
3. Compiles `ky` + `kyc_runtime` for all 4 platforms in parallel
4. Generates flat bundles + SHA-256 checksums
5. Uploads assets to the release

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

---

## Cross-Platform Development Guide

### Setting up a development environment

#### macOS (Apple Silicon)

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

# Or Option B — download installer + extract with 7-Zip (no admin needed)
Invoke-WebRequest -Uri "https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/LLVM-18.1.8-win64.exe" -OutFile "$env:TEMP\llvm-18.exe"
& "C:\Program Files\7-Zip\7z.exe" x "$env:TEMP\llvm-18.exe" -o"$env:USERPROFILE\llvm-18" -y
$env:LLVM_SYS_181_PREFIX = "$env:USERPROFILE\llvm-18"
# Then run tools\windows\patch-llvm.ps1 to create missing headers

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
1. Push tag → CI builds all 4 platforms natively (`.github/workflows/release.yml`)
2. OR build locally for host platform, use CI for the rest

### Known issues per platform

| Platform | Issue | Status |
|----------|-------|--------|
| **macOS** | None | ✅ Fully supported |
| **Linux** | None | ✅ Fully supported |
| **Windows** | Build requires `clang+llvm` tarball (936 MB) for headers | ✅ CI handles it |
| **Windows** | VS Build Tools required for `link.exe` (MSVC toolchain) | ⚠️ Documented |
| **Windows** | CI runner `windows-2025` has LLVM 20 pre-installed; workflow installs LLVM 18 separately via tarball + 7z | ✅ Works |

### Distribution model

Each release publishes 4 platform-specific bundles + VSIX extension (flat archives with `ky` + `libkyc_runtime.a` + `LICENSE`):

| Platform | Install command |
|----------|----------------|
| macOS / Linux | `curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh \| sh` |
| Windows | `iwr -Uri "https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.ps1" \| iex` |

Both scripts detect the platform, download the correct bundle, verify SHA-256, install, and configure PATH.

See `docs/07-tools/distribution.md` for full details.

---

## Self-hosting: Rust → Kyle

**El compilador de Kyle está escrito en Rust (~50k líneas). Podemos portearlo a Kyle pieza por pieza, porque Kyle ya tiene todo lo necesario para compilarse a sí mismo.**

### ¿Qué se puede pasar a Kyle AHORA?

| Componente | Rust | Kyle | Cómo |
|-----------|:----:|:----:|------|
| **Runtime** (alloc, strings, listas, dicts, I/O) | `kyc_runtime/` | ✅ **Ahora** | `extern fn` a libc (`malloc`, `write`), mismo patrón que `packages/http/` |
| **Lexer + Parser** (texto → AST) | `kyc_frontend/` | ✅ **Ahora** | Lógica pura: `match`, enums, strings |
| **Type Checker** (tipos, scopes) | `kyc_semantic/` | ✅ **Ahora** | Lógica pura: algoritmos de inferencia |
| **MIR + SSA** (lowering, optimizaciones) | `kyc_mir/` | ✅ **Ahora** | Lógica pura: grafos, `for`, `match` |
| **LLVM codegen** (MIR → binario) | `kyc_backend/` | 🟡 FFI a C API | `extern fn` para ~200 funciones de LLVM C API |
| **Linker** (.o → ejecutable) | `kyc_backend/linker.rs` | ✅ **Ahora** | `system()` via FFI para llamar a clang/ld |
| **CLI** (`ky build`, `ky run`) | `kyc_cli/` | ✅ **Ahora** | `read_file()` + `system()` via FFI |

**La sintaxis completa de Kyle con ejemplos funcionales está en:** `docs/15-kyle-syntax-reference.md` (en este proyecto)

### Orden de migración

```
FASE 1 (2-3 sem): Runtime + Lexer + Parser en Kyle
  → El runtime Kyle se compila con el ky actual (Rust)
  → El parser Kyle se compila con el ky actual (Rust)
  → Todo Kyle, pero compilado por Rust

FASE 2 (4-6 sem): Type Checker + MIR + LLVM bindings en Kyle
  → Type checker + MIR: lógica pura en Kyle
  → LLVM: wrappear C API con extern fn
  → Sigue compilado por Rust

FASE 3 (1 sem): Bootstrap
  → El ky Rust compila el ky Kyle → binario ky (Kyle)
  → El ky (Kyle) compila su propio código fuente
  → Kyle corre Kyle. Rust ya no es necesario.
```

### Referencia de sintaxis rápida

```
Variables:        name = "hola"              # inmutable
                  count: ^i32 = 0            # mutable con ^
                  count += 1                 # += existe (no ++)
Funciones:        fn add(a: i32, b: i32) i32:
                      return a + b
                  fn view(s: &str):          # borrow con &
                  fn update(s: ^&str):       # mutable borrow
Extern fn:        @link "c"
                  extern fn malloc(size: i64) ptr
Control flow:     if/elif/else, while, for, match
Match:            match x: 0: "cero" 1: "uno" _: "otro"
Enum:             enum Color: Red(g: i32) Green Blue(b: i32, a: i32)
Clases:           final class Point: x: i32 y: i32
Listas:           items: ^{i32} = {1, 2, 3}; items.push(4)
Dicts:            scores: ^{str: i32} = {"ana": 100}
Opcional:         name: str? = none; if name: ...
Errores:          fn div(a: i32, b: i32) i32!: result = div(10, 0)!
Interpolación:    msg = "Hola, {name}!"
```

Ver la referencia completa en `docs/15-kyle-syntax-reference.md` para todos los detalles con ejemplos funcionales.

---

*Version: v0.8.4 · Last updated: 2026-07-16 — Fix field mutability + contracts + inheritance.*



