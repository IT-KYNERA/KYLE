# Panic

> Errores fatales que terminan la ejecución del programa.
> No recuperables — el proceso se aborta.

## Cuándo ocurre un panic

| Condición | Ejemplo |
|-----------|---------|
| División por cero | `x = 10 / 0` |
| Index out of bounds | `arr[100]` cuando `len = 3` |
| Null pointer | `ky_free(null)` |
| Assertion failed | `assert.is_true(false)` |
| Overflow en debug | `i32.MAX + 1` |

## Salida

```
KL PANIC: division by zero
```

El programa termina con código de error distinto de cero. No hay stack trace
en la implementación actual.

## Evitar panics con `T!`

Usar `Result` (`T!`) para operaciones que pueden fallar:

```ky
fn divide(a: i32, b: i32) i32!:
    if b == 0:
        return error("division by zero")
    a / b

match divide(10, 0):
    ok(v): println(v.to_str())
    error(e): println("error: " + e)    # no panic
```

## unsafe

Para operaciones que pueden panic pero están verificadas manualmente:

```ky
unsafe:
    x = 10 / 0     # panic aquí si b == 0
```

## Ver también

- `option.md` — Option (valores opcionales sin error)
- `result.md` — Result (errores recuperables)
- `diagnostics.md` — Sistema de diagnóstico del compilador
