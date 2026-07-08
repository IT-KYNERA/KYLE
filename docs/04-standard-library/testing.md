# testing — Testing y Aserciones

> Módulo de testing y aserciones.
> Import: `from testing import assert`

## assert: aserciones

```ky
from testing import assert

fn test_addition():
    result = 2 + 2
    assert.eq(result, 4)

fn test_string():
    assert.eq("hello", "hello")
    assert.ne("hello", "world")
    assert.str_eq("hello", "hello")

fn test_condition():
    assert.is_true(1 == 1)
```

### Funciones

| Función | Descripción |
|---------|-------------|
| `assert.is_true(cond)` | Afirmar que condición es `true` |
| `assert.eq(a, b)` | Afirmar que a == b |
| `assert.ne(a, b)` | Afirmar que a != b |
| `assert.str_eq(a, b)` | Afirmar strings iguales |
| `assert.gt(a, b)` | Afirmar a > b |
| `assert.lt(a, b)` | Afirmar a < b |
| `assert.gte(a, b)` | Afirmar a >= b |
| `assert.lte(a, b)` | Afirmar a <= b |

## Atributo `#[test]`

```ky
#[test]
fn test_addition():
    assert.eq(2 + 2, 4)

#[test]
fn test_substraction():
    assert.eq(5 - 3, 2)
```

Ejecutar tests:

```bash
ky test
```

### Ejemplo completo

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
