# Kyle — Roadmap de Desarrollo

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

### UI Framework (RFC-0005 v2)

| Fase | Estado |
|:----:|:------:|
| **UI-IR + Backend system** | ✅ Completo |
| **JS Runtimes → ESM** | ✅ Completo |
| **`ky run` + `ky new kyui`** | ✅ Completo |
| **Routing: `<router>` + `<route>` + `<layout>` + `<slot>`** | ✅ **v0.8.0 — COMPLETADO** |
| **Module resolver multi-archivo** | ✅ **COMPLETADO** |
| **`app.kyx` entry point + template kyui** | ✅ **COMPLETADO** |
| **`navigate()` / `set_title()` / `<link>` built-in** | ✅ **COMPLETADO** |
| **Web backend funcional** | ✅ **Funciona** |
| **Desktop backend (SDL2)** | 🟡 **Roto** — Sin `SDL_PollEvent`, eventos, ni ventana responsive |
| **iOS backend (SwiftUI)** | 🟡 **Roto** — Swift inválido, Package.swift mal configurado |
| **File picker nativo** | 📅 **FASE 1 — PRIORIDAD MÁXIMA** |
| **Form models + class binding** | 📅 **FASE 2 — PRIORIDAD ALTA** |
| **Config block global** | 📅 **FASE 5** |
| **WASM target** | 📅 **FASE 6** |

---

## Plan de Implementación Priorizado

### FASE 1: File Picker Nativo + File I/O (5-7 días)

**Objetivo:** Poder abrir selector de archivos nativo (imagen, PDF, cualquier archivo), leer contenido, y enviar a backend HTTP.

| Tarea | Esfuerzo |
|-------|:--------:|
| 1.1 Tipo `file_data` en IR (`name`, `size`, `mime`, `content`) | Pequeño |
| 1.2 Tag `FilePicker` en ComponentTag | Pequeño |
| 1.3 Web backend: `<input type="file">` + FileReader | Medio |
| 1.4 Verificar tipo `bytes` funcional en runtime | Medio |
| 1.5 Demo: upload de archivo a backend HTTP | Medio |

### FASE 2: Form Models + Class Binding (5-7 días)

**Objetivo:** Declarar clase Kyle como modelo de formulario con binding + validación automática.

| Tarea | Esfuerzo |
|-------|:--------:|
| 2.1 Atributo `field=` en inputs (alternativa a `bind=`) | Medio |
| 2.2 Atributo `model=` en `<form>` | Medio |
| 2.3 Web backend: binding desde modelo | Grande |
| 2.4 Validación automática desde `fn validate()` | Medio |
| 2.5 Validaciones declarativas (`@required`, `@email`, `@min`) | Grande |

### FASE 3: Arreglar Desktop Backend (3-5 días)

| Tarea | Esfuerzo |
|-------|:--------:|
| 3.1 `SDL_PollEvent` + manejar `SDL_QUIT` | Medio |
| 3.2 Arreglar `SDL_WINDOWPOS_UNDEFINED` | Pequeño |
| 3.3 `SDL_RenderFillRect` en vez de drawLine loop | Pequeño |
| 3.4 `@if`, `@for`, `@match`, `@expr` en desktop | Medio |
| 3.5 Soportar 20+ ComponentTags más | Grande |

### FASE 4: Arreglar iOS Backend (3-5 días)

| Tarea | Esfuerzo |
|-------|:--------:|
| 4.1 `.fontWeight(.bold())` → `.fontWeight(.bold)` | Pequeño |
| 4.2 Extensión `Color(hex:)` en código generado | Pequeño |
| 4.3 `Package.swift` con `.library` | Medio |
| 4.4 No ignorar `Slot`, `Match`, `Expr`, `CodeBlock` | Medio |
| 4.5 Soportar 20+ ComponentTags más | Grande |

### FASE 5: Config Block Global (2-3 días)

| Tarea | Esfuerzo |
|-------|:--------:|
| 5.1 Parsear bloque `config:` con propiedades tipadas | Medio |
| 5.2 `target(Target.web): port = 8080` | Medio |
| 5.3 Tipografía global (fuente base, tamaños) | Medio |

### FASE 6: WASM + Performance (2-4 días)

| Tarea | Esfuerzo |
|-------|:--------:|
| 6.1 Probar y arreglar build WASM | Grande |
| 6.2 Code splitting por ruta (lazy loading) | Medio |

---

## Estado de Backends UI

| Backend | Estado | Próximo paso |
|---------|:------:|-------------|
| **Web** | ✅ Funcional | Mantener + WASM |
| **Desktop (macOS)** | 🟡 Genera código con bugs | FASE 3 |
| **Desktop (Windows)** | ❌ No implementado | FASE 3 |
| **Desktop (Linux)** | ❌ No implementado | FASE 3 |
| **iOS** | 🟡 Genera Swift inválido | FASE 4 |
| **Android** | ❌ No implementado | Post-FASE 4 |

---

## Testing

```bash
# Rust tests
cargo test --workspace --exclude kyc_runtime_wasm

# Build release
cargo build --release --bin ky

# Create test project
ky new kyui /tmp/test
cd /tmp/test

# Build web
ky build web
# Serve
ky run web        # Dev server on :8080

# Desktop (WIP)
ky build desktop  # Genera main.ky + compila

# iOS (WIP)
ky build ios      # Genera ios-app/ (necesita arreglos)
```

---

*Última actualización: 2026-07-12 · Plan priorizado por fases*
