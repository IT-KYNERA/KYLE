# core — Tipos Fundamentales

> Módulo base con tipos `option` (`T?`) y `result` (`T!`).
> Import: `from core import option, result`

## option: `option<T>` / `T?`

Representa un valor opcional: `some(val: T)` o `none`.

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

| Método | Firma | Descripción |
|--------|-------|-------------|
| `is_some` | `fn(self) bool` | `true` si tiene valor |
| `is_none` | `fn(self) bool` | `true` si es none |
| `unwrap` | `fn(self) T` | Retorna valor o panic |
| `unwrap_or` | `fn(self, default: T) T` | Valor o default |

```ky
name: str? = get_user_name()
if name.is_some():
    println(name.unwrap())
```

## result: `result<T, E>` / `T!`

Representa una operación que puede fallar: `ok(val: T)` o `error(msg: E)`.

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

res: i32! = divide(10, 2)
match res:
    ok(v): println(v.to_str())
    error(e): println("error: " + e)
```

### Métodos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `is_ok` | `fn(self) bool` | `true` si es ok |
| `is_error` | `fn(self) bool` | `true` si es error |
| `unwrap` | `fn(self) T` | Retorna valor o panic |
| `unwrap_or` | `fn(self, default: T) T` | Valor o default |

## Ver también

- `03-language/error-handling/option.md`
- `03-language/error-handling/result.md`
