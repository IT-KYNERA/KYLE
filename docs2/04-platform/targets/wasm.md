# Wasm Target

**Status:** Planned

Kyle can compile to WebAssembly via LLVM's WASM target.

## Build

```bash
ky build --target wasm32-unknown-unknown app.ky
```

## Limitations

- WASM has no file system, networking, or threads
- Only pure computation and DOM manipulation via JS glue code
- Full browser API bindings are planned for the `ky-web` package
