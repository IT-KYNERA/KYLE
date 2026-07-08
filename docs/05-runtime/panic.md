# Panic

> Manejo de pánicos del runtime de Kyle.
> Crate: `kyc_runtime/src/panic.rs` (6 líneas, función: `ky_panic`).

## Responsabilidad

El sistema de panic maneja errores fatales del runtime: division by zero,
acceso fuera de bounds, null pointer dereference, etc.

## Implementación

```rust
pub fn ky_panic(message: &str) -> ! {
    eprintln!("KL PANIC: {}", message);
    std::process::abort();
}
```

- Imprime el mensaje de error en stderr
- Termina el proceso inmediatamente con `abort()` (no limpia recursos)
- No hay stack trace ni recovery — es un error fatal

## Errores que causan panic

| Condición | Mensaje |
|-----------|---------|
| Division by zero | `KL PANIC: division by zero` |
| Index out of bounds | `KL PANIC: index out of bounds` |
| Null pointer dereference | `KL PANIC: attempted to dereference null pointer` |
| Assertion failed | `KL PANIC: assertion failed` |
| Overflow | `KL PANIC: arithmetic overflow` |

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
