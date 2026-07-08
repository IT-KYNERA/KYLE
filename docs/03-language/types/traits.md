# Traits / Contracts

> Interfaces en Kyle mediante `contract`. Definen comportamiento que las clases
> pueden implementar.

## Declaración

```ky
contract Drawable:
    fn draw()
    fn get_size() (i32, i32)

contract Serializable:
    fn serialize() str
    fn deserialize(data: str)
```

## Implementación

```ky
final class Circle :: Drawable, Serializable:
    radius: i32

    fn draw():
        println("circle r=" + radius.to_str())

    fn get_size() (i32, i32):
        (radius * 2, radius * 2)

    fn serialize() str:
        "Circle(" + radius.to_str() + ")"

    fn deserialize(data: str):
        radius = 10    # simplified
```

## Herencia simple

```ky
class Animal:
    name: str

class Dog :: Animal:
    fn speak():
        println(name + " says woof")
```

## Contracts vs class inheritance

| Concepto | `contract` | `class :: Parent` |
|----------|-----------|------------------|
| Propósito | Interface | Herencia de implementación |
| Múltiples | ✅ Sí | ❌ No (simple) |
| Estado | ❌ No | ✅ Sí (fields) |
| Métodos default | ❌ No | ✅ Sí |

## Ver también

- `generics.md` — Genéricos con constraints
- `structs.md` — Clases y structs
