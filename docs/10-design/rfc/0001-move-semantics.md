# RFC-0001: Move Semantics

**Status:** Implemented (v0.5 → v0.6 actualizado)  
**Date:** 2026-06-01  
**Última actualización:** v0.6 — move por defecto, `^` = mutable, `&` = borrow

## Resumen original (v0.5)

El diseño original de Kyle usaba **borrow por defecto** y `^T` para move explícito.

```ky
fn read(s: str):          # borrow (default)   ← v0.5
fn append(s: &str):       # mutable borrow     ← v0.5
fn consume(^s: str):      # move (explicit)    ← v0.5
```

## Cambio en v0.6

A partir de v0.6, Kyle usa **move por defecto**. El cambio fue:

| Concepto | v0.5 (original) | v0.6 (actual) |
|----------|-----------------|----------------|
| Default param | Borrow | **Move** |
| Mutable | `&T` | **`^T`** |
| Borrow | — | **`&T`** |
| Mutable borrow | — | **`^&T`** |
| Move | `^T` (explícito) | Default |

### Sintaxis actual (v0.6)

```ky
fn read(s: &str):          # borrow
fn append(s: ^&str):       # mutable borrow
fn consume(s: str):        # move (default)
```

## Copy types (sin cambios)

Simple types (i32, i64, f32, f64, bool) son Copy — nunca se mueven, siempre se copian.
Complex types (str, list, dict) son Move — `y = x` transfiere ownership.

## Motivación del cambio

La decisión de mover de borrow-by-default a move-by-default se tomó porque:

1. **Seguridad**: `y = x` con borrow implícito crea dangling pointers si `x` se modifica después
2. **Consistencia con Rust**: move por defecto es el estándar en lenguajes de sistemas
3. **Predictibilidad**: siempre es obvio cuándo se transfiere ownership vs cuándo se presta

## Ver también

- `03-language/memory/move.md` — Move semantics actual
- `03-language/memory/ownership.md` — Reglas de ownership completas
- `03-language/memory/copy.md` — Copy types
