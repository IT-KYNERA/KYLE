# Macros

> **No implementado.** Kyle no tiene sistema de macros actualmente.

## Estado

| Tipo de macro | Status |
|---------------|--------|
| `#[attribute]` | ✅ Parcial (solo `#[test]`, `#[bench]`) |
| Macros procedurales | ❌ No implementado |
| Macros declarativas (`macro_rules!`) | ❌ No implementado |
| `compile-time` functions | ❌ No implementado |

## Alternativas actuales

Para metaprogramación, usa funciones y genéricos:

```ky
fn max<T: copy>(a: T, b: T) T:
    if a > b: a else: b

fn clamp<T: copy>(val: T, min: T, max: T) T:
    if val < min: min
    else if val > max: max
    else: val
```

## Futuro

Las macros están en el roadmap para permitir:
- DSLs embebidos
- Generación de código
- Serialización automática
- Traits derive

## Ver también

- `generics.md` — Genéricos (alternativa a macros)
- `attributes.md` — Atributos existentes
