# Kyle Distribution & Infrastructure Plan v1.0

> How to make Kyle installable, usable, and discoverable by any developer
> on any platform with a single command.

---

## Table of Contents

1. [Overview](#1-overview)
2. [One-Command Installer](#2-one-command-installer)
3. [GitHub Actions & CI/CD](#3-github-actions--cicd)
4. [Website (kl-lang.org)](#4-website-kl-langorg)
5. [VS Code Extension Distribution](#5-vs-code-extension-distribution)
6. [Cross-Platform Strategy](#6-cross-platform-strategy)
7. [Branding & Logo](#7-branding--logo)
8. [Implementation Roadmap](#8-implementation-roadmap)

---

## 1. Overview

Kyle currently lives in a GitHub repository and must be built with `cargo build`.
For public adoption, we need:

| What | Why | Priority |
|------|-----|----------|
| Pre-compiled binaries | So users don't need Rust installed | 🔴 Critical |
| One-command installer | So installation is frictionless | 🔴 Critical |
| Website | So users can find docs, examples, downloads | 🟠 High |
| VS Code extension | So users get IDE support out of the box | 🟠 High |
| CI/CD | So releases are automatic and reliable | 🟡 Medium |
| Cross-platform | So it works on macOS, Linux, Windows | 🟡 Medium |
| Logo & branding | So it looks professional | 🟢 Nice-to-have |

---

## 2. One-Command Installer

### 2.1 — The Command

```bash
curl -fsSL https://kl-lang.org/install.sh | sh
```

That's it. No Rust, no LLVM, no dependencies.

### 2.2 — What install.sh Does

```
1. Detect OS and architecture
   ├── macOS ARM    → aarch64-apple-darwin
   ├── macOS Intel  → x86_64-apple-darwin
   ├── Linux x64    → x86_64-unknown-linux-gnu
   ├── Linux ARM    → aarch64-unknown-linux-gnu
   ├── Windows x64  → x86_64-pc-windows-msvc
   └── Windows ARM  → aarch64-pc-windows-msvc

2. Download binario precompilado
   → https://github.com/kynera/kl/releases/download/v{VERSION}/klc-{TARGET}.tar.gz

3. Verify SHA-256 checksum

4. Extraer klc binary

5. Instalar en:
   ├── /usr/local/bin/klc     (Linux/macOS — if writable)
   └── ~/.kl/bin/klc          (fallback)

6. Agregar ~/.kl/bin al PATH (si es necesario)

7. Mensaje de éxito:
   "Kyle v{VERSION} installed! Try: klc --version"
```

### 2.3 — Post-Install

```bash
klc --version                    # → klc v0.6.0
klc new my_project               # → creates src/main.kl + kl.toml
klc run my_project/src/main.kl   # → compiles and runs
klc fmt file.kl                  # → formats code
klc lsp                          # → starts language server
```

### 2.4 — Uninstall

```bash
rm /usr/local/bin/klc        # or ~/.kl/bin/klc
rm -rf ~/.kl                 # removes all Kyle data
```

### 2.5 — Windows Support

For Windows, the install script works in:
- **Git Bash** (full script support)
- **WSL** (Linux environment)
- **PowerShell** (alternative install.ps1 script)

A native Windows installer (`.msi` via WiX) is optional.

---

## 3. GitHub Actions & CI/CD

### 3.1 — CI Workflow (on every push/PR)

```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Install LLVM
        run: .github/scripts/install-llvm.sh
      - name: Build
        run: cargo build --workspace
      - name: Test
        run: cargo test -p klc_core -p klc_frontend -p klc_semantic -p klc_mir
      - name: Lint
        run: cargo clippy -- -D warnings
      - name: Build examples
        run: |
          for f in examples/*.kl; do
            cargo run --bin klc -- build "$f"
          done
```

### 3.2 — Release Workflow (on tag push)

```yaml
name: Release
on:
  push:
    tags: ["v*"]
jobs:
  build:
    strategy:
      matrix:
        target:
          - aarch64-apple-darwin
          - x86_64-apple-darwin
          - x86_64-unknown-linux-gnu
          - aarch64-unknown-linux-gnu
          - x86_64-pc-windows-msvc
    runs-on: ${{ matrix.target.runner }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Install LLVM
        run: .github/scripts/install-llvm-${{ matrix.target }}.sh
      - name: Build release
        run: cargo build --release
      - name: Package
        run: |
          tar czf klc-${{ matrix.target }}.tar.gz \
            -C target/release klc
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: klc-${{ matrix.target }}
          path: klc-${{ matrix.target }}.tar.gz

  release:
    needs: [build]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
      - name: Create Release
        uses: softprops/action-gh-release@v2
        with:
          files: klc-*.tar.gz
          generate_release_notes: true
```

### 3.3 — Package VS Code Extension

```yaml
  vscode:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - name: Package extension
        run: |
          cd vscode-kl
          npm install
          npx vsce package
      - uses: actions/upload-artifact@v4
        with:
          name: vscode-kl.vsix
          path: vscode-kl/*.vsix
```

### 3.4 — Publish Website

```yaml
  website:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build website
        run: |
          cd website
          npm install
          npm run build
      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v4
        with:
          publish_dir: ./website/dist
```

---

## 4. Website (kl-lang.org)

### 4.1 — Technology

| Component | Choice | Reason |
|-----------|--------|--------|
| Static site generator | **Hugo** (Go) | Fast, single binary, no JS runtime needed |
| OR | **Zola** (Rust) | Same language as Kyle compiler |
| Theme | Custom (minimal) | Must match Kyle branding |
| Hosting | **GitHub Pages** | Free, automatic from CI |
| Domain | kl-lang.org | Short, memorable |
| Analytics | Umami (self-hosted) | Privacy-respecting |

### 4.2 — Site Structure

```
kl-lang.org/
├── index.html              # Landing page (hero + features + install)
├── docs/
│   ├── index.md            # Getting started guide
│   ├── syntax.md           # Language reference
│   ├── types.md            # Type system
│   ├── stdlib.md           # Standard library API
│   ├── examples.md         # Example gallery
│   └── faq.md              # Frequently asked questions
├── download/
│   ├── index.md            # Downloads page
│   └── v0.6.0/             # Per-version binaries
├── blog/                   # Release announcements, tutorials
│   ├── index.md
│   ├── hello-world.md
│   └── v0.6.0-release.md
├── assets/
│   ├── css/
│   ├── js/
│   └── images/
│       ├── logo.svg
│       ├── logo-icon.svg
│       └── screenshots/
└── _redirects              # Short URLs
```

### 4.3 — Landing Page Content

```
┌─────────────────────────────────────────────────────┐
│  [Logo] Kyle                                         │
│  Readable like Python. Fast like C.                  │
│                                                      │
│  curl -fsSL https://kl-lang.org/install.sh | sh      │
│                                                      │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │
│  │ Python  │ │  Rust   │ │   Go    │ │  LLVM   │   │
│  │ Readable│ │  Type   │ │  Simple │ │  Fast   │   │
│  │ Syntax  │ │  Safety │ │  Tools  │ │  Code   │   │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘   │
│                                                      │
│  ## Hello, World!                                    │
│  fn main():                                          │
│      println("Hello, World!")                        │
│                                                      │
│  ## Features                                         │
│  • Indentation-based (like Python)                   │
│  • Strong static typing (like Rust)                  │
│  • RAII memory management (no GC)                    │
│  • Built-in async/await                              │
│  • Generics, enums, pattern matching                 │
│  • Native compilation via LLVM                        │
│                                                      │
│  Footer: GitHub | Docs | VS Code | Blog              │
└─────────────────────────────────────────────────────┘
```

### 4.4 — Color Palette

```css
/* Primary */
--kl-purple: #6C3FC5;      /* Main brand color */
--kl-dark:   #1A1A2E;      /* Background / dark mode */
--kl-white:  #FFFFFF;      /* Text on dark */

/* Secondary */
--kl-pink:   #E94560;      /* Accent */
--kl-blue:   #0F3460;      /* Secondary accent */
--kl-gray:   #F5F5F5;      /* Light background */

/* Code */
--kl-bg-code: #1E1E2E;     /* Code block background */
--kl-text-code: #CDD6F4;   /* Code text */
```

### 4.5 — SEO & Social

```html
<meta name="description" content="Kyle: A compiled programming language
combining Python readability, Rust type safety, and LLVM performance.">
<meta property="og:title" content="Kyle Programming Language">
<meta property="og:description" content="Readable like Python. Fast like C.">
<meta property="og:image" content="https://kl-lang.org/assets/images/og.png">
```

---

## 5. VS Code Extension Distribution

### 5.1 — Package as .vsix

```bash
cd vscode-kl
npm install
npx vsce package
# → vscode-kl-0.1.0.vsix
```

### 5.2 — Distribution Options

| Option | Effort | Reach |
|--------|--------|-------|
| Host on kl-lang.org | Low | Users download manually |
| VS Code Marketplace | Medium | Auto-updates, searchable |
| Both | Medium | Best coverage |

### 5.3 — Installation from VS Code

```bash
# From kl-lang.org:
curl -O https://kl-lang.org/downloads/vscode-kl.vsix
code --install-extension vscode-kl.vsix

# Or install directly:
code --install-extension https://kl-lang.org/downloads/vscode-kl.vsix
```

### 5.4 — Workspace Settings

After installation, VS Code automatically:
1. Activates the extension for `.kl` files
2. Launches `klc lsp` in the background
3. Provides syntax highlighting
4. Registers commands: `KL: Run`, `KL: Build`, `KL: Check`

---

## 6. Cross-Platform Strategy

### 6.1 — Why It's Easy

Kyle= is written in **Rust** + **LLVM**, both inherently cross-platform:

```
┌──────────────────────────────────────────────────────────┐
│                    Kyle Compiler (klc)                    │
├──────────────────────────────────────────────────────────┤
│  Rust code       → cross-platform via cargo              │
│  LLVM codegen    → cross-platform via inkwell            │
│  Runtime (Rust)  → cross-platform via std library        │
│  Linker (clang)  → cross-platform via clang              │
└──────────────────────────────────────────────────────────┘
```

### 6.2 — What Needs Platform-Specific Changes

Only **5 localized changes** are needed (Phase 7):

#### 6.2.1 — Runtime I/O (klc_runtime/src/io.rs)

**Problem:** Uses POSIX raw syscalls (`open`, `read`, `write`, `close`, `nanosleep`).

**Fix:** Replace with Rust `std::fs::File` + `std::io::{Read, Write}` + `std::thread::sleep`.

```rust
// Before (POSIX-only):
use libc::{open, read, write, close, O_RDONLY, O_WRONLY};

// After (cross-platform):
use std::fs::File;
use std::io::{Read, Write};
```

**Effort:** ~100 lines. Straightforward refactor.

#### 6.2.2 — Target Triple (klc_driver/src/pipeline.rs)

**Problem:** Hardcoded `"arm64-apple-macosx"` / `"aarch64-unknown-linux-gnu"`.

**Fix:** Use `TargetMachine::get_default_triple()` from inkwell.

```rust
// Before:
let target_triple = TargetTriple::create("arm64-apple-macosx");

// After:
let target_triple = TargetMachine::get_default_triple(&[]);
```

**Effort:** ~5 lines.

#### 6.2.3 — Linker (klc_backend/src/linker.rs)

**Problem:** Hardcoded `"clang"` command, no `.exe` extension, no Windows linker.

**Fix:** Conditional compilation with `cfg!(windows)`.

```rust
// After:
let linker = if cfg!(target_os = "windows") { "link" } else { "clang" };
let output_path = format!("{}{}", output_base, std::env::consts::EXE_SUFFIX);
```

**Effort:** ~20 lines.

#### 6.2.4 — LLVM Installation (CI scripts)

**Problem:** Only tested on macOS with brew and Linux with apt.

**Fix:** Create CI install scripts per platform:

```bash
# Linux (ubuntu):
sudo apt-get install llvm-18-dev libpolly-18-dev libzstd-dev

# macOS:
brew install llvm@18

# Windows:
choco install llvm
# Or download from https://github.com/llvm/llvm-project/releases
```

**Effort:** ~30 lines (3 scripts).

#### 6.2.5 — `.cargo/config.toml`

**Problem:** LLVM paths hardcoded per platform.

**Fix:** Use conditional `[target]` sections:

```toml
[target.'cfg(target_os = "macos")']
rustflags = ["-L", "/opt/homebrew/opt/llvm@18/lib"]

[target.'cfg(target_os = "linux")']
rustflags = ["-L", "/usr/lib/llvm-18/lib"]

[target.'cfg(target_os = "windows")']
rustflags = ["-L", "C:\\Program Files\\LLVM\\lib"]
```

**Effort:** ~10 lines.

### 6.3 — Platform Testing Matrix

| Platform | CI Runner | LLVM Install | Risk |
|----------|-----------|-------------|------|
| macOS ARM | `macos-latest` (ARM) | `brew install llvm@18` | Very Low (already works) |
| macOS Intel | `macos-13` (Intel) | `brew install llvm@18` | Low |
| Linux x64 | `ubuntu-latest` | `apt install llvm-18-dev` | Low |
| Linux ARM | `ubuntu-24.04-arm` | `apt install llvm-18-dev` | Low |
| Windows x64 | `windows-latest` | `choco install llvm` | Medium (linker) |
| Windows ARM | Not available | Cross-compile from x64 | High |

### 6.4 — Testing Strategy

```text
1. CI runs ALL tests on macOS ARM (already works)
2. Add Linux x64 to CI (highest priority after macOS)
3. Add macOS Intel to CI
4. Add Windows x64 to CI
5. Manual test on Linux ARM
6. Cross-compile for Windows ARM (lowest priority)
```

---

## 7. Branding & Logo

### 7.1 — Logo Requirements

| Format | Usage | Status |
|--------|-------|--------|
| SVG logo (full) | Website header, docs | ❌ Need design |
| SVG icon (square) | VS Code, favicon, GitHub avatar | ❌ Need design |
| PNG (512x512) | VS Code marketplace | ❌ Need design |
| PNG (og:image) | Social media cards | ❌ Need design |

### 7.2 — Logo Design Brief

```
Style: Modern, minimal, geometric
Colors: Purple (#6C3FC5) + Dark (#1A1A2E)
Elements:
  - Stylized "K" or "kl" monogram
  - Clean lines, no gradients
  - Works in both light and dark mode
  - Recognizable at small sizes (16px favicon)
```

### 7.3 — File Icon for .kl files

VS Code extension already has a partial icon theme (`vscode-kl/icons/`).
Need a distinctive icon for `.kl` files:
- Purple "K" on white background for light themes
- White "K" on dark background for dark themes

---

## 8. Implementation Roadmap

### Sprint 1 — Cross-Platform (Phase 7, ~1 week)

| Task | Effort | Owner |
|------|--------|-------|
| Refactor runtime I/O to use std::fs | 1 day | |
| Auto-detect target triple | 2 hours | |
| Platform-specific linker | 4 hours | |
| CI: add Linux x64 runner | 1 day | |
| CI: add macOS Intel runner | 1 day | |
| CI: add Windows x64 runner | 1 day | |
| Fix cross-platform issues found in CI | 2 days | |

### Sprint 2 — CI/CD & Installer (Phase 8, ~1 week)

| Task | Effort | Owner |
|------|--------|-------|
| Create install.sh script | 1 day | |
| Create GitHub Actions release workflow | 1 day | |
| Package VS Code extension (.vsix) | 4 hours | |
| Test install.sh on all platforms | 1 day | |
| Publish first release binary | 4 hours | |

### Sprint 3 — Website (Phase 8, ~1 week)

| Task | Effort | Owner |
|------|--------|-------|
| Design logo and branding | 2 days | |
| Set up Hugo/Zola static site | 1 day | |
| Build landing page | 1 day | |
| Build docs section | 2 days | |
| Build download section | 1 day | |
| Configure domain + GitHub Pages | 1 day | |

### Sprint 4 — LSP & VS Code Polish (Phase 8, ~1 week)

| Task | Effort | Owner |
|------|--------|-------|
| Implement LSP completion handler | 1 day | |
| Implement LSP goto-definition | 1 day | |
| Implement LSP hover | 1 day | |
| VS Code compile-on-save | 4 hours | |
| VS Code error squiggles | 4 hours | |
| Publish to VS Code Marketplace | 1 day | |

---

## Version

```text
Distribution & Infrastructure Plan v1.0
Last updated: 2026-06-25
```
