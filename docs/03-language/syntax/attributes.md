# Attributes

> Metadatos para funciones, clases y módulos mediante `#[...]`.

## Atributos disponibles

| Atributo | Destino | Descripción |
|----------|---------|-------------|
| `#[test]` | `fn` | Marcar función como test |
| `#[bench]` | `fn` | Marcar función como benchmark |

## #[test]

```ky
#[test]
fn test_addition():
    assert.eq(2 + 2, 4)

#[test]
fn test_string():
    assert.eq("hello", "hello")
```

Ejecutar:

```bash
ky test
```

## #[bench]

```ky
#[bench]
fn bench_fib():
    fib(1000000)
```

## Atributos en módulos (import)

```ky
from json import json
from http import client
```

## Ver también

- `04-standard-library/testing.md` — Testing con aserciones
- `macros.md` — Macros (futuro)
