# Memory Management

> Gestión de memoria en el runtime de Kyle: asignación, liberación, retain/release.
> Crate: `kyc_runtime/src/memory.rs` (67 líneas).

## Responsabilidad

El runtime de Kyle proporciona gestión manual de memoria para strings, listas, diccionarios
y otros tipos heap-allocados. No hay garbage collector — la memoria se libera explícitamente
mediante el borrow analysis del compilador, que inserta llamadas a `ky_free` automáticamente.

## Funciones

### ky_alloc

```rust
#[unsafe(no_mangle)]
pub extern "C" fn ky_alloc(size: i64) -> *mut u8
```

Asigna un bloque de memoria del heap del tamaño especificado.

- Retorna un puntero al bloque asignado, o null si falla
- La memoria incluye un header con refcount y tamaño
- Equivalente a `malloc()` de C

```ky
extern fn ky_alloc(size: i64) ptr
buf = ky_alloc(1024)     # asigna 1024 bytes
```

### ky_free

```rust
#[unsafe(no_mangle)]
pub extern "C" fn ky_free(ptr: *mut u8)
```

Libera un bloque de memoria previamente asignado con `ky_alloc`.

- No hace nada si `ptr` es null
- No verifica doble-free (undefined behavior)
- Equivalente a `free()` de C

```ky
extern fn ky_free(ptr)
ky_free(buf)              # libera memoria
```

### ky_retain / ky_release

```rust
#[unsafe(no_mangle)]
pub extern "C" fn ky_retain(ptr: *mut u8)
#[unsafe(no_mangle)]
pub extern "C" fn ky_release(ptr: *mut u8)
```

Sistema de reference counting para memoria compartida.

- `ky_retain`: incrementa el contador de referencias atómico
- `ky_release`: decrementa y libera si llega a 0
- Usa `AtomicI64` para el contador

### Header

Cada bloque de memoria tiene un header con:

```rust
struct Header {
    strong: AtomicI64,   // reference count
    size: i64,            // tamaño del bloque
}
```

## Integración con borrow analysis

El compilador (`borrow_analysis.rs`) determina automáticamente cuándo insertar `ky_free`:

```rust
// Generado por el compilador al final del scope
ky_free(s_ptr)        # str sale de scope → liberar
ky_list_free(lst)     # lista sale de scope → liberar (llama ky_free internamente)
ky_dict_free(d)       # diccionario sale de scope → liberar
```

Esto elimina la necesidad de `free()` manual o garbage collector.

## Estrategia de asignación

- `ky_alloc` usa `Layout::from_size_align` de Rust (que usa `malloc`/`free` del sistema)
- Strings, listas y diccionarios usan `ky_alloc` internamente
- El runtime NO tiene su propio pool/arena — delega en el asignador del sistema

## Ver también

- `allocator.md` — Detalles del asignador de memoria
- `06-compiler/borrow-analysis.md` — Cómo el compilador inserta `ky_free`
