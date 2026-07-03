# Ecosystem Architecture

Kyle is not just a programming language. It is a complete software development platform.

## Layered architecture

Each layer only knows the layer immediately below it. This guarantees scalability, maintainability, and portability.

```
Applications
    │
Kyle UI (widgets)
    │
Kyle Scene (scene graph, layout)
    │
Kyle Graphics (canvas, GPU)
    │
Kyle Windowing (windows, events)
    │
Kyle Platform (FS, net, threads, audio)
    │
Kyle Runtime (memory, strings, collections)
    │
Kyle Language (compiler)
```

## Language layer

The compiler transforms `.ky` source code into native binaries. It has no knowledge of UI, windows, GPUs, or operating systems. Its only responsibility is generating machine code.

## Runtime layer

Language services that do not depend on the operating system: memory allocation, strings, lists, dictionaries, panic handling.

## Platform layer

Every interaction with the operating system goes through Kyle Platform, which defines platform-independent APIs: file system, networking, threads, audio, sensors, etc.

## Windowing layer

Window management, keyboard and mouse events, clipboard, drag & drop. Separated from graphics because not all programs need windows.

## Graphics layer

2D/3D rendering: canvas, text, images, shadows, shaders, textures. Does not create windows or read files — only renders.

## Scene layer

Scene graph, layout engine, event dispatch, hit testing, animation scheduling. Does not render — only organizes the visual hierarchy.

## UI layer

Reusable components: buttons, text boxes, lists, tables, dialogs, menus. Contains no platform-specific logic.

## Platform adapters

Each operating system implements Kyle Platform interfaces independently:
- Linux, macOS, Windows, Android, iOS, Web (WASM), KyleOS

## Package manager

The package manager is a horizontal service that applies to all layers equally. It does not belong to any specific layer.
