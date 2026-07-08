# Allocator

> El runtime de Kyle delega la asignación de memoria en el allocator de Rust/libc.
> No hay un allocator personalizado — se usa `malloc`/`free` del sistema.

## Estrategia actual

Kyle usa `std::alloc::alloc` / `std::alloc::dealloc` de Rust (que a su vez usa `malloc`/`free` de libc o el allocator del sistema operativo).

```rust
// memory.rs
 unsafe extern "C" fn ky_alloc(size: i64) -> *mut u8 {
    let layout = Layout::from_size_align(size as usize, 8).unwrap();
    std::alloc::alloc(layout)
}

 unsafe extern "C" fn ky_free(ptr: *mut u8) {
    if ptr.is_null() { return; }
    let layout = Layout::from_size_align(0, 8).unwrap();
    std::alloc::dealloc(ptr, layout)
}
```

## Limitaciones

- No hay pool/arena allocator
- Strings pequeños (< 16 bytes) no tienen small string optimization (SSO)
- Cada `ky_concat` hace una nueva allocación + copia
- No hay asignador regional (arena) para operaciones temporales

## Futuro

| Mejora | Status | Impacto |
|--------|--------|---------|
| Pequeño pool de objetos (< 64 bytes) | 📅 Planeado | Reduce fragmentación |
| SSO (Small String Optimization) | 📅 Planeado | Strings < 15 bytes inline |
| Arena allocator para requests HTTP | 📅 Planeado | Liberación por región |
| Thread-local allocation cache | 📅 Planeado | Reduce contención |

## Ver también

- `memory.md` — API de memoria del runtime
- `03-language/memory/allocator.md` — Diseño conceptual del asignador
