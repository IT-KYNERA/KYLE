# Kyle Platform Architecture

**Version:** 2.0  
**Status:** Approved  
**Date:** 2026-07-03  
**Scope:** Entire Kyle Ecosystem — language, runtime, platform, packages, tooling, desktop, OS

---

## Table of Contents

1. [Philosophy](#1-philosophy)
2. [Architectural Principles](#2-architectural-principles)
3. [Ecosystem Layers](#3-ecosystem-layers)
4. [Layer Definitions](#4-layer-definitions)
5. [Project Structure](#5-project-structure)
6. [Package System](#6-package-system)
7. [File Types: `.ky` vs `.kyx`](#7-file-types-ky-vs-kyx)
8. [FFI Strategy: extern, @link, ptr](#8-ffi-strategy)
9. [Implementation Phases](#9-implementation-phases)
10. [Current vs Target State](#10-current-vs-target-state)

---

## 1. Philosophy

Kyle is not a programming language with a UI framework bolted on.  
It is a **complete software development platform**.

Every component follows a single principle:

> **A layer only knows the layer immediately below it.**

This guarantees:
- Long-term scalability
- Independent evolution of each subsystem
- Platform portability
- High-performance native execution
- Future kernel replacement without affecting applications

The ecosystem must allow applications, tools, desktop environments, and future operating systems to share **exactly the same architecture**.

---

## 2. Architectural Principles

### Single Language
Kyle is the **only** programming language in the ecosystem.  
Console apps, desktop apps, mobile apps, web apps, system apps, and KyleOS all use Kyle.

### Single Platform API
Every operating system is accessed through a unified **Kyle Platform** API.  
Applications never communicate directly with Windows, Linux, macOS, Android, iOS, or the browser.

### Single Visual System
The same rendering architecture is used for:
- User applications
- Kyle Desktop shell
- KyleOS graphical environment
- IDE, package manager, file explorer, settings, login screen

There are no separate UI technologies. Everything runs on the same visual engine.

### Platform Independence
No layer above Kyle Platform contains platform-specific logic.  
Platform-specific implementations are isolated inside **Platform Adapters**.

### Package Manager is Horizontal
The package manager is **not a layer**. It is a **cross-cutting service** that manages distribution and dependencies for all layers equally.

---

## 3. Ecosystem Layers

```
┌──────────────────────────────────────────────────────────┐
│                    Applications                           │
│  (CLI tools, IDE, Servers, Desktop, Web Apps, KyleOS)    │
├──────────────────────────────────────────────────────────┤
│                      Kyle UI                              │
│  (Button, TextBox, Grid, ListView, Tabs, Dialog, Menu...)│
├──────────────────────────────────────────────────────────┤
│                     Kyle Scene                            │
│  (Scene graph, Layout engine, Event dispatch, Hit test,   │
│   Dirty regions, Animation scheduling, Render tree gen)   │
├──────────────────────────────────────────────────────────┤
│                   Kyle Graphics                           │
│  (Canvas, Text rendering, Images, SVG, Paths, Shapes,     │
│   Gradients, GPU commands, Shaders, Textures, Surfaces)   │
├──────────────────────────────────────────────────────────┤
│                  Kyle Windowing                           │
│  (Window creation, Event loop, Keyboard, Mouse, Touch,    │
│   Clipboard, Drag & drop, Cursor, Screen info)            │
├──────────────────────────────────────────────────────────┤
│                  Kyle Platform                            │
│  (FileSystem, Networking, Threads, Async, Audio,          │
│   GPU context, Environment, Time, Random, Process,        │
│   Sensors, USB, Bluetooth, Camera, Notifications,         │
│   Dialogs, Timers)                                        │
├──────────────────────────────────────────────────────────┤
│                  Kyle Runtime                             │
│  (Memory allocation, Refcounting, Strings, Lists, Dicts,  │
│   Panic handling, Error types, Reflection, Type info)     │
├──────────────────────────────────────────────────────────┤
│                Kyle Language (Compiler)                   │
│  (.ky → Lexer → Parser → HIR → Semantic → MIR → SSA      │
│   → LLVM IR → Optimize → Native Binary)                  │
└──────────────────────────────────────────────────────────┘

  ╔═══════════════════════════════════════════════════════╗
  ║            Package Manager (cross-cutting)              ║
  ║  Registry · Cache · Lock · Publish · Install            ║
  ║  Works for ALL layers — not part of any single layer    ║
  ╚═══════════════════════════════════════════════════════╝
```

---

## 4. Layer Definitions

### 4.1 Kyle Language (Layer 0)

The compiler. Transforms `.ky` source code into native executables.

| Component | Status |
|-----------|--------|
| Lexer → Parser → AST | ✅ Complete |
| HIR (desugaring) | ✅ Complete |
| Semantic Analysis (type checking, scope) | ✅ Complete |
| MIR (lowering, borrow analysis) | ✅ Complete |
| SSA Form (mem2reg, phi, GVN) | ✅ Complete |
| LLVM Codegen (inkwell) | ✅ Complete |
| Optimization Pipeline (O3, nsw, TBAA) | ✅ Complete |
| **`extern fn` declarations** | 🔜 **Phase 0 — next** |
| **`@link` directives** | 🔜 **Phase 0 — next** |
| **`ptr` arithmetic (load/store/offset)** | 🔜 **Phase 0 — next** |
| **WASM target** | 📅 Future |

### 4.2 Kyle Runtime (Layer 1)

Language-level services. **No OS dependencies.**

| Service | Current | Target |
|---------|---------|--------|
| Memory allocator (refcounted) | ✅ `libkyc_runtime.a` | ✅ Same |
| Strings (concat, slice, search) | ✅ `libkyc_runtime.a` | ✅ Same |
| Lists (dynamic array) | ✅ `libkyc_runtime.a` | ✅ Same |
| Dicts (hash map) | ✅ `libkyc_runtime.a` | ✅ Same |
| Panic handling | ✅ `libkyc_runtime.a` | ✅ Same |
| Type reflection | ❌ Missing | 🔜 `ky:reflect` package |
| JSON (runtime calls) | ✅ `libkyc_runtime.a` | 🔜 Move to `ky-json` package |
| File I/O (open/read/write/close) | ✅ `libkyc_runtime.a` | 🔜 **Move to Kyle Platform** |
| Threads / Async | ✅ `libkyc_runtime.a` | 🔜 **Move to Kyle Platform** |

**Rule:** Everything that depends on the OS moves OUT of Runtime into Platform.

### 4.3 Kyle Platform (Layer 2)

The bridge between Kyle and the operating system.  
Defines **platform-independent interfaces** with **concrete implementations per OS**.

| Module | Description | Status |
|--------|-------------|--------|
| `ky:fs` | FileSystem: read, write, list, mkdir, remove, stat | 🔜 **Phase 1** |
| `ky:net` | Networking: TCP, UDP, DNS, TLS | 🔜 **Phase 1** |
| `ky:http` | HTTP/1.1 client + server (pure Kyle after extern/ptr) | 🔜 **Phase 2** |
| `ky:time` | Time, timers, sleep | 🔜 **Phase 1** |
| `ky:env` | Environment variables, process info | 🔜 **Phase 1** |
| `ky:thread` | Thread spawn, join, sync primitives | 🔜 **Phase 2** |
| `ky:async` | Async runtime, task spawn, await | 🔜 **Phase 2** |
| `ky:audio` | Audio playback, recording | 📅 Future |
| `ky:gpu` | GPU context, device info | 📅 Future |
| `ky:sensors` | Accelerometer, gyroscope, GPS | 📅 Future |
| `ky:usb` | USB device enumeration, communication | 📅 Future |
| `ky:bluetooth` | BLE, classic Bluetooth | 📅 Future |

### 4.4 Kyle Windowing (Layer 3)

Window management and input event processing.

| Module | Description | Status |
|--------|-------------|--------|
| `ky:window` | Create/manage OS windows | 📅 Future |
| `ky:events` | Keyboard, mouse, touch, gamepad input | 📅 Future |
| `ky:clipboard` | System clipboard access | 📅 Future |
| `ky:dnd` | Drag and drop support | 📅 Future |
| `ky:cursor` | Cursor shape, visibility | 📅 Future |
| `ky:display` | Screen info (resolution, DPI, refresh rate) | 📅 Future |

### 4.5 Kyle Graphics (Layer 4)

2D/3D rendering engine. Hardware-accelerated via GPU backends.

| Module | Description | Status |
|--------|-------------|--------|
| `ky:canvas` | 2D drawing (lines, rectangles, circles, paths) | 📅 Future |
| `ky:text` | Text layout, font rendering, glyph shaping | 📅 Future |
| `ky:image` | Image decoding (PNG, JPEG, WebP), compositing | 📅 Future |
| `ky:gpu` | GPU command buffer, shaders, textures, render passes | 📅 Future |
| `ky:svg` | SVG parsing and rendering | 📅 Future |

**Backends:** Skia, Direct2D, Vulkan, Metal, WebGPU, Canvas2D (swap via adapter)

### 4.6 Kyle Scene (Layer 5)

The scene graph that organizes visual elements.

| Module | Description | Status |
|--------|-------------|--------|
| `ky:scene` | Scene graph tree, dirty region management | 📅 Future |
| `ky:layout` | Layout engine (flex, grid, absolute) | 📅 Future |
| `ky:events` | Hit testing, event dispatch, focus system | 📅 Future |
| `ky:animate` | Animation scheduling, keyframes, transitions | 📅 Future |
| `ky:render` | Render tree generation from scene graph | 📅 Future |

### 4.7 Kyle UI (Layer 6)

Reusable user interface widgets.

| Widget | Description | Status |
|--------|-------------|--------|
| Button, Label, TextBox | Core form controls | 📅 Future |
| Grid, Row, Column, Stack | Layout containers | 📅 Future |
| ListView, TreeView, Table | Data display widgets | 📅 Future |
| Tabs, Dialog, Menu, Tooltip | Navigation and overlays | 📅 Future |
| ScrollView, ProgressBar | Utility widgets | 📅 Future |
| Canvas, Markdown, CodeEditor | Advanced widgets | 📅 Future |
| Charts, VideoPlayer, Map | Media and data viz | 📅 Future |

### 4.8 Applications (Layer 7)

Everything built on top of Kyle UI. No architectural distinction between system apps and user apps — the difference is only in permissions.

| Example | Description |
|---------|-------------|
| Kyle IDE | Code editor, debugger, project manager |
| Kyle Explorer | File browser |
| Kyle Terminal | Terminal emulator |
| Kyle Browser | Web browser |
| Kyle Store | Application store |
| Kyle Settings | System settings |
| Kyle Desktop | Desktop shell, launcher, taskbar, notification center |
| Login Manager | Display manager, session management |

### 4.9 Platform Adapters

Each supported OS implements the Platform API independently.

| Adapter | Status |
|---------|--------|
| Linux (kernel) | 📅 Future |
| macOS | 🔜 **Phase 1** (first target, we're on it now) |
| Windows | 📅 Future |
| Android | 📅 Future |
| iOS | 📅 Future |
| Web (WASM + Browser APIs) | 📅 Future |
| KyleOS (Linux-based → custom kernel) | 📅 Future |

---

## 5. Project Structure

```
ky/                                         # Workspace root
├── Cargo.toml                              # Workspace manifest (10→12 crates)
│
├── crates/
│   ├── kyc_core/                           # ✅ Foundation (AST, types, diagnostics)
│   ├── kyc_frontend/                       # ✅ Lexer + Parser
│   ├── kyc_hir/                            # ✅ HIR desugaring
│   ├── kyc_semantic/                       # ✅ Type checker, scope, borrow analysis
│   ├── kyc_mir/                            # ✅ MIR lowering + SSA
│   ├── kyc_backend/                        # ✅ LLVM codegen + linker
│   ├── kyc_driver/                         # ✅ Pipeline orchestration
│   ├── kyc_cli/                            # ✅ CLI binary (`ky`)
│   ├── kyc_runtime/                        # ✅ Minimal runtime (memory, strings, lists, dicts)
│   ├── kyc_tools/                          # ✅ LSP, formatter, package manager, completions
│   │
│   ├── kyc_platform/                       # 🔜 NUEVO: Platform API (FS, Net, Time, Env)
│   ├── kyc_platform_macos/                 # 🔜 NUEVO: macOS adapter
│   └── kyc_platform_linux/                 # 🔜 NUEVO: Linux adapter
│
├── packages/                               # 🔜 Kyle packages (registry-distributable)
│   ├── ky-http/
│   ├── ky-json/
│   ├── ky-sqlite/
│   ├── ky-postgres/
│   ├── ky-websocket/
│   ├── ky-crypto/
│   ├── ky-testing/
│   └── ...                                 # More as needed
│
├── std/                                    # ✅ Standard library (.ky files)
├── docs/                                   # ✅ Documentation
├── tests/                                  # ✅ End-to-end test files
├── vscode-ky/                              # ✅ VS Code extension
├── examples/                               # ✅ Example projects
│
└── tools/                                  # Build scripts, CI helpers
```

---

## 6. Package System

### 6.1 Package Types

| Type | Extension | Description |
|------|-----------|-------------|
| **Source** | `.ky` | Kyle source code. Platform-independent. |
| **Source package** | `.tar.gz` (registry) | Published source code, compiled on install. |
| **Compiled extension** | `.kyx` | Pre-compiled native library (`.so`/`.dylib`/`.dll`) with Kyle FFI bindings. |
| **Extension package** | `.tar.gz` (registry) | Contains `.ky` wrappers + `.kyx` binaries per platform. |

### 6.2 Package Directory Structure

**Pure Kyle package** (no native code):
```
ky-testing-1.0.0/
├── ky.toml                  # Package metadata
├── src/
│   └── lib.ky               # Public API — imported via `from ky-testing import ...`
├── tests/
│   └── test_assertions.ky
├── README.md
└── LICENSE
```

**Hybrid package** (Kyle + native `.kyx`):
```
ky-http-1.0.0/
├── ky.toml
├── src/
│   ├── lib.ky               # Public Kyle API (wraps FFI calls)
│   └── internal/
│       └── ffi.ky           # extern fn declarations + @link directives
├── native/
│   ├── ky-http.kyx          # Pre-compiled native library (per platform)
│   └── ky-http.kyx.sha256   # Integrity checksum
├── tests/
│   └── test_http.ky
├── README.md
└── LICENSE
```

### 6.3 Package Naming Convention

- **Registry name:** `ky-<category>` (e.g., `ky-http`, `ky-sqlite`, `ky-json`)
- **Import path:** `from ky-http import ...` → resolved via package manager
- **Rust crate name** (for native code): `kyx_<name>` (e.g., `kyx_http`)
- **`.kyx` filename:** `<name>-<arch>-<os>.kyx` for multi-platform support

---

## 7. File Types: `.ky` vs `.kyx`

### `.ky` — Kyle Source Code

The primary file type. Human-readable, platform-independent.

```ky
# src/lib.ky
from ky-platform import fs

pub fn read_config(path: str) -> str:
    if fs.exists(path):
        fs.read(path)
    else:
        ""
```

### `.kyx` — Kyle Compiled Extension

A native shared library exposed as a Kyle module.  
Generated from Rust (or Kyle compiled to native) with a standard FFI ABI.

**Purpose:** Allow packages to ship pre-compiled binaries so users don't need to compile native code from source.

**Platform suffixes:**

| Platform | `.kyx` file name |
|----------|------------------|
| macOS ARM64 | `ky-http-arm64-darwin.kyx` |
| Linux ARM64 | `ky-http-arm64-linux.kyx` |
| Linux x64 | `ky-http-x64-linux.kyx` |
| Windows x64 | `ky-http-x64-windows.kyx` |
| WASM | `ky-http-wasm32.kyx` |

**Implementation (Rust side):**
```rust
// native/src/lib.rs
#[no_mangle]
pub extern "C" fn kyx_http_get(url: *const u8, len: u32, out: *mut *mut u8, out_len: *mut u32) -> i32 {
    // ... implementation ...
}
```

**Usage (Kyle side):**
```ky
@link "ky-http"
extern fn kyx_http_get(url: ptr, len: i32, out: ptr, out_len: ptr) i32

pub fn get(url: str) -> str:
    # ... wraps the extern call ...
```

---

## 8. FFI Strategy

### 8.1 What Must Be Added to the Compiler

To write packages in pure Kyle without Rust, the compiler needs three additions:

#### 8.1.1 `extern fn` — Declare External Functions

```ky
# Declare a C function that exists in a linked library
extern fn socket(domain: i32, type_: i32, protocol: i32) i32
extern fn connect(sock: i32, addr: ptr, addrlen: i32) i32
extern fn send(sock: i32, buf: ptr, len: i64, flags: i32) i64
extern fn recv(sock: i32, buf: ptr, len: i64, flags: i32) i64
```

**Compiler impact:**
- **Parser:** `extern fn name(params) ret_type:` — new declaration kind
- **Semantic:** Register as external symbol (skip body check)
- **MIR:** Generate `MirInst::Call` to the named symbol (same as runtime calls)
- **Codegen:** Already works — `self.module.add_function(name, ft, None)` — just don't compile a body

#### 8.1.2 `@link` — Linker Directives

```ky
@link "libcurl"         # Link with -lcurl
@link "libssl"          # Link with -lssl
@link "libsqlite3"      # Link with -lsqlite3
```

**Compiler impact:**
- **Parser:** `@link "libname"` — new statement at module level
- **Driver:** Pass to linker as `-l<libname>` arguments
- **No MIR/Codegen impact:** Only affects the link step

#### 8.1.3 `ptr` — Raw Pointer Operations (Complete)

`ptr` already exists as a type but needs **arithmetic + dereference**:

```ky
buf: ptr = alloc(1024)
buf[0] = 0xFF as i8       # store byte at offset
value = buf[4] as i32     # load i32 from offset
dest = buf + 16           # pointer arithmetic
```

**Compiler impact:**
- **Codegen:** Implement `load/store` with `ptr` type → `GEP` + `load`/`store` LLVM instructions
- **MIR:** Already has `PtrOffset` instruction — verify it handles `ptr` type
- **Type system:** `ptr` is a valid type; ensure `[]` operator works

### 8.2 Priority Order

| Feature | Effort | Why |
|---------|--------|-----|
| **`extern fn`** | ~1 day | Foundation for everything — without it, can't call C libraries |
| **`@link`** | ~0.5 day | Without it, extern functions exist but linker can't find them |
| **`ptr` operations** | ~1 day | Without it, can't read/write raw memory (required for C interop) |
| **Total** | **~2.5 days** | After this, ALL packages can be written in 100% Kyle |

### 8.3 How It Will Look (Example)

After these three features, a package like `ky-http` can be written entirely in Kyle:

```ky
# ky-http/src/lib.ky
@link "libcurl"

extern fn curl_easy_init() ptr
extern fn curl_easy_setopt(handle: ptr, option: i32, value: ptr) i32
extern fn curl_easy_perform(handle: ptr) i32
extern fn curl_easy_cleanup(handle: ptr)

pub fn get(url: str) -> str:
    curl = curl_easy_init()
    curl_easy_setopt(curl, CURLOPT_URL, url)
    curl_easy_perform(curl)
    # ... read response ...
    curl_easy_cleanup(curl)
    response
```

---

## 9. Implementation Phases

### Phase 0 — Compiler Foundation (NOW)
**Goal:** Add `extern fn`, `@link`, `ptr` to the compiler so packages can be 100% Kyle.

| Task | Duration |
|------|----------|
| `extern fn` declaration (parser → semantic → MIR → codegen) | 1 day |
| `@link` directive (parser → driver → linker) | 0.5 day |
| `ptr` load/store/offset (codegen) | 1 day |
| **Total** | **~2.5 days** |

### Phase 1 — Kyle Platform (backlog)
**Goal:** File system, networking, time, environment as Kyle packages with FFI.

| Package | Duration |
|---------|----------|
| `ky-platform` — FileSystem (exists, read, write, mkdir, remove, stat) | 3-4 days |
| `ky-platform` — Networking (TCP, UDP, DNS, TLS) | 5-7 days |
| `ky-platform` — Time (now, sleep, timer) | 1 day |
| `ky-platform` — Environment (args, vars, exit) | 1 day |

### Phase 2 — Backend Packages (backlog)
**Goal:** HTTP, JSON, SQLite, PostgreSQL, WebSocket, Crypto as Kyle packages.

| Package | Depends On | Duration |
|---------|-----------|----------|
| `ky-json` | Phase 0 | 1 day |
| `ky-http` | Phase 0 + Phase 1 (networking) | 3-4 days |
| `ky-sqlite` | Phase 0 + Phase 1 (fs) | 2-3 days |
| `ky-postgres` | Phase 0 + Phase 1 (net) | 3-4 days |
| `ky-websocket` | `ky-http` | 2 days |
| `ky-crypto` | Phase 0 | 2-3 days |
| `ky-testing` | (pure Kyle, no FFI) | 1 day |

### Phase 3 — Registry Server
**Goal:** Production-ready package registry server.

| Task | Duration |
|------|----------|
| Registry HTTP API (upload, download, search, auth) | 5-7 days |
| Database backend (PostgreSQL for registry metadata) | 2-3 days |
| Package namespace management, versioning, yanking | 2 days |

### Phase 4+ — Frontend (long-term)
| Layer | Duration |
|-------|----------|
| Kyle Windowing + Kyle Graphics (basic: Skia/WebGPU backend) | Months |
| Kyle Scene (layout engine, scene graph) | Months |
| Kyle UI (core widgets) | Months |
| Kyle Desktop (shell, explorer, terminal) | Months-Years |

---

## 10. Current vs Target State

| Aspect | Today (2026-07-03) | Target (After Phase 0) |
|--------|-------------------|----------------------|
| **FFI to C libraries** | ❌ Requires Rust wrapper | ✅ Direct `extern fn` in Kyle |
| **Linker control** | ❌ Hardcoded in pipeline.rs | ✅ `@link` directive in .ky source |
| **Pointer operations** | 🔶 Declared, incomplete codegen | ✅ Full ptr load/store/offset |
| **Packages with native code** | ❌ Not possible | ✅ Via `.kyx` compiled extensions |
| **Pure Kyle HTTP client** | ❌ Impossible | ✅ Possible (calls libcurl via extern) |
| **Pure Kyle SQLite** | ❌ Impossible | ✅ Possible (calls libsqlite3 via extern) |
| **Compiler performance** | = C/Rust | = C/Rust (no change) |
| **Binary size** | ~16KB (hello world) | ~16KB (no change) |

---

## Summary

The architecture is designed for **decades of evolution**.

- **Phase 0** (next 2.5 days): Add `extern fn`, `@link`, `ptr` — unlocks 100% Kyle packages
- **Phase 1-2** (weeks): Kyle Platform + backend packages
- **Phase 3** (weeks): Registry server
- **Phase 4+** (months-years): Windowing, Graphics, Scene, UI, Desktop, KyleOS

Each phase builds on the previous without invalidating it.  
The architecture remains stable regardless of how far we go.

---

*Version: 2.0 · Last updated: 2026-07-03*
