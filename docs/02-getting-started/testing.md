# Testing

> Kyle tiene un framework de testing integrado con el atributo `#[test]`.

## Escribir tests

```ky
from testing import assert

#[test]
fn test_addition():
    result: i32 = 2 + 2
    assert.eq(result, 4)

#[test]
fn test_string():
    assert.eq("hello", "hello")
    assert.ne("hello", "world")
    assert.str_eq("hello", "hello")
```

## Ejecutar tests

```bash
ky test
```

Busca funciones `#[test]` en `tests/`, las compila y ejecuta.

## Aserciones

| Función | Descripción |
|---------|-------------|
| `assert.is_true(cond)` | Afirmar condición verdadera |
| `assert.eq(a, b)` | Afirmar a == b |
| `assert.ne(a, b)` | Afirmar a != b |
| `assert.str_eq(a, b)` | Afirmar strings iguales |

## Estructura del proyecto

```
my-project/
├── ky.toml
├── src/
│   └── main.ky
└── tests/
    ├── test_unit.ky
    └── test_integration.ky
```

## Requisitos

Cada función de test debe:
- No tomar parámetros
- Retornar `i32` (0 = pasa, !=0 = falla)
- Usar funciones `assert` para validación

## Ejemplo completo

```ky
from testing import assert

fn sum_list(lst: {i32}) i32:
    result: ^i32 = 0
    for val in lst:
        result = result + val
    result

#[test]
fn test_sum_list():
    assert.eq(sum_list({1, 2, 3}), 6)

#[test]
fn test_sum_empty():
    assert.eq(sum_list({}), 0)
```

## Ver también

- `04-standard-library/testing.md` — Documentación completa de testing
- `project-layout.md` — Estructura de proyectos
