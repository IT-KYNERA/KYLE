# Reflection

> Información de tipos en tiempo de ejecución.
> Actualmente soporte mínimo: `type()` retorna información básica.

## type()

```ky
t: type_info = (42).type()
println(t.name)    # "i32"
println(t.kind)    # "primitive"
println(t.size)    # 4 (bytes)
```

```ky
t = "hello".type()
println(t.name)    # "str"
println(t.size)    # 8 (bytes del puntero)
```

## type_info

```ky
class type_info:
    name: str
    kind: str        # "primitive", "struct", "enum", "array", "list"
    size: i32        # bytes
```

## Usos

| Propósito | Ejemplo |
|-----------|---------|
| Debug | `println(val.type().name)` |
| Serialización | `json.serialize(val)` usa type internamente |
| Genéricos | El compilador infiere tipos en tiempo de compilación |

## Limitaciones

- No hay `instanceof` o type checking dinámico
- No hay cast entre tipos no relacionados
- El type system es completamente estático (excepto `type()`)

## Ver también

- `primitive-types.md` — Todos los tipos disponibles
- `generics.md` — Polimorfismo en tiempo de compilación
