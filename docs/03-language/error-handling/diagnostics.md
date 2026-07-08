# Diagnostics

> Sistema de errores y diagnósticos del compilador Kyle.
> Reporta errores con código, mensaje y ubicación precisa.

## Formato de error

```
KL-E0001: Type mismatch

  expected 'i32', found 'str'

  --> archivo.ky:10:5
   |
10 |     x = "hello" + 42
   |         ^^^^^^^^^^^^^ expected i32 here
   |
```

## Códigos de error

| Código | Significado |
|--------|-------------|
| `KL-E0001` | Type mismatch |
| `KL-E0002` | Syntax error |
| `KL-E0003` | Undefined type |
| `KL-E0004` | Cannot modify immutable |
| `KL-E0005` | Undefined function |
| `KL-E0006` | Wrong number of arguments |
| `KL-E0007` | Cannot modify constant |
| `KL-E0008` | Invalid assignment target |
| `KL-E0009` | Undefined symbol |
| `KL-E0010` | Wrong number of type arguments |
| `KL-E0011` | Ambiguous type |
| `KL-E0012` | Return type mismatch |
| `KL-E0013` | Move analysis error |

## Warnings

| Warning | Condición |
|---------|-----------|
| Variable no usada | Declarada pero nunca leída |
| Import no usado | Módulo importado no utilizado |
| Código muerto | Código después de `return`/`break` |
| Conversión implícita | Sin `as` cast explícito |

## Move analysis errors

El borrow checker produce errores específicos:

```
KL-E0013: use-after-move: cannot move `s` (local #2) — value has been moved
KL-E0013: cannot mutably borrow `s` — it is already borrowed immutably
```

## Ver también

- `06-compiler/diagnostics.md` — Implementación del sistema de diagnósticos
- `06-compiler/borrow-analysis.md` — Errores del borrow checker
