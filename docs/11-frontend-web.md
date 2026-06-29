# Frontend Web Framework Specification

- **Status:** 📅 Planned
- **Target phase:** Post-v1.0

---

## Purpose

Compile Kyle to WebAssembly for frontend web development. Provide a reactive
UI framework similar to React/Vue but with Kyle's type safety and syntax.

---

## Proposed Syntax

```kl
import web

# Define a component
component Counter:
    count: i32 = 0

    fn render() Element:
        return web.div(
            web.h1("Counter: {this.count}"),
            web.button("+", click=(fn() =>
                this.count = this.count + 1
            )),
            web.button("-", click=(fn() =>
                this.count = this.count - 1
            ))
        )

# Mount the app
web.mount(Counter(), "#app")
```

---

## Key Types

| Type | Description |
| :--- | :--- |
| `Component` | Base class for all components |
| `Element` | A virtual DOM element |
| `Event` | Browser event wrapper |

---

## Functions

| Function | Signature | Description |
| :--- | :--- | :--- |
| `web.mount` | `(component: Component, selector: str) void` | Mount app to DOM node |
| `web.div` | `(...children: [Element], attrs?: {str: T}) Element` | Create div element |
| `web.h1`–`web.h6` | `(text: str, attrs?: {str: T}) Element` | Heading elements |
| `web.button` | `(text: str, attrs?: {str: T}) Element` | Button element |
| `web.input` | `(attrs?: {str: T}) Element` | Input element |
| `web.text` | `(text: str) Element` | Text node |
| `Component.render` | `() Element` | Abstract: returns the component's UI tree |
| `Component.set_state` | `(new_state) void` | Trigger re-render |

---

## Key Concepts

- **One-way data flow**: parent passes props to children
- **Reactive updates**: `set_state()` triggers selective re-render
- **Virtual DOM**: minimal DOM mutations
- **CSS scoping**: component styles are scoped by default
- **Event handling**: inline closures with automatic cleanup
- **WASM compilation**: Kyle compiles to WASM via LLVM's wasm target

---

## Implementation Notes

- Requires WASM target support (Phase 14)
- Small runtime size target (< 100KB gzipped for minimal app)
- Interop with JavaScript via `wasm-bindgen`-style bindings
- No external JS framework dependency
