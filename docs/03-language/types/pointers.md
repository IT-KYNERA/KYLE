# Pointers

> Punteros raw en Kyle: `ptr` para FFI y operaciones de bajo nivel.

## `ptr` type

`ptr` es un puntero raw de 8 bytes (64 bits). Es un tipo **Copy**.

```ky
p: ptr = 0 as ptr       # null pointer
p = some_variable as ptr  # convertir dirección
```

## Operaciones

```ky
extern fn ky_ptr_read_i32(ptr) i32
extern fn ky_ptr_write_i32(ptr, i32)
extern fn ky_ptr_read_ptr(ptr) ptr

buf: ptr = ky_alloc(1024)
ky_ptr_write_i32(buf, 42)
val: i32 = ky_ptr_read_i32(buf)
println(val.to_str())    # 42
ky_free(buf)
```

## Aritmética de punteros

```ky
buf: ptr = ky_alloc(100)
ptr: ptr = buf + 8       # offset en bytes
ky_ptr_write_i32(ptr, 99)
ky_free(buf)
```

## Usos comunes

| Uso | Ejemplo |
|-----|---------|
| FFI C | `buf: ptr = malloc(1024)` |
| Memory-mapped I/O | `reg: ptr = 0xFF00_0000 as ptr` |
| Buffer pool | `ky_alloc` / `ky_free` |
| Type erasure | `fn_ptr = my_function as ptr` |

## Seguridad

`ptr` es **inseguro** por naturaleza:
- No hay bounds checking
- No hay null checking
- No hay lifetime tracking
- El usuario es responsable de la gestión

```ky
unsafe:
    buf: ptr = ky_alloc(100)
    ky_ptr_write_i32(buf, 42)
    ky_free(buf)
```

## Ver también

- `ffi/abi.md` — ABI y calling convention
- `ffi/c.md` — Llamar funciones C
