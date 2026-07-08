# Option

> Valores opcionales: `T?` / `option<T>`.
> Un valor que puede ser `some(val)` o `none`.

## Sintaxis sugar `T?`

```ky
nombre: str? = "Kyle"
nombre = none

if valor = nombre:        # pattern matching
    println(valor)
```

## Pattern matching

```ky
match nombre:
    some(v): println("nombre: " + v)
    none: println("sin nombre")
```

## Métodos

| Método | Descripción |
|--------|-------------|
| `is_some()` | `true` si tiene valor |
| `is_none()` | `true` si es none |
| `unwrap()` | Retorna valor o panic |
| `unwrap_or(default)` | Retorna valor o default |

```ky
nombre: str? = obtener_nombre()
if nombre.is_some():
    println(nombre.unwrap())

nombre = nombre.unwrap_or("invitado")
```

## Uso en retorno de función

```ky
fn find_user(id: i32) User?:
    if id == 0:
        return none
    User { name: "Kyle", id: id }

match find_user(1):
    some(u): println(u.name)
    none: println("no encontrado")
```

## Ver también

- `result.md` — Result (versión con error)
- `panic.md` — Errores fatales
