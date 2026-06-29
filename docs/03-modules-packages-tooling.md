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
curl -fsSL -o /tmp/kl.vsix https://github.com/IT-KYNERA/KYLE/releases/latest/download/kl-0.2.2.vsix
code --install-extension /tmp/kl.vsix
```

Verify:

```bash
kl --version
# kl v0.2.2
```

### 1.2 Create a Project

```bash
kl new myapp
cd myapp
```

Generated layout:

```
myapp/
├── src/
│   ├── main.kl           ← entry point (fn main)
│   └── lib.kl            ← library code (optional)
├── tests/
│   └── test_main.kl      ← test stubs
├── .vscode/
│   └── settings.json     ← editor config
├── kl.toml               ← project manifest
├── README.md             ← project README
└── .gitignore
```

### 1.3 Build & Run

```bash
# Run in debug mode (fast compile, slow execution)
kl run

# Build to native binary
kl build
# → ./myapp

# Build optimized release binary
kl build --release
# → target/release/myapp

# Type-check without building
kl check src/main.kl
```

---

## 2. Module System

### 2.1 Import Forms

Kyle has four import forms:

```kl
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

```kl
# In src/widgets/button.kl
import ~helpers    # → src/widgets/helpers.kl
import ~~shared    # → src/shared.kl
import ~~~../utils # → src/utils.kl
```

Each `~` goes up one directory.

### 2.3 Module Resolution Order

When you write `import x`, the compiler searches in this order:

1. The directory of the importing file
2. The project's `src/` directory
3. `cwd/std/`
4. The compiler's bundled `std/` directory

The first file matching `x.kl` is loaded.

### 2.4 Visibility (Module-Level)

Module-level declarations use the same underscore convention as class members:

| Form | Visibility | Importable from |
|---|---|---|
| `name` | public | any module |
| `_name` | protected | same package only (not yet enforced) |
| `__name` | private | not importable |

The leading underscores are stripped from the import name.

```kl
# In str/uppercase.kl
fn __internal_helper():
    # ...

fn uppercase(s: str) str:
    return __internal_helper() + s
```

```kl
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

### 3.1 The Manifest (`kl.toml`)

Every project has a `kl.toml`:

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

### 3.2 The Lock File (`kl.lock`)

After a successful build, `kl.lock` is generated with the resolved versions
of all dependencies. The lock file is currently a stub — real dependency
resolution is planned for Phase 13.

### 3.3 Commands

| Command | Description | Status |
|---|---|---|
| `kl new <name>` | Create a new project from a template | ✅ |
| `kl add <dep@version>` | Add a dependency to `kl.toml` | ✅ (manifest only) |
| `kl remove <dep>` | Remove a dependency from `kl.toml` | ✅ (manifest only) |
| `kl info` | Show project metadata | ✅ |
| `kl build` | Build the project | ✅ |
| `kl run` | Build and run | ✅ |
| `kl test` | Run tests in `tests/` | 🔶 (type-checks only) |

Dependency **resolution** and **fetching** are not yet implemented. The
package manager currently only manages the manifest.

---

## 4. Build System

### 4.1 Project Mode

When `kl` is run inside a directory with a `kl.toml`, it operates in project
mode. The entry point is `src/main.kl` (or the file specified in `[project]
main`).

```bash
cd myapp
kl run          # uses src/main.kl
kl build        # produces ./myapp
kl build --release  # optimized
```

### 4.2 Single-File Mode

When `kl <command> <file.kl>` is run with an explicit file, it operates in
single-file mode. The given file is the entry point, and `kl.toml` is
ignored.

```bash
kl run hello.kl
kl check my_script.kl
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
| `--check` | Type-check only, don't generate code (alias for `kl check`) |
| `--verbose` | Print compilation steps |
| `--no-color` | Disable ANSI colors in output |

---

## 5. CLI Reference

### 5.1 Project Commands

Run these from the project root (where `kl.toml` lives):

| Command | Description |
|---|---|
| `kl new <name>` | Create a new project |
| `kl run` | Build and execute |
| `kl build` | Compile to native binary |
| `kl build --release` | Compile with optimizations |
| `kl test` | Run project tests |
| `kl add <dep@ver>` | Add a dependency |
| `kl remove <dep>` | Remove a dependency |
| `kl info` | Show project info |
| `kl fmt` | Format all `.kl` files in `src/` |
| `kl completions <shell>` | Print shell completion script |

### 5.2 File Commands

Run these with an explicit file:

| Command | Description |
|---|---|
| `kl run <file>` | Compile and execute |
| `kl check <file>` | Type-check only (no codegen, fast) |
| `kl build <file>` | Compile to native binary |
| `kl parse <file>` | Print the AST as text |
| `kl mir <file>` | Print the MIR (mid-level IR) |
| `kl fmt <file>` | Format the file in-place |
| `kl lsp` | Start the language server (stdio) |

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

The extension `.vsix` is a single file in each release. Install it once:

```bash
curl -fsSL -o /tmp/kl.vsix https://github.com/IT-KYNERA/KYLE/releases/latest/download/kl-0.2.2.vsix
code --install-extension /tmp/kl.vsix
```

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
| `kl.klcPath` | string | `"kl"` | Path to the `kl` binary |
| `kl.semanticHighlighting` | bool | `true` | Enable semantic colors |

### 6.4 Extension Activation

The extension activates when:

- A `.kl` file is opened
- The user runs `KL: Run`, `KL: Build`, or `KL: Check`

If the language server fails to start, the extension falls back to syntax
highlighting and snippets only.

---

*Version: v0.3.0 · Last updated: 2026-06-28*
