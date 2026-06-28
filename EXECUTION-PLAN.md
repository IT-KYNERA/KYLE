# Kyle Evolution — Execution Master Plan

> **Hoja de ruta de implementación** para transformar Kyle en un lenguaje de
> sistemas de clase mundial. Cada fase produce un estado funcional y testeable.
> No se pasa a la siguiente fase hasta que la anterior esté completa y todos
> los tests pasen.

---

## Principios de Ejecución

1. **Una fase a la vez.** No empezar una fase hasta que la anterior esté
   completa y todos los tests pasen.
2. **Tests primero.** Cada cambio de sintaxis debe tener un test que lo
   verifique antes de implementarlo (TDD).
3. **Compilador siempre funcional.** `cargo build --workspace` nunca debe
   estar roto.
4. **Backward compatibility solo intra-fase.** Entre fases puede haber
   ruptura sintáctica.

---

## Fase 1: Documentación y Especificación (✅ COMPLETADA)

**Duración estimada:** 1–2 días · **Real:** ~1 día

Actualizar todos los documentos para reflejar las nuevas decisiones de diseño.

### 1.1 docs/01-language-reference.md
- [x] Cambiar sistema de variables: `=` inmutable, `:=` mutable, `::=` constante
- [x] Eliminar `mut` keyword de toda la sintaxis
- [x] Añadir `T?` como syntax de Option (eliminar `Option<T>` público)
- [x] Unificar `struct` → `final class` (deprecar `struct`, después eliminar)
- [x] Renombrar `abs class` → `abstract class`
- [x] Actualizar todos los ejemplos con la nueva sintaxis
- [x] Añadir destructuring: `(x, y) = punto`
- [x] Añadir match or-patterns (completos)
- [x] Añadir default params `fn f(x = 5):`
- [x] Añadir propiedades get/set
- [x] Añadir `super` keyword
- [x] Añadir labeled loops
- [x] Añadir `ptr` type, `null` literal, `as` casting
- [x] Añadir `is` operator section
- [x] Añadir function pointer type `(T) -> U`
- [x] Añadir `if let` / `while let`
- [x] Añadir doc comments (`##`)
- [x] Añadir attributes (`#[test]`, `#[inline]`)
- [x] Añadir `exit()`, `eprintln()`, `panic()`, `dbg()` builtins
- [x] Añadir `sizeof()`, `alignof()`, `offset_of()`, `transmute()`
- [x] Añadir operator overloading section
- [x] Añadir integer overflow behavior
- [x] Añadir move vs copy semantics
- [x] Corregir checkboxes inconsistentes (§3.3, §3.4, §3.5)
- [x] Corregir todos los ejemplos que usan `=>` en match → `:`
- [x] Corregir ejemplo de `pair<T,U>` que usa tuplas (no implementadas)
- [x] Añadir sección de `no_std` mode
- [x] Añadir sección de conditional compilation

### 1.2 docs/02-types-errors-memory.md
- [x] Reflejar T? en lugar de Option<T>
- [x] Documentar move semantics (nuevo modelo de memoria)
- [x] Documentar Copy vs Move types
- [x] Actualizar ejemplos que usan `=>` en match → `:`
- [x] Documentar borrow checker (opcional, futuro)
- [x] Eliminar referencias a refcounting como default
- [x] Añadir sección de `Rc`/`Arc` en stdlib

### 1.3 docs/03-modules-packages-tooling.md
- [x] Actualizar ejemplos de sintaxis (no había ejemplos que actualizar)
- [ ] Añadir `kl doc`, `kl publish`, `#[test]` (documentado en 01-language-reference.md)
- [ ] Añadir nested module paths (documentado en 01-language-reference.md)
- [ ] Añadir package registry (plan futuro)

### 1.4 docs/04-compiler-architecture.md
- [x] Nuevo pipeline: Lexer → Parser → HIR → Semantic → MIR → Move Analysis → SSA → Codegen
- [x] Nuevos crates propuestos
- [x] Remove refcounting references

### 1.5 docs/05-roadmap-status.md
- [x] Reflejar nuevo plan de fases
- [x] Actualizar feature matrix con nuevas decisiones
- [x] V1.0 checklist actualizado

---

## Fase 2: AGENTS.md + README.md (✅ COMPLETADA)

**Duración estimada:** 0.5 días · **Real:** ~0.5 días

- [x] AGENTS.md: actualizar con nuevo sistema de variables, T?, move semantics
- [x] AGENTS.md: nuevo pipeline, nuevos crates
- [x] README.md: ejemplos de código con nueva sintaxis
- [x] README.md: features actualizados

---

## Fase 3: Lexer (✅ COMPLETADA)

**Duración estimada:** 2–3 días · **Real:** ~1 día

### 3.1 Tokenización nueva
- [x] Añadir token `Walrus` (`:=`) — "morsa" (walrus operator)
- [x] Añadir token `ConstDecl` (`::=`)
- [x] Eliminar token/keyword `mut`

### 3.2 Tokenización modificada
- [x] Añadir token `Question` para `T?` type syntax (postfix `?`) — ya existía
- [x] Añadir token `At` (`@`) para `#[attr]`
- [x] Añadir `##` para doc comments (Kyle usa `##` no `///`, al estilo Python)
- [x] Añadir token `DotDotEquals` (`..=`) para rangos inclusivos
- [x] Añadir token `DotDotLess` (`..<`) para rangos semi-abiertos

### 3.3 Modificaciones existentes
- [x] `abs` keyword → `abstract` keyword (mantener ambas como alias)
- [x] `struct` keyword → `final` keyword (deprecación gradual)

---

## Fase 4: Parser (🔜 EN PROGRESO)

**Duración estimada:** 4–5 días

### 4.1 Nuevas sintaxis
- [x] `name := expr` — parsing de variable mutable
- [x] `name ::= expr` — parsing de constante
- [ ] `T?` postfix en tipos — parsing de opcional
- [x] `final class Name:` — clase sin herencia
- [x] `abstract class Name:` — clase abstracta
- [ ] `::= expr` en constantes (global scope) — ya cubierto por parse_decl
- [ ] Destructuring: `(x, y) = expr`
- [ ] `if let pattern = expr:` — if-let
- [ ] `while let pattern = expr:` — while-let

### 4.2 Modificaciones existentes
- [x] Eliminar parsing de `mut` keyword en variables
- [x] `abs class` → `abstract class` (o alias)
- [x] `struct` → `final class` (o alias y deprecation warning)

### 4.3 Error recovery
- [ ] Implementar modo pánico en parser (no parar en primer error)
- [ ] Reportar múltiples errores

---

## Fase 5: HIR — High-Level IR (⭐⭐ NUEVO CRATE)

**Duración estimada:** 3–4 días

- [ ] Crear crate `klc_hir`
- [ ] Definir tipos HIR (después de desugaring)
- [ ] Implementar desugaring:
  - `T?` → `Option<T>` interno
  - `:=` → `mut` flag interno
- [ ] Pasar HIR a semantic analysis

---

## Fase 6: Semantic (⭐ MUY ALTA)

**Duración estimada:** 3–4 días

- [ ] `T?` type checking
- [ ] `:=` mutability checking (reemplazar `mut`)
- [ ] `::=` constant evaluation checking
- [ ] Destructuring type checking
- [ ] `if let` / `while let` type checking
- [ ] Abstract method enforcement
- [ ] Or-patterns y guard en match (terminar implementación)
- [ ] Default params
- [ ] Properties (get/set)

---

## Fase 7: Move Semantics (⭐⭐⭐⭐⭐ CRÍTICO)

**Duración estimada:** 2–3 semanas

- [ ] Definir tipos Copy vs Move
- [ ] Implementar análisis de flujo para detección de "uso después de mover"
- [ ] Reemplazar pase `ownership` (refcounting) con move analysis
- [ ] Implementar `.clone()` para tipos Move
- [ ] Mantener refcount solo para `Rc`/`Arc` en stdlib
- [ ] Tests de garantía de memoria

---

## Fase 8: Backend — Release mode (⭐⭐⭐ ALTA)

**Duración estimada:** 1 día

- [ ] Conectar `--release` a `OptimizationLevel::Aggressive`
- [ ] Verificar que `-O2` / `-O3` funciona

---

## Fase 9: Async Scheduler (⭐⭐⭐ ALTA)

**Duración estimada:** 2–3 semanas

- [ ] `async fn name():` syntax
- [ ] State machine generation
- [ ] Work-stealing scheduler (tokio-style)
- [ ] Mantener `async expr` como tarea en scheduler

---

## Fase 10: Iteradores (⭐⭐⭐ ALTA)

**Duración estimada:** 1–2 semanas

- [ ] Trait `Iterable<T>` en stdlib
- [ ] `iter()` en listas, dicts, rangos, strings
- [ ] `map`, `filter`, `fold`, `reduce`, `collect`
- [ ] Lazy evaluation

---

## Fase 11: Package Manager (⭐⭐)

**Duración estimada:** 1–2 semanas

- [ ] `kl.package` / `kl.lock`
- [ ] Version resolution (semver)
- [ ] `kl publish` + registry
- [ ] `kl doc` with `##` comments

---

## Fase 12: Tooling (⭐⭐)

**Duración estimada:** 1 semana

- [ ] `#[test]` attribute → `kl test`
- [ ] Formatter updated for new syntax
- [ ] LSP updated for new syntax
- [ ] VS Code extension updated

---

## Fase 13: Borrow Checker (⭐ Baja)

**Duración estimada:** 2–3 semanas

- [ ] `&T` y `&mut T` references
- [ ] Region inference (no lifetime annotations)
- [ ] Compatibilidad con move semantics

---

## Fase 14: Backends Alternativos (⭐ Baja)

**Duración estimada:** 2–4 semanas

- [ ] Cranelift backend
- [ ] WASM target

---

## Resumen de Prioridades

```
Fase 1-2: Docs + Spec    ✅ COMPLETADO
Fase 3:   Lexer          ✅ COMPLETADO
Fase 4:   Parser         🔜 EN PROGRESO (4.1 items restantes)
Fase 5:   HIR            📅
Fase 6:   Semantic       📅
Fase 7:   Move Semantics 📅 (bloqueante)
Fase 8:   Release mode   📅
Fase 9:   Async          📅
Fase 10:  Iterators      📅
Fase 11:  Package Mgr    📅
Fase 12:  Tooling        📅
Fase 13:  Borrow Checker 📅
Fase 14:  Backends       📅
```

---

## Estado Actual (v0.3.0)

### Completado
- ✅ 6 documentos actualizados con nueva sintaxis
- ✅ AGENTS.md como pilar central
- ✅ Token: Walrus, ConstDecl, Abstract, Final, At, DotDotEquals, DotDotLess; Mut eliminado
- ✅ Lexer: `:=`, `::=`, `..=`, `..<`, `@`, `abstract`, `final`; `mut` eliminado; `##` doc comments
- ✅ Parser: `=`, `:=`, `::=` en decl y stmt; `abstract class`, `final class`
- ✅ 102 tests Rust pasando

### Siguiente
- Parser: `T?` postfix, destructuring, if-let, while-let, error recovery

---

*Versión: 1.0 · Actualizado: 2026-06-28*
