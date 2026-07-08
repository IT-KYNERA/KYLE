# Allocator

> El runtime de Kyle delega en `malloc`/`free` del sistema para la gestión de heap.
> No hay allocator personalizado — cada asignación va al allocator de Rust/libc.

## Estrategia actual

```rust
// memory.rs — simplified
 extern "C" fn ky_alloc(size: i64) -> *mut u8 {
    let layout = Layout::from_size_align(size as usize, 8).unwrap();
    std::alloc::alloc(layout)
}
```

Toda asignación de heap (str, list, dict) pasa por `ky_alloc`, que llama a
`std::alloc::alloc` de Rust (que usa `malloc`/`mmap` del SO).

## Estrategia de listas

Las listas `{T}` usan crecimiento exponencial (×2) para push:

```rust
fn grow(list: *mut KlList) {
    let new_cap = (*list).cap * 2;
    let new_data = ky_alloc(new_cap * 8) as *mut i64;
    std::ptr::copy_nonoverlapping((*list).data, new_data, (*list).len as usize);
    ky_free((*list).data as *mut u8);
    (*list).data = new_data;
    (*list).cap = new_cap;
}
```

Esto da O(1) amortizado por push.

## Estrategia de str_builder

```rust
// str_builder append
if new_len > cap {
    let new_cap = cap * 2;
    data = realloc(data, new_cap);  // redimensionar buffer
}
```

## Limitaciones actuales

| Limitación | Impacto | Futuro |
|-----------|---------|--------|
| Sin pool/arena | Fragmentación | Pool de objetos < 64 bytes |
| Sin SSO | Strings < 16 bytes van al heap | SSO en v0.7 |
| Sin thread-local cache | Contención en multithreading | TLAB |

## Ver también

- `memory.md` — API de memoria del runtime
- `stack-heap.md` — Stack vs Heap
