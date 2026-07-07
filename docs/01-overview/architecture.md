# Ecosystem Architecture

Kyle is not just a programming language. It is a complete software development platform.

## Layered architecture

Each layer only knows the layer immediately below it. This guarantees scalability, maintainability, and portability.

```
Applications (KYOS...)
    │  📅 Aspiracional — sin fecha
Kyle UI (widgets)
    │  📅 Aspiracional — sin fecha
Kyle Scene (scene graph, layout)
    │  📅 Aspiracional — sin fecha
Kyle Graphics (canvas, GPU)
    │  📅 Aspiracional — sin fecha
Kyle Windowing (windows, events)
    │  📅 Aspiracional — sin fecha
    │
Kyle Platform (FS, net, threads, audio)
    │  🔜 En desarrollo — FS + time implementados
Kyle Runtime (memory, strings, collections)
    │  ✅ Completo
Kyle Language (compiler)
    │  ✅ Completo
```

> **Nota:** Las capas superiores (Windowing, Graphics, Scene, UI, Applications) son **aspiracionales**.
> El foco actual es la capa **Platform** y los **packages backend** (HTTP, SQLite, Postgres, websocket, WASM).

## Language layer

The compiler transforms `.ky` source code into native binaries. It has no knowledge of UI, windows, GPUs, or operating systems. Its only responsibility is generating machine code.

## Runtime layer

Language services that do not depend on the operating system: memory allocation, strings, lists, dictionaries, panic handling.

> ⏸️ **Runtime reescritura en Kyle:** Pausada. El archivo `crates/kyc_runtime/src/string.ky` existe con 18 funciones pero no se usa. Se reactivará cuando Kyle se use en proyectos reales.

## Platform layer

Every interaction with the operating system goes through Kyle Platform, which defines platform-independent APIs: file system, networking, threads, audio, sensors, etc.

> 🔜 **Estado:** FS (file I/O) y Time implementados en `kyc_platform` crate. Networking, threads, env, audio pendientes.

## Windowing layer — 📅 Aspiracional

Window management, keyboard and mouse events, clipboard, drag & drop. Separated from graphics because not all programs need windows.

## Graphics layer — 📅 Aspiracional

2D/3D rendering: canvas, text, images, shadows, shaders, textures. Does not create windows or read files — only renders.

## Scene layer — 📅 Aspiracional

Scene graph, layout engine, event dispatch, hit testing, animation scheduling. Does not render — only organizes the visual hierarchy.

## UI layer — 📅 Aspiracional

Reusable components: buttons, text boxes, lists, tables, dialogs, menus. Contains no platform-specific logic.

## Platform adapters

Each operating system implements Kyle Platform interfaces independently:
- Linux, macOS, Windows, Android, iOS, Web (WASM), KYOS

## Package manager

The package manager is a horizontal service that applies to all layers equally. It does not belong to any specific layer.
