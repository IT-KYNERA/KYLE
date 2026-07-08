# Identifiers

> Reglas para nombres de identificadores en Kyle.

## Reglas

- Deben empezar con letra o `_`
- Pueden contener letras, dígitos y `_`
- Distinguen mayúsculas/minúsculas (`foo` ≠ `Foo`)
- Longitud ilimitada
- Palabras reservadas (keywords) no pueden ser identificadores

## Válidos

```ky
foo
mi_variable
_private
contador_123
MAX_SIZE
StringBuilder
```

## Inválidos

```ky
123foo      # empieza con dígito
foo-bar     # guión no permitido
if          # keyword reservada
```

## Convenciones

| Convención | Ejemplo | Cuándo usar |
|------------|---------|-------------|
| `snake_case` | `mi_variable`, `calcular_total` | Funciones, variables, métodos |
| `UPPER_SNAKE` | `MAX_SIZE`, `PI` | Constantes (`:=`) |
| `_prefix` | `_internal`, `_cache` | Privado / interno |

## Ver también

- `keywords.md` — Palabras reservadas
- `comments.md` — Comentarios
