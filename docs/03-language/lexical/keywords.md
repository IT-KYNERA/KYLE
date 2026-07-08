# Keywords

> Palabras reservadas de Kyle. No pueden usarse como identificadores.

```
fn          class       final       abstract    enum
contract    type        if          elif        else
while       for         in          match       return
break       continue    defer       guard       unsafe
async       await       static      true        false
none        error       ok          and         or
not         is          as          this        import
from        pub
```

## Notas

| Keyword | Uso |
|---------|-----|
| `none` | Valor nulo para `T?` |
| `error` | Retornar error en `T!` |
| `ok` | Retornar éxito en `T!` |
| `this` | Referencia al objeto actual (en métodos) |
| `is` | Type check (`x is Type`) |
| `as` | Cast explícito (`x as T`) |
| `static` | Método estático (`static fn name`) |
| `pub` | Visibilidad pública |

## No son keywords

| Palabra | Por qué |
|---------|---------|
| `const` | No existe. Usar `NAME := value` para constantes |
| `let` / `var` | No existen. Usar `nombre = valor` |
| `mut` | No existe. Usar `^T` para mutable |
| `self` | No existe. Usar `this.field` para campos |
| `interface` | No existe. Usar `contract` |

## Ver también

- `identifiers.md` — Reglas de identificadores
- `literals.md` — Literales del lenguaje
