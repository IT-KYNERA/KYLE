# Panic

> Manejo de pánicos del runtime de Kyle.
> Crate: `kyc_runtime/src/panic.rs` (6 líneas).

## Responsabilidad

El sistema de panic maneja errores fatales del runtime: division by zero,
acceso fuera de bounds, null pointer dereference, etc.

## Implementación actual

```rust
pub extern "C" fn ky_panic(msg: *const u8) {
    panic!("Kyle panic: {}", unsafe { CStr::from_ptr(msg).to_str().unwrap_or("unknown") });
}
```

Cuando ocurre un panic:

1. El runtime imprime el mensaje de error
2. Muestra un backtrace si está disponible
3. Termina el proceso con código de error distinto de cero

## Errores que causan panic

| Condición | Mensaje |
|-----------|---------|
| Division by zero | `panic: division by zero` |
| Index out of bounds | `panic: index out of bounds` |
| Null pointer dereference | `panic: attempted to dereference null pointer` |
| Assertion failed | `panic: assertion failed` |
| Overflow | `panic: arithmetic overflow` |

## Integración con Result

El sistema `T!` (Result) permite manejar errores sin panic:

```ky
fn divide(a: i32, b: i32) i32!:
    if b == 0:
        return error("division by zero")
    a / b

result = divide(10, 0)
match result:
    ok(v): println(v.to_str())
    error(e): println(e)    # "division by zero"
```

## Ver también

- `03-language/error-handling/panic.md` — Sintaxis de panic/error
- `startup.md` — Inicialización del runtime
