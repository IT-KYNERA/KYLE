# Lifetimes

> Kyle **no tiene lifetimes explícitos** como Rust. Los borrows se validan
> mediante el borrow checker en tiempo de compilación, pero sin anotaciones
> de lifetime en el código fuente.

## Cómo funciona

En Rust, las referencias llevan anotaciones de lifetime:

```rust
fn get_first<'a>(s: &'a str) -> &'a str { &s[0..1] }
```

En Kyle, no hay `'a`:

```ky
fn get_first(s: &str) &str:    # ❌ PROHIBIDO
    s.substr(0, 1)
```

Kyle **prohíbe** devolver referencias. Solo se pueden devolver valores owned:

```ky
fn get_first(s: &str) str:     # ✅ OWNED, correcto
    s.substr(0, 1)
```

## Borrows como parámetros

Los borrows solo pueden ser parámetros entrantes, nunca valores de retorno:

```ky
fn read(s: &str):              # ✅ borrow como parámetro
    println(s)

fn read_and_return(s: &str) &str:   # ❌ prohibido
    s
```

## Limitación

Esta limitación simplifica el lenguaje pero impide ciertos patrones como
iterators que devuelven referencias a datos internos. Para esos casos,
usa `.clone()` o rediseña la API.

## Ver también

- `ownership.md` — Reglas de ownership
- `borrow-analysis.md` — Implementación del borrow checker
