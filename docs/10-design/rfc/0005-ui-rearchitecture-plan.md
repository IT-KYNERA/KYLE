# RFC-0005: Re-arquitectura UI Framework — Multiplataforma desde el núcleo

**Status:** Implementado (FASIS A-C)
**Date:** 2026-07-11
**Documentación relacionada:**
- [0002-ui-architecture.md](0002-ui-architecture.md) — Arquitectura original
- [0003-ui-translation.md](0003-ui-translation.md) — Traducción multi-target
- [0004-ui-implementation-roadmap.md](0004-ui-implementation-roadmap.md) — Roadmap anterior

---

## 0. Resumen Ejecutivo

El framework UI se ha refactorizado para tener una **capa intermedia agnóstica (UI-IR)** que permite
a `.kyx` compilar a cualquier plataforma. El `index.html` ya no está hardcodeado. 
`ky serve` se unificó en `ky run`. Los JS runtimes ahora usan ES Modules.

---

## 1. Arquitectura Implementada

```
.ky / .kyx (código fuente)
     │
     ▼
┌──────────────────────────────┐
│      .kyx Parser             │  ← XML → AST Kyle → UI-IR
│   (kyc_ui/parser.rs)         │
└──────────┬───────────────────┘
           ▼
┌──────────────────────────────┐
│   UI-IR (Intermediate Repr)  │  ← Agnóstico, tipado (ir.rs)
│   UiNode, ComponentTag       │
└──────────┬───────────────────┘
           │
     ┌─────┴─────────────────────┐
     │                           │
     ▼                           ▼
┌──────────────────┐    ┌──────────────────┐
│  Web Backend     │    │ Desktop Backend  │  ← FUTURO
│  UI-IR → JS ESM │    │ UI-IR → Skia     │
│  + HTML auto     │    │ + FFI nativo     │
└──────────────────┘    └──────────────────┘
     │
     ├── Android Backend ← FUTURO
     ├── iOS Backend ← FUTURO
     └── Terminal Backend ← FUTURO
```

### 1.1 UI-IR (UI Intermediate Representation)

Definido en `crates/kyc_ui/src/ir.rs`:

```rust
pub struct UiProgram {
    pub view_paths: Vec<String>,
    pub code_blocks: Vec<String>,
    pub styles: Vec<StyleDecl>,
    pub animations: Vec<AnimDecl>,
    pub body: Vec<UiNode>,
}

pub enum UiNode {
    Element { tag: ComponentTag, attrs: Vec<UiAttr>, children: Vec<UiNode> },
    SelfClosing { tag: ComponentTag, attrs: Vec<UiAttr> },
    Text(String), Slot { name, fallback },
    If { condition, then_branch, else_branch },
    For { item, list, body },
    Match { expr, cases },
    Expr(String), CodeBlock(String),
}
```

### 1.2 ComponentTag

Tags conocidos (30+) que cada backend traduce a su equivalente nativo:

| Tag | Web (DOM) | Desktop (Skia) | Android | iOS |
|-----|:---------:|:--------------:|:-------:|:---:|
| view | div | SkContainer | View | UIView |
| text | span | SkText | TextView | UILabel |
| button | button | SkButton | Button | UIButton |
| column | div flex-col | FlexColumn | LinearLayout V | VStack |
| row | div flex-row | FlexRow | LinearLayout H | HStack |
| ... | ... | ... | ... | ... |

### 1.3 Sistema de Backends

Trait `UiBackend` en `crates/kyc_ui/src/backend/mod.rs`:

```rust
pub trait UiBackend {
    fn name(&self) -> &str;
    fn target_triple(&self) -> &str;
    fn generate(&self, program: &UiProgram) -> BackendOutput;
}
```

Backends registrados: `web` (wasm32), `desktop` (futuro), `android` (futuro).

---

## 2. Cambios Realizados (FASIS A-C)

### FASE A: UI-IR + Backend System

| Archivo | Acción |
|---------|--------|
| `crates/kyc_ui/src/ir.rs` | **NUEVO** — UiNode, ComponentTag, UiProgram |
| `crates/kyc_ui/src/backend/mod.rs` | **NUEVO** — trait UiBackend + registry |
| `crates/kyc_ui/src/backend/web.rs` | **NUEVO** — web backend (UI-IR → JS ESM) |
| `crates/kyc_ui/src/parser.rs` | **MODIFICADO** — to_ui_program() conversión |
| `crates/kyc_driver/src/pipeline.rs` | **MODIFICADO** — usa backend system |

### FASE B: ES Modules en JS Runtimes

| Archivo | Cambio |
|---------|--------|
| `runtimes/js/reactivity.js` | `require` → `export class/function` |
| `runtimes/js/router.js` | `module.exports` → `export` |
| `runtimes/js/a11y.js` | ESM exports |
| `runtimes/js/portal.js` | ESM exports |
| `runtimes/js/error_boundary.js` | ESM exports |
| `runtimes/js/i18n.js` | ESM exports |
| `runtimes/js/glue.js` | ESM exports |
| `runtimes/js/ssr.js` | ESM exports |
| `runtimes/js/testing.js` | ESM exports |

### FASE C: CLI Unificado + app_config + HTML auto-gen

| Archivo | Cambio |
|---------|--------|
| `crates/kyc_ui/src/app_config.rs` | **NUEVO** — parser de configuración |
| `crates/kyc_cli/src/main.rs` | `ky run` = dev server, `ky serve` deprecated |
| `ROADMAP.md` | Actualizado con nueva arquitectura |
| `AGENTS.md` | Actualizado con FASIS UI |

---

## 3. Comandos Actualizados

```bash
ky new mi-app              # Crea proyecto UI (main.kyx + lib.ky)
cd mi-app
ky run                     # Compila y sirve en localhost:8080
ky run --port 9090         # Puerto custom
ky serve                   # Deprecado, delega a ky run
ky run app.ky              # Compila y ejecuta nativo
```

---

## 4. Plan de Implementación Futura

### FASE D: Desktop Nativo (Skia)

| Tarea | Esfuerzo |
|-------|:--------:|
| FFI Skia: extern fn para canvas 2D | Grande |
| Backend desktop: UI-IR → Kyle AST | Grande |
| Ventana nativa (GLFW via FFI) | Medio |
| Layout engine (flexbox en Kyle) | Grande |
| Componentes Skia: View, Text, Button | Grande |
| `ky run --target desktop` | Pequeño |

### FASE E: Mobile (Android + iOS)

| Tarea | Esfuerzo |
|-------|:--------:|
| Backend Android: UI-IR → XML layouts + Kotlin | Grande |
| Backend iOS: UI-IR → SwiftUI | Grande |
| `ky run --target android` | Pequeño |

### FASE F: Terminal / TUI

| Tarea | Esfuerzo |
|-------|:--------:|
| Backend terminal: UI-IR → NCurses/ratatui | Grande |
| `ky run --target terminal` | Pequeño |

---

## 5. Glosario

| Término | Significado |
|---------|-------------|
| UI-IR | UI Intermediate Representation — AST agnóstico de plataforma |
| Backend | Traductor de UI-IR a código de plataforma específica |
| ComponentTag | Enum de tags conocidos (View, Text, Button, etc.) |
| ESM | ES Modules — sistema de módulos nativo de JS |
| UiBackend | Trait que deben implementar todos los backends |
