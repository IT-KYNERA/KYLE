# Modules, Packages & Tooling

> How Kyle projects are organized, how dependencies are managed, and how
> the command-line tools work.

---

## 1. Getting Started

### 1.1 Install

One line for the language:

```bash
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh | sh
```

One line for the VS Code extension:

```bash
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/vscode-ky/install-extension.sh | sh
```

Verify:

```bash
ky --version
# ky v0.2.2
```

### 1.2 Create a Project

```bash
ky new myapp
cd myapp
```

Generated layout:

```
myapp/
├── src/
│   ├── main.ky           ← entry point (fn main)
│   └── lib.ky            ← library code (optional)
├── tests/
│   └── test_main.ky      ← test stubs
├── .vscode/
│   └── settings.json     ← editor config
├── ky.toml               ← project manifest
├── README.md             ← project README
└── .gitignore
```

### 1.3 Build & Run

```bash
# Run in debug mode (fast compile, slow execution)
ky run

# Build to native binary
ky build
# → ./myapp

# Build optimized release binary
ky build --release
# → target/release/myapp

# Type-check without building
ky check src/main.ky
```

---

## 2. Module System

### 2.1 Import Forms

Kyle has four import forms:

```ky
# 1. Import a whole module
import math                # use: math.sqrt(2.0)

# 2. Import a module under an alias
import math as m            # use: m.sqrt(2.0)

# 3. Import one symbol from a module
from str import capitalize   # use: capitalize("hi")

# 4. Import one symbol under an alias
from str import capitalize as cap  # use: cap("hi")
```

### 2.2 Relative Imports

Use `~` to import relative to the current file:

```ky
# In src/widgets/button.ky
import ~helpers    # → src/widgets/helpers.ky
import ~~shared    # → src/shared.ky
import ~~~../utils # → src/utils.ky
```

Each `~` goes up one directory.

### 2.3 Module Resolution Order

When you write `import x`, the compiler searches in this order:

1. The directory of the importing file
2. The project's `src/` directory
3. `cwd/std/`
4. The compiler's bundled `std/` directory

The first file matching `x.ky` is loaded.

### 2.4 Visibility (Module-Level)

Module-level declarations use the same underscore convention as class members:

| Form | Visibility | Importable from |
|---|---|---|
| `name` | public | any module |
| `_name` | protected | same package only (not yet enforced) |
| `__name` | private | not importable |

The leading underscores are stripped from the import name.

```ky
# In str/uppercase.ky
fn __internal_helper():
    # ...

fn uppercase(s: str) str:
    return __internal_helper() + s
```

```ky
# In another file
from str import uppercase
# from str import __internal_helper  ← compile error: private
```

### 2.5 Module Limitations

- ❌ No nested module paths (`import a.b.c`)
- ❌ No package-qualified imports (`import mypkg.str`)
- ❌ No cyclic imports
- ❌ No conditional imports

---

## 3. Package Manager

### 3.1 The Manifest (`ky.toml`)

Every project has a `ky.toml`:

```toml
[project]
name = "myapp"
version = "0.1.0"
authors = ["Your Name"]
description = "My Kyle application"
license = "MIT"
edition = "2024"

[dependencies]
math = "1.0.0"
json = "2.1.0"

[dev-dependencies]
testing = "1.0.0"
```

The compiler reads `name`, `version`, and `dependencies` from the manifest.
The other fields are stored but not currently used by the compiler.

### 3.2 The Lock File (`ky.lock`)

After a successful build, `ky.lock` is generated with the resolved versions
of all dependencies. The lock file is currently a stub — real dependency
resolution is planned for Phase 13.

### 3.3 Commands

| Command | Description | Status |
|---|---|---|
| `ky new <name>` | Create a new project from a template | ✅ |
| `ky add <dep@version>` | Add a dependency to `ky.toml` | ✅ (manifest only) |
| `ky remove <dep>` | Remove a dependency from `ky.toml` | ✅ (manifest only) |
| `ky info` | Show project metadata | ✅ |
| `ky build` | Build the project | ✅ |
| `ky run` | Build and run | ✅ |
| `ky test` | Run tests in `tests/` | 🔶 (type-checks only) |

Dependency **resolution** and **fetching** are not yet implemented. The
package manager currently only manages the manifest.

### 3.4 Package Ecosystem Plan (Post-v1.0)

> **TL;DR:** Monorepo para desarrollo, GitHub Releases como registry inicial,
> separar a repos independientes cuando haya 20+ paquetes estables.

#### 3.4.1 Architecture — 3 Components

```
┌─────────────────────────────────────────────────────────────────┐
│  1. Kyle Compiler (github.com/IT-KYNERA/kyle)                   │
│     El binario `ky` incluye: compilador + runtime + std +       │
│     package manager (kyc_tools). Todo en un mismo repo.         │
│                                                                 │
│     Cuando el usuario escribe `ky add json`:                    │
│       1. Resuelve semver desde ky.toml                          │
│       2. HTTP GET → registry/v1/packages/json                   │
│       3. Descarga .tar.gz, extrae a ~/.ky/cache/json-1.0.0/    │
│       4. import json busca en ~/.ky/cache/                      │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  2. Registry Server (github.com/IT-KYNERA/ky-registry)          │
│     Servidor HTTP independiente (Rust, axum). Sirve .tar.gz     │
│     de paquetes. Hosteado en VPS o como GitHub Releases.        │
│                                                                 │
│     GET /v1/packages/:name           → lista versiones          │
│     GET /v1/packages/:name/:ver/download → .tar.gz              │
│     PUT /v1/packages/:name/:ver/upload ← publish (auth)         │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  3. Official Packages (github.com/IT-KYNERA/ky-packages)        │
│     Código fuente de paquetes oficiales. Cada uno es un         │
│     proyecto Kyle independiente con su propio ky.toml.          │
│                                                                 │
│     packages/                                                   │
│     ├── json/        ky.toml + src/lib.ky                       │
│     ├── http/        ky.toml + src/lib.ky                       │
│     ├── postgresql/  ky.toml + src/lib.ky                       │
│     ├── sqlite/      ky.toml + src/lib.ky                       │
│     ├── crypto/      ky.toml + src/lib.ky                       │
│     └── registry.json  ← índice generado automáticamente        │
└─────────────────────────────────────────────────────────────────┘
```

#### 3.4.2 Monorepo Development (Current/Fase Actual)

Durante el desarrollo del lenguaje, **todo vive en el mismo repo:**

```
kyle/
├── crates/           (compiler: 9 crates)
├── kyc_runtime/      (runtime Rust)
├── std/              (stdlib: core.ky, io.ky, math.ky...)
├── packages/         ← NUEVO — paquetes oficiales en desarrollo
│   ├── json/
│   │   ├── ky.toml
│   │   └── src/lib.ky
│   ├── http/
│   └── postgresql/
├── docs/
└── ky.toml           (root manifest)
```

**Path dependencies** — un proyecto de prueba declara:

```toml
[dependencies]
json = { path = "../kyle/packages/json" }
```

El compilador resuelve la ruta local, compila el paquete como un módulo más,
sin descargar nada de Internet. Esto permite iterar a velocidad de luz.

#### 3.4.3 GitHub Releases as Registry (Fase Intermedia)

Cuando los paquetes estén estables, se publican como GitHub Releases:

```bash
# publish.sh — script interno
cd packages/json
git tag v1.0.0
git push origin v1.0.0
# GitHub Action crea Release con .tar.gz
```

El `registry.json` estático se sirve via GitHub Pages:

```json
{
  "json": {
    "1.0.0": {
      "url": "https://github.com/IT-KYNERA/ky-packages/releases/download/json-1.0.0.tar.gz",
      "checksum": "sha256:a1b2c3..."
    }
  }
}
```

#### 3.4.4 Package Format

Cada paquete es un `.tar.gz` con:

```
json-1.0.0/
├── ky.toml           ← name, version, dependencies
├── src/
│   └── lib.ky        ← entry point (pub fn exports)
├── tests/
├── README.md
└── LICENSE
```

Convenciones:
- Entry point: `src/lib.ky`
- `pub fn` = API pública
- `_fn` = protected (mismo paquete)
- `__fn` = private

#### 3.4.5 Path Dependencies Implementation

El compilador debe soportar `{ path = "..." }` en `ky.toml`:

| Archivo | Cambio |
|---------|--------|
| `kyc_tools/src/package/manifest.rs` | Parsear `Dependency::Path(String)` |
| `kyc_driver/src/pipeline.rs` | Resolver imports desde `packages/` + cache paths |
| `kyc_frontend/src/parser.rs` | Search paths para módulos de paquetes |

**Module resolution order (actualizado):**
```
import json → busca en:
  1. Paquetes instalados en ~/.ky/cache/json-*/src/ (desde ky.lock)
  2. packages/json/ (desarrollo local)
  3. Directorio del archivo actual
  4. src/ del proyecto
  5. std/ (librería estándar)
```

#### 3.4.6 Future: Independent Repositories

Cuando haya 20+ paquetes estables y contribuidores externos:

```
github.com/IT-KYNERA/
├── kyle/           ← compiler + runtime + std + tools
├── ky-registry/    ← registry server (Rust, axum)
└── KylePackages/   ← organización de paquetes oficiales
    ├── json/
    ├── http/
    ├── postgresql/
    ├── sqlite/
    ├── websocket/
    ├── crypto/
    ├── image/
    ├── xml/
    ├── yaml/
    └── jwt/
```

El usuario escribe exactamente lo mismo: `ky add postgres`.
La única diferencia es que el cliente descarga desde `packages.kylelang.org`
en lugar de GitHub Releases.

---

## 4. Build System

### 4.1 Project Mode

When `ky` is run inside a directory with a `ky.toml`, it operates in project
mode. The entry point is `src/main.ky` (or the file specified in `[project]
main`).

```bash
cd myapp
ky run          # uses src/main.ky
ky build        # produces ./myapp
ky build --release  # optimized
```

### 4.2 Single-File Mode

When `kl <command> <file.ky>` is run with an explicit file, it operates in
single-file mode. The given file is the entry point, and `ky.toml` is
ignored.

```bash
ky run hello.ky
ky check my_script.ky
```

### 4.3 Build Artifacts

```
myapp/
└── target/
    ├── debug/                # debug builds
    │   ├── myapp             # executable
    │   ├── myapp.ll          # LLVM IR (debug)
    │   └── myapp.o           # object file
    └── release/              # optimized builds
        └── myapp
```

The `target/` directory is gitignored.

### 4.4 Build Options

| Flag | Effect |
|---|---|
| `--release` | Use optimized codegen (planned — currently same as debug) |
| `--check` | Type-check only, don't generate code (alias for `ky check`) |
| `--verbose` | Print compilation steps |
| `--no-color` | Disable ANSI colors in output |

---

## 5. CLI Reference

### 5.1 Project Commands

Run these from the project root (where `ky.toml` lives):

| Command | Description |
|---|---|
| `ky new <name>` | Create a new project |
| `ky run` | Build and execute |
| `ky build` | Compile to native binary |
| `ky build --release` | Compile with optimizations |
| `ky test` | Run project tests |
| `ky add <dep@ver>` | Add a dependency |
| `ky remove <dep>` | Remove a dependency |
| `ky info` | Show project info |
| `ky fmt` | Format all `.ky` files in `src/` |
| `ky completions <shell>` | Print shell completion script |

### 5.2 File Commands

Run these with an explicit file:

| Command | Description |
|---|---|
| `ky run <file>` | Compile and execute |
| `ky check <file>` | Type-check only (no codegen, fast) |
| `ky build <file>` | Compile to native binary |
| `ky parse <file>` | Print the AST as text |
| `ky mir <file>` | Print the MIR (mid-level IR) |
| `ky fmt <file>` | Format the file in-place |
| `ky lsp` | Start the language server (stdio) |

### 5.3 Global Flags

| Flag | Effect |
|---|---|
| `--help` | Show usage |
| `--version` | Show version |
| `--verbose` | Verbose output |
| `--no-color` | Disable colors |

---

## 6. VS Code Extension

### 6.1 Install

Instalación en una línea:

```bash
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/vscode-ky/install-extension.sh | sh
```

El script detecta automáticamente VS Code, construye desde source (Node 20+)
o descarga el VSIX pre-compilado, y lo instala.

Then in VS Code, run **Developer: Reload Window** (`Cmd+Shift+P`).

### 6.2 Features

- **Syntax highlighting** — keywords, types, builtins, strings, numbers, operators
- **Semantic coloring** — variables, types, functions, parameters (LSP semantic tokens)
- **Autocompletion** — 44 builtins, all declarations, keywords
- **Dot-completions** — `obj.` shows fields and methods
- **Scope-aware** — local variables, function params, block-scoped declarations
- **Go-to-definition** — jump to declaration of any symbol
- **Hover** — function signatures, builtin docs, identifier info
- **Snippets** — 15 common patterns (fn, class, enum, match, for, if, etc.)
- **Commands** — `KL: Run`, `KL: Build`, `KL: Check` from the command palette
- **Rename refactor** — F2 to rename any symbol
- **Format on save** — Shift+Option+F (configurable)

### 6.3 Configuration

| Setting | Type | Default | Description |
|---|---|---|---|
| `ky.kycPath` | string | `"kl"` | Path to the `ky` binary |
| `kl.semanticHighlighting` | bool | `true` | Enable semantic colors |

### 6.4 Extension Activation

The extension activates when:

- A `.ky` file is opened
- The user runs `KL: Run`, `KL: Build`, or `KL: Check`

If the language server fails to start, the extension falls back to syntax
highlighting and snippets only.

---

*Version: v0.5.0 · Last updated: 2026-07-02*
