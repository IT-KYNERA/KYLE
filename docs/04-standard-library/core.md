# core — Tipos Fundamentales

> Módulo base con tipos `option` (`T?`) y `result` (`T!`).
> Import: `from core import option, result`

## option: `option<T>` / `T?`

Representa un valor opcional: `some(val)` o `none`.

```ky
from core import option

name: option<str> = option.some("Kyle")
name = option.none

match name:
    option.some(v): println(v)
    option.none: println("no name")
```

### Sintaxis sugar `T?`

```ky
name: str? = "Kyle"
name = none

match name:
    some(v): println(v)
    none: println("no name")
```

### Métodos

| Método | Descripción | Ejemplo |
|--------|-------------|---------|
| `is_some()` | `true` si tiene valor | `opt.is_some()` |
| `is_none()` | `true` si es none | `opt.is_none()` |
| `unwrap()` | Retorna valor o panic | `opt.unwrap()` |
| `unwrap_or(default)` | Valor o default | `opt.unwrap_or("")` |

```ky
name = get_user_name()  # str?
if name.is_some():
    println(name.unwrap())
```

## result: `result<T, E>` / `T!`

Representa una operación que puede fallar: `ok(val)` o `error(msg)`.

```ky
from core import result

fn divide(a: i32, b: i32) result<i32, str>:
    if b == 0:
        return result.error("division by zero")
    result.ok(a / b)
```

### Sintaxis sugar `T!`

```ky
fn divide(a: i32, b: i32) i32!:
    if b == 0:
        return error("division by zero")
    a / b

result = divide(10, 2)
match result:
    ok(v): println(v.to_str())
    error(e): println("error: " + e)
```

### Métodos

| Método | Descripción |
|--------|-------------|
| `is_ok()` | `true` si es ok |
| `is_error()` | `true` si es error |
| `unwrap()` | Retorna valor o panic |
| `unwrap_or(default)` | Valor o default |

## Ver también

- `03-language/error-handling/option.md`
- `03-language/error-handling/result.md`
