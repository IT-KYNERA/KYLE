# Operator Precedence

> Tabla de precedencia de operadores en Kyle.
> De mayor a menor precedencia.

| Nivel | Operadores | Descripción |
|-------|-----------|-------------|
| 17 | `()` `[]` `.` `::` | Llamada, índice, acceso, herencia |
| 16 | `as` `is` `not` | Cast, type check, negación |
| 15 | `^` `&` `*` `-` (unario) | Sigilos y negación unaria |
| 14 | `*` `/` `%` | Multiplicación, división, módulo |
| 13 | `+` `-` | Suma, resta |
| 12 | `<<` `>>` | Shift bitwise |
| 11 | `&` `\|` `^` (bitwise) | Bitwise AND, OR, XOR |
| 10 | `..` `..=` | Range (inclusive/exclusive) |
| 9 | `==` `!=` `<` `>` `<=` `>=` | Comparación |
| 8 | `and` | AND lógico |
| 7 | `or` | OR lógico |
| 6 | `??` | Null-coalescing |
| 5 | `..` (range) | Range en for |
| 4 | `=` `+=` `-=` etc. | Asignación |
| 3 | `->` | Arrow (return type) |
| 2 | `:` | Type annotation |
| 1 | `:=` | Constante |

## Notas

- Los operadores del mismo nivel se evalúan de izquierda a derecha
- Usar paréntesis para desambiguar: `(a + b) * c`
- `not` tiene mayor precedencia que `and`/`or`
- `??` es asociativo a la derecha: `a ?? b ?? c` = `a ?? (b ?? c)`

## Ver también

- `03-language/lexical/operators.md` — Lista completa de operadores
- `03-language/syntax/expressions.md` — Expresiones del lenguaje
