# Kyle — Roadmap de Desarrollo

> Documento completo: [`docs/01-introduction/roadmap.md`](docs/01-introduction/roadmap.md)
> UI Framework: [`docs/10-design/rfc/0005-ui-rearchitecture-plan.md`](docs/10-design/rfc/0005-ui-rearchitecture-plan.md)

---

## Estado Actual

| Área | Estado |
|------|--------|
| **Compilador** (Fases 1-17) | ✅ Completo |
| **Runtime** (memoria, strings, colecciones) | ✅ Completo |
| **Borrow checker** | ✅ Completo |
| **Cross-platform** (macOS, Linux, Windows) | ✅ Completo |
| **Tooling** (LSP, formatter, VS Code, package manager) | ✅ Completo |
| **Paquetes** (http, json, sqlite) | ✅ Completo |
| **FFI** (extern fn, @link, ptr) | ✅ Completo |
| **Tipos especializados** (46 tipos) | ✅ Todos implementados |

### UI Framework — Nueva Arquitectura (RFC-0005)

| Fase | Estado |
|:----:|:------:|
| **FASE A: UI-IR + Backend system** | ✅ Completo |
| **FASE B: JS Runtimes → ESM** | ✅ Completo |
| **FASE C: `ky run` unificado + app auto-gen** | ✅ Completo |
| **FASE D: Desktop Skia** | ⬜ Pendiente |
| **FASE E: Android / iOS** | ⬜ Pendiente |
| **FASE F: Terminal / TUI** | ⬜ Pendiente |

---

## Arquitectura UI-IR

```
.ky / .kyx
     │
     ▼
┌──────────────────┐
│  .kyx Parser     │  → produce UI-IR (agnóstico)
└──────┬───────────┘
       ▼
┌──────────────────┐
│  UI-IR           │  ← UiNode, ComponentTag, UiProgram
│  (ir.rs)         │
└──────┬───────────┘
       │
  ┌────┴──────────────────────┐
  │                           │
  ▼                           ▼
Web Backend            Desktop Backend   ← FUTURO
(JS ESM + WASM)        (Skia + FFI)
  │                           │
  ├── Android Backend  ← FUTURO
  ├── iOS Backend      ← FUTURO
  └── Terminal Backend ← FUTURO
```

### Capas implementadas

| Capa | Archivo | Descripción |
|------|---------|-------------|
| UI-IR | `crates/kyc_ui/src/ir.rs` | UiNode, ComponentTag (30+ tags), UiProgram |
| Backend trait | `crates/kyc_ui/src/backend/mod.rs` | UiBackend trait + registry |
| Web Backend | `crates/kyc_ui/src/backend/web.rs` | UI-IR → JS (ESM) + HTML auto-gen |
| Parser | `crates/kyc_ui/src/parser.rs` | .kyx → KyxFile → UI-IR |
| App Config | `crates/kyc_ui/src/app_config.rs` | Config parser (port, title, target settings) |
| CLI | `crates/kyc_cli/src/main.rs` | `ky run` = dev server, `ky serve` deprecated |

---

## Próximos Pasos

### FASE D: Desktop Nativo (Skia) — Sem 7-10

| Tarea | Esfuerzo |
|-------|:--------:|
| FFI Skia: extern fn para canvas 2D | Grande |
| Backend desktop: UI-IR → Kyle AST | Grande |
| Ventana nativa (GLFW via FFI) | Medio |
| Layout engine (flexbox en Kyle) | Grande |
| Componentes Skia: View, Text, Button | Grande |
| `ky run --target desktop` | Pequeño |

### FASE E: Mobile (Android + iOS) — Sem 11-16

| Tarea | Esfuerzo |
|-------|:--------:|
| Backend Android: UI-IR → XML layouts | Grande |
| Backend iOS: UI-IR → SwiftUI | Grande |
| `ky run --target android` | Pequeño |

### FASE F: Terminal / TUI — Sem 17-18

| Tarea | Esfuerzo |
|-------|:--------:|
| Backend terminal: UI-IR → NCurses | Grande |
| `ky run --target terminal` | Pequeño |

---

## Bugs Conocidos

| Bug | Estado |
|-----|:------:|
| `kyc_runtime_wasm` panic_impl lang item | ⬜ Pre-existente (no afecta build) |
| Generic methods: `Box<T>.get()` monomorphization | ⬜ Pre-existente |
| Fn pointer calls: `apply(&fn, arg)` silent crash | ⬜ Pre-existente |

---

## Benchmarks (Apple M5/macOS, Jul 2026)

| Benchmark | C | C++ | Rust | C# | Java | Go | Python | **Kyle** |
|-----------|:--:|:---:|:----:|:--:|:----:|:--:|:------:|:--------:|
| Prime Sieve (3M) | 9ms | 9ms | 9ms | 26ms | 31ms | 9ms | 196ms | **23ms** |
| Fibonacci (500M) | 116ms | 117ms | 121ms | 251ms | 137ms | 118ms | TO | **235ms** |
| String Concat (500k) | 8ms | 10ms | 3ms | 22ms | 35ms | 4ms | 22ms | **65ms** |
| MatMul (100x100x10) | 7ms | 7ms | 8ms | 33ms | 38ms | 14ms | 1171ms | **9ms** |

---

## Testing

```bash
# Rust tests (excluye kyc_runtime_wasm que requiere wasm32)
cargo test --workspace --exclude kyc_runtime_wasm

# Build release
cargo build --release --bin ky

# Kyle type-check
ky check <file.ky>

# UI build
ky run   # sirve proyecto UI en localhost:8080
```

---

*Última actualización: 2026-07-11 · Arquitectura multi-target (RFC-0005)*
