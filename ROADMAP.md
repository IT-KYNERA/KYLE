# Kyle — Roadmap de Desarrollo

> Documento completo: [`docs/01-introduction/roadmap.md`](docs/01-introduction/roadmap.md)
> UI Framework: [`docs/10-design/rfc/0004-ui-implementation-roadmap.md`](docs/10-design/rfc/0004-ui-implementation-roadmap.md)

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
| **Tipos especializados** (22 tipos) | ✅ Todos implementados |
| **--target flag** | ⬜ CLI + codegen + linker |
| **.kyx parser** | ⬜ No implementado |
| **Traductor JS** | ⬜ No implementado |
| **UI componentes** | ⬜ No implementado |
| **Desktop nativo** (Skia) | ⬜ No implementado |
| **Mobile** (Android/iOS) | ⬜ No implementado |

---

## Próximos Pasos (UI Framework)

```
FASE 0 ─── CLI --target + WASM ───────────────── Sem 1-2
  │
FASE 1 ─── Parser .kyx + Traductor JS ────────── Sem 3-4
  │
  ├── FASE 2 ─── Estilos tipados ─────────────── Sem 5
  │
  ├── FASE 3 ─── Eventos + Binding ───────────── Sem 6
  │
  ├── FASE 4 ─── Componentes UI ──────────────── Sem 7-8
  │
  ├── FASE 5 ─── Routing ─────────────────────── Sem 9
  │
  ├── FASE 6 ─── Animaciones ─────────────────── Sem 10
  │
  └── FASE 7 ─── a11y ────────────────────────── Sem 11-12
```

### FASE 0: CLI --target + WASM (Semana 1-2)

| Tarea | Archivos |
|-------|----------|
| Flag `--target` en CLI | `crates/kyc_cli/src/main.rs` |
| Target triple en Codegen | `crates/kyc_backend/src/codegen.rs` |
| Target triple en Pipeline | `crates/kyc_driver/src/pipeline.rs` |
| Linker por target triple | `crates/kyc_backend/src/linker.rs` |
| Compilar runtime a WASM | `crates/kyc_runtime/` |
| JS glue runtime básico | `runtimes/js/glue.js` |

### FASE 1: Parser .kyx + Traductor JS (Semana 3-4)

| Tarea | Archivos |
|-------|----------|
| Nueva crate `kyc_ui` | `crates/kyc_ui/src/lib.rs` |
| Parser XML .kyx → AstType | `crates/kyc_ui/src/parser.rs` |
| Resolver .kyx | `crates/kyc_frontend/src/resolver.rs` |
| JS generator | `crates/kyc_ui/src/js_gen.rs` |

### FASE 2-7: Ver plan detallado

→ [`docs/10-design/rfc/0004-ui-implementation-roadmap.md`](docs/10-design/rfc/0004-ui-implementation-roadmap.md)

---

## Documentos de Diseño UI

| Documento | Contenido |
|-----------|-----------|
| [`ui-syntax.md`](docs/03-language/syntax/ui-syntax.md) | Sintaxis .kyx (componentes, eventos, slots) |
| [`style-system.md`](docs/03-language/ui/style-system.md) | Sistema de estilos tipado (sin CSS) |
| [`state-events.md`](docs/03-language/ui/state-events.md) | Estado, eventos, binding, formularios |
| [`animation.md`](docs/03-language/ui/animation.md) | Animaciones y transiciones |
| [`routing.md`](docs/03-language/ui/routing.md) | Routing auto-routing y navegación |
| [`accessibility.md`](docs/03-language/ui/accessibility.md) | Accesibilidad (a11y) |
| [`i18n.md`](docs/03-language/ui/i18n.md) | Internacionalización, plurales, RTL |
| [`portals.md`](docs/03-language/ui/portals.md) | Portales/teleport para modals, tooltips |
| [`error-boundaries.md`](docs/03-language/ui/error-boundaries.md) | Captura de errores y fallback UI |
| [`composition.md`](docs/03-language/ui/composition.md) | Patrones de composición (slots, HOCs) |
| [`context-patterns.md`](docs/03-language/ui/context-patterns.md) | Context avanzado y provider/consumer |
| [`ssr.md`](docs/03-language/ui/ssr.md) | Server-Side Rendering e hidratación |
| [`testing.md`](docs/03-language/ui/testing.md) | Testing de UI: unit, interacción, E2E |
| [`RFC-0002`](docs/10-design/rfc/0002-ui-architecture.md) | Arquitectura general |
| [`RFC-0003`](docs/10-design/rfc/0003-ui-translation.md) | Traducción multi-target |
| [`RFC-0004`](docs/10-design/rfc/0004-ui-implementation-roadmap.md) | Roadmap de implementación |

---

## Testing

```bash
# Rust tests
cargo test --workspace

# Kyle type-check
ky check <file.ky>

# UI build (cuando exista)
ky build --target wasm32 app.kyx
```

---

*Última actualización: 2026-07-10*
