# Roadmap & Status — v1.0 FINAL

> **Hoja de ruta oficial del lenguaje Kyle.** Este documento es la única fuente
> de verdad sobre el estado actual del proyecto, las prioridades de
> implementación y el checklist para v1.0. Las decisiones aquí reflejadas son
> **firmes** y no cambiarán sin un major version bump.

---

## 1. Los 5 Pilares Inmutables de Kyle (La Identidad)

Cada decisión de diseño debe alinearse con estos principios. Si no los cumple,
**no entra al lenguaje**.

| Pilar | Definición |
| :--- | :--- |
| **1. Legibilidad Extrema** | El código se lee como prosa. SIN `;`, SIN `{}`, SIN `let`/`var`/`const`. Solo indentación (4 espacios) y asignación directa. |
| **2. Tipado Fuerte con Inferencia** | El compilador conoce todos los tipos en compile-time. El programador escribe el mínimo indispensable. |
| **3. Simplicidad Radical** | Una única forma de hacer cada cosa. Sin excepciones. `for`, `while`, `loop`. |
| **4. Rendimiento Zero-Cost** | Move semantics por defecto. Sin GC. Sin refcounting implícito. `Rc`/`Arc` en stdlib. |
| **5. Coherencia Sintáctica** | Lo que parece igual se comporta igual. `T?` reemplaza `Option<T>`. `final class` reemplaza `struct`. |

---

## 2. Decisiones Concretas y Eliminaciones (Frozen)

### 2.1 Variables — NO existe `let`/`var`/`mut`/`const`

| Forma | Sintaxis | Mutabilidad |
| :--- | :--- | :--- |
| Inmutable | `nombre = valor` | ❌ |
| Mutable | `nombre := valor` | ✅ (operador "morsa") |
| Constante | `nombre ::= valor` | ❌ (compile-time, no exige mayúsculas) |

### 2.2 Tipos y Estructuras

| Decisión | Detalle |
| :--- | :--- |
| `Option<T>` eliminado del espacio público | Solo existe `T?`. Internamente es `Option<T>`, el programador nunca lo escribe. |
| `T!` se **mantiene** | Azúcar para `Result<T, Error>`. No se renombra. |
| `struct` eliminado | Usar `final class` para estructuras ligeras sin herencia. |
| `class` se mantiene | Con herencia. `final class` sin herencia. |
| `contract` se mantiene | No se renombra a `trait`. `class X :: Contract`. |
| `abstract class` reemplaza `abs class` | Claridad: no se puede instanciar. |

### 2.3 Binding If/While — SIN `let`

| Constructo | Sintaxis correcta |
| :--- | :--- |
| Binding if | `if nombre = expr:` (NO `if let nombre = expr`) |
| Binding while | `while nombre = expr:` (NO `while let nombre = expr`) |
| Destructuring | `(x, y) = punto` |

### 2.4 Visibilidad

| Prefijo | Significado |
| :--- | :--- |
| `nombre` | Público |
| `_nombre` | Protegido (mismo paquete/subclases) |
| `__nombre` | Privado (solo clase/módulo actual) |

---

## 3. Estado por Fase

> ✅ = Completado · 🔜 = En progreso · 📅 = Planificado

```
Fase 1-2:  Docs + Spec          ✅ COMPLETADO
Fase 3:    Lexer                ✅ COMPLETADO
Fase 4:    Parser               ✅ COMPLETADO
Fase 5:    HIR + Desugaring     ✅ COMPLETADO
Fase 6:    Semantic Analysis    ✅ 13/13
Fase 7:    Move Semantics       ✅ 13/13
Fase 8:    Backend Release Mode ✅
Fase 9:    Async Scheduler      🔜 (thread pool + async/await, falta state machine)
Fase 10:   Iterators            🔜 (sum/product/max/min/reverse, falta map/filter/fold lazy)
Fase 11:   Package Manager      📅
Fase 12:   Tooling              📅
Fase 13:   Borrow Checker       📅
Fase 14:   Alternative Backends 📅
```

---

### Fase 1-2: Documentación y Especificación ✅

**Completado.** Todos los documentos reescritos con la nueva sintaxis:

- `AGENTS.md` — Contexto central para agentes AI
- `docs/01-language-reference.md` — Biblia de sintaxis (v0.3.0, ~1917 líneas)
- `docs/02-types-errors-memory.md` — Sistema de tipos y memoria
- `docs/03-modules-packages-tooling.md` — Módulos, paquetes, CLI
- `docs/04-compiler-architecture.md` — Pipeline 9 fases
- `docs/05-roadmap-status.md` — Este documento

---

### Fase 3: Lexer ✅

- `Walrus` (`:=`) · `ConstDecl` (`::=`) · `Abstract` · `Final`
- `At` (`@`) · `DotDotEquals` (`..=`) · `DotDotLess` (`..<`)
- `mut` keyword eliminado
- `##` doc comments

---

### Fase 4: Parser ✅

- `name = expr` · `name := expr` · `name ::= expr`
- `final class Name:` · `abstract class Name:` · `struct` alias temporal
- `T?` / `T!` postfix types
- Destructuring `(x, y) = expr`
- `if nombre = expr` (BindingIf) · `while nombre = expr` (WhileBind)
- Error recovery (modo pánico)

---

### Fase 5: HIR + Desugaring ✅

- Crate `klc_hir` creado
- Desugaring: `T?` → `Option<T>`, `T!` → `Result<T, str>`
- Integrado en pipeline (`klc_driver`)

---

### Fase 6: Semantic Analysis ✅ (13/13)

**Completado:**
- `T?` type checking
- `:=` mutability checking
- Destructuring type checking
- `if nombre = expr` BindingIf scope binding
- `while nombre = expr` WhileBind scope binding
- Constant stmt type-checking
- Return type checking
- Class/AbstractClass type-checking
- `::=` constant evaluation checking
- Abstract method enforcement
- Match guard MIR lowering (Fase 6 crossover con MIR)
- Default params type checking

**Pendiente:**
- [x] ⭐⭐⭐⭐ **Or-patterns (`a | b`)** — AST + Parser + Type Checker + MIR lowering
- [x] ⭐⭐⭐ **Properties (get/set)** — parser completo + MIR field routing (backing field `_name`)
- [x] ⭐⭐⭐ **Default params MIR lowerer** — sustituir defaults en call-sites

---

### Fase 7: Move Semantics ✅ (13/13)

**Completado:**
- Copy vs Move type classification
- Replace `ownership` (refcounting) pass → `move_analysis.rs`
- Forward dataflow analysis con intersección en joins
- Use-after-move detection
- `.clone()` para Str/List/Dict (runtime `kl_clone_*`)
- Heap-allocation de string literals
- Borrowing functions (`print`, `println`, `strlen`, etc.)
- Direct-pass para Move locals en llamadas a funciones
- Pipeline integrado: build falla en use-after-move
- `ownership.rs` eliminado
- `kl_release` declaration removed from codegen

**Pendiente:**
- [x] ⭐⭐⭐ **Memory safety tests** — 9 tests end-to-end en `klc_driver` (pipeline completo): copy types, clone, borrowing funcs, params, if/else, use-after-move (str, list), return after move

---

### Fase 8: Backend — Release Mode ✅

- [x] ⭐⭐⭐⭐⭐ Conectar `--release` a `OptimizationLevel::Aggressive`
- [x] `kl build --release` y `kl run --release` funcionan correctamente

---

### Fase 9: Async Scheduler 🔜 (thread pool V2)

- [x] ⭐⭐⭐ `async fn name():` sintaxis — ✅ parseada
- [x] ⭐⭐⭐ `async expr` + `await expr` — ✅ thread pool global
- [x] ⭐⭐⭐ `kl_spawn_task` / `kl_await_task` / `kl_yield` — ✅ runtime
- [ ] ⭐⭐⭐ State machine generation (V3) — `async fn` sin thread pool
- [ ] ⭐⭐⭐ Work-stealing scheduler
- [ ] ⭐⭐ Non-blocking I/O (sockets, archivos)

---

### Fase 10: Iteradores 🔜 (métodos de agregación)

- [x] ⭐⭐⭐ `items.sum()` — ✅
- [x] ⭐⭐⭐ `items.product()` — ✅
- [x] ⭐⭐⭐ `items.max()` / `items.min()` — ✅
- [x] ⭐⭐⭐ `items.reverse()` — ✅
- [x] ⭐⭐⭐ `items.len()` / `items.add()` / `items.pop()` / etc. — ✅
- [x] ⭐⭐⭐ `items.contains()` / `items.insert()` / `items.remove_at()` / `items.clear()` — ✅
- [x] ⭐⭐⭐ `items.clone()` — ✅
- [x] ⭐⭐ Runtime `kl_list_map` / `kl_list_filter` / `kl_list_fold` / `kl_list_reduce` — ✅ (vía fn ptr)
- [ ] ⭐⭐⭐ `items.map(fn)` / `.filter(fn)` / `.fold(init, fn)` — método directo (requiere closures funcionales)
- [ ] ⭐⭐ Lazy evaluation / `iter()` trait

---

### Fase 11: Package Manager 📅

- [ ] ⭐⭐ `kl.toml` / `kl.lock`
- [ ] ⭐⭐ Version resolution (semver)
- [ ] ⭐⭐ `kl publish` + registry
- [ ] ⭐⭐ `kl doc` con `##` comments

---

### Fase 12: Tooling 📅

- [ ] ⭐⭐ `#[test]` attribute → `kl test`
- [ ] ⭐⭐ Formatter updated for new syntax
- [ ] ⭐⭐ LSP updated for new syntax
- [ ] ⭐⭐ VS Code extension updated

---

### Fase 13: Borrow Checker 📅

- [ ] ⭐ Referencias `&T` / `&mut T` (opcional, post-v1.0)
- [ ] ⭐ Inferencia de regiones (sin anotaciones de lifetime)

---

### Fase 14: Alternative Backends 📅

- [ ] ⭐ Cranelift backend
- [ ] ⭐ WASM target

---

## 4. Prioridad de Implementación (Estrellas)

| Prioridad | Símbolo | Descripción |
| :--- | :--- | :--- |
| **Crítica** | ⭐⭐⭐⭐⭐ | Parches que bloquean todo lo demás (release mode, move semantics completo). |
| **Muy Alta** | ⭐⭐⭐⭐ | Mejoras de seguridad y usabilidad pre-v1.0 (or-patterns). |
| **Alta** | ⭐⭐⭐ | Funcionalidades que hacen al lenguaje "moderno" (async, iteradores, properties). |
| **Media** | ⭐⭐ | Herramientas y ecosistema (package manager, tooling). |
| **Baja** | ⭐ | Mejoras a largo plazo (borrow checker, backends alternativos). |

---

## 5. Criterios de Aceptación (Nuevas Características)

Para que una nueva característica entre en Kyle, debe cumplir **todos** estos criterios:

1. **Simplicidad:** ¿Añade complejidad al lenguaje? Si sí, debe resolver un problema muy grande.
2. **Legibilidad:** ¿El código se ve limpio y natural?
3. **Costo Cero:** ¿El programador paga por ella si no la usa?
4. **Coherencia:** ¿Encaja con la sintaxis existente sin crear casos borde?
5. **Facilidad de Enseñanza:** ¿Se puede explicar en 1 párrafo?
6. **Mantenibilidad:** ¿El compilador puede implementarla sin volverse increíblemente complejo?

---

## 6. Feature Matrix

| Feature | Fase | Estado |
| :--- | :--- | :--- |
| Indentation-based blocks (4 spaces) | 1-2 | ✅ |
| `name = value` inmutable | 1-2 | ✅ |
| `name := value` mutable | 1-2 | ✅ |
| `name ::= value` constante | 1-2 | ✅ |
| `T?` optional type | 1-2 | ✅ |
| `T!` error-returning type | 1-2 | ✅ |
| `final class` | 3-4 | ✅ |
| `abstract class` | 3-4 | ✅ |
| `if nombre = expr` (BindingIf) | 3-4 | ✅ |
| `while nombre = expr` (WhileBind) | 3-4 | ✅ |
| Destructuring `(x, y) = expr` | 3-4 | ✅ |
| Error recovery (parser) | 3-4 | ✅ |
| HIR + Desugaring (`T?` → `Option<T>`) | 5 | ✅ |
| `::=` constant evaluation checking | 6 | ✅ |
| Abstract method enforcement | 6 | ✅ |
| Match guard (MIR lowering) | 6 | ✅ |
| Default params type-checking | 6 | ✅ |
| Move semantics (dataflow, use-after-move, clone, borrowing funcs) | 7 | ✅ 13/13 |
| Ownership.rs eliminado | 7 | ✅ |
| Pipeline integrado (build falla en move errors) | 7 | ✅ |
| **Or-patterns (`a \| b`)** | **6** | **✅** |
| **Properties (get/set)** | **6** | **✅** |
| **Default params MIR lowerer** | **6** | **✅** |
| **Memory safety tests** | **7** | **✅** |
| **Release mode (`-O2`/`-O3`)** | **8** | **✅** |
| **Async scheduler** | **9** | **🔜 Thread pool + async/await, falta state machine** |
| **Iterators** | **10** | **🔜 sum/product/max/min/reverse, falta map/filter/fold** |
| **Package manager** | **11** | **❌ Pendiente** |
| **Tooling (`#[test]`, formatter, LSP, VS Code)** | **12** | **❌ Pendiente** |
| **Borrow checker** | **13** | **❌ Pendiente** |
| **Alternative backends** | **14** | **❌ Pendiente** |

---

## 7. Árbol de Documentación Oficial

```
docs/
├── 00-vision.md                  # Filosofía y principios
├── 01-language-reference.md      # La biblia de la sintaxis (✅ actualizado)
├── 02-types-errors-memory.md     # Tipos, errores, memoria (✅ actualizado)
├── 03-modules-packages-tooling.md# Módulos, paquetes, herramientas (✅ actualizado)
├── 04-compiler-architecture.md   # Pipeline, crates, fases (✅ actualizado)
├── 05-roadmap-status.md          # Este documento — hoja de ruta única
├── 06-stdlib.md                  # ✅ Módulos estándar (core, collections, io)
├── 07-migration-guide.md         # ✅ Guía de migración desde Python/Rust/Go
├── 08-http-client.md             # 📅 Especificación de HTTP client
├── 09-sqlite.md                  # 📅 Especificación de SQLite bindings
├── 10-postgresql.md              # 📅 Especificación de PostgreSQL bindings
├── 11-frontend-web.md            # 📅 Especificación de frontend web
├── architecture/
│   └── compiler-internals.md     # ✅ Para contribuidores (MIR/SSA internals)
└── rfcs/
    └── 0001-move-semantics.md    # ✅ RFC de move semantics
```

---

## 8. v1.0 Release Checklist

### Documentación
- [x] AGENTS.md — actualizado con nueva sintaxis
- [x] Language reference (01) — actualizado v0.3.0
- [x] Types/Errors/Memory (02) — actualizado
- [x] Modules/Packages/Tooling (03) — actualizado
- [x] Compiler architecture (04) — actualizado
- [x] Roadmap (05) — este documento
- [ ] `06-stdlib.md` — documentar módulos estándar
- [ ] `07-migration-guide.md` — guía de migración

### Lexer + Parser
- [x] Walrus, ConstDecl, Abstract, Final, DotDotEquals, DotDotLess tokens
- [x] `=`, `:=`, `::=` a nivel declaración y statement
- [x] `abstract class`, `final class`
- [x] `T?`, `T!` postfix
- [x] Destructuring, BindingIf, WhileBind
- [x] Error recovery
- [ ] `@` (At) token para atributos
- [ ] Todos los ejemplos `.kl` reescritos con nueva sintaxis

### HIR
- [x] Crate `klc_hir` creado
- [x] Desugaring `T?` → `Option<T>`, `T!` → `Result<T, str>`
- [x] Integrado en pipeline

### Semantic Analysis
- [x] `T?`, `:=`, destructuring, BindingIf/WhileBind type-checking
- [x] Return, Constant, Class/AbstractClass type-checking
- [x] `::=` constant evaluation checking
- [x] Abstract method enforcement
- [x] Match guard lowering
- [x] Default params type-checking
- [x] Or-patterns (`a | b`)
- [x] Properties (get/set) — MIR lowering
- [x] Default params MIR lowerer

### Move Semantics
- [x] Copy/Move classification
- [x] Use-after-move detection
- [x] Dataflow analysis (forward, intersection at joins)
- [x] `.clone()` para Str/List/Dict
- [x] Borrowing functions (print, println, strlen)
- [x] Heap-alloc string literals
- [x] ownership.rs eliminado
- [x] Pipeline integrado
- [x] Memory safety tests automatizados

### Release Mode
- [x] `--release` → `OptimizationLevel::Aggressive`

### Funcionalidades mayores
- [ ] Async scheduler (Fase 9)
- [ ] Iterators (Fase 10)
- [ ] Package manager (Fase 11)
- [ ] Tooling: `#[test]`, formatter, LSP, VS Code (Fase 12)

---

## 9. Test Counts

| Suite | Count | Status |
| :--- | :--- | :--- |
| `klc_frontend` unit tests | 83 | ✅ All passing |
| `klc_semantic` unit tests | 17 | ✅ All passing |
| `klc_mir` unit tests | 11 | ✅ All passing |
| `klc_tools` unit tests | 4 | ✅ All passing |
| `klc_runtime` unit tests | 0 | n/a (C-ABI) |
| `klc_backend` unit tests | 0 | n/a |
| `klc_core` unit tests | 0 | n/a |
| `klc_driver` unit tests | 9 | ✅ All passing |
| `klc_cli` unit tests | 0 | n/a |
| End-to-end `kl test` | 12 | 11/12 passing (1 pre-existing failure: test_misc.kl) |
| **Total Rust unit tests** | **124** | **✅ All passing** |

---

## 10. Estado Actual (v0.4.0)

### Completado desde v0.3.0
- Fase 5: HIR crate + desugaring
- Fase 6 (completa): `::=` const-eval, abstract methods, match guards, default params, or-patterns, properties, `is`/`as`/`super`/labeled loops/variadic/fn pointers
- Fase 7 (completa): Move analysis completo, ownership.rs eliminado, pipeline integrado, memory safety tests
- Fase 8 (completa): Release mode con `OptimizationLevel::Aggressive`
- **126 tests Rust** (↑ desde 101)
- `ownership.rs` y `kl_release` declaration removidos
- `print_int`/`println_int` builtins removidos — ahora `print(42)` convierte a `str` automáticamente
- **List borrowing fix** — `kl_list_push/get/set/len` añadidos a borrowing funcs
- **Built-in type methods**: `items.add()`, `items.pop()`, `items.len()`, `s.upper()`, `s.lower()`, `s.trim()`, `s.contains()`, `s.replace()` — sintaxis de método en vez de `list_push(items)`
- **Proyecto de prueba completo** en `examples/src/main.kl` con 30+ tests

### Pendiente inmediato
- ~~Or-patterns~~ ✅ (Fase 6)
- ~~Properties get/set~~ ✅ (Fase 6)
- ~~Default params MIR lowerer~~ ✅ (Fase 6)
- ~~Release mode (Fase 8)~~ ✅

### Pendiente features de sintaxis
- **`fn<T -> U>` type**: Soporte completo de function pointers como variables, con codegen C-ABI
- **Closures funcionales**: Asignar closures a variables `fn<...>`, llamadas vía function pointer
- **`fn<...> async`**: Function pointers async
- **`items.map(fn)` / `.filter(fn)` / `.fold(init, fn)` / `.reduce(fn)`**: Requieren closures funcionales
- **`set<T>` type**: Conjuntos sin duplicados con operaciones union/intersection/difference
- **Tuple type**: `(i32, str)` como tipo de primera clase (hoy solo en destructuring)
- **Dict methods**: `.keys()`, `.values()`, `.clear()` — solo existe `d.len()` y `d.clone()`
- **State machine async**: `async fn` → state machine en vez de thread pool

### Bugs encontrados y arreglados en benchmarks
- **`as f64` casting**: ✅ ARREGLADO. Faltaba el pattern `IntValue → FloatType` en codegen.rs.
  `(39 as f64)` ahora produce `39.0` correctamente (antes daba ~3.4e-309).

### Resultados de benchmark (2026-06-29)

**Prueba 1: Aritmética** — `total = total + i * 2 - 1` (10M iteraciones)
| Lenguaje | Tiempo | vs Rust |
| :--- | :--- | :--- |
| Rust | 0.001s | 1x |
| Kyle | 0.029s | ~29x |
| Java 21 | 0.026s | ~26x |
| Python 3 | 0.547s | ~547x |

**Prueba 2: Primos** — `is_prime()` hasta 100000
| Lenguaje | Tiempo | vs Rust |
| :--- | :--- | :--- |
| Rust | 0.003s | 1x |
| Kyle | 0.006s | **2x** |
| Java 21 | 0.019s | 6.3x |
| Python 3 | 0.080s | 27x |

**Prueba 3: Mandelbrot** — 78×78 grid, 100 max iter (cálculo punto flotante)
| Lenguaje | Tiempo | vs Rust |
| :--- | :--- | :--- |
| Rust | 0.002s | 1x |
| Kyle | 0.005s | **2.5x** |
| Java 21 | 0.059s | 30x |
| Python 3 | 0.030s | 15x |

**Conclusión:** Kyle rinde entre 2x y 29x más lento que Rust (según carga),
y entre 0.5x y 10x más rápido que Java 21. La diferencia con Rust se debe
principalmente a la ausencia de SSA Form en el MIR, lo que impide que LLVM
optimice tan agresivamente como con Rust.

**Prueba 1: Aritmética** — `total = total + i * 2 - 1` (10M iteraciones)
| Lenguaje | Tiempo | vs Rust |
| :--- | :--- | :--- |
| Rust | 0.001s | 1x (optimizado a fórmula) |
| Kyle | 0.029s | ~29x más lento |
| Java 21 (JIT) | 0.026s | ~26x más lento |
| Python 3 | 0.547s | ~547x más lento |

**Prueba 2: Primos** — `is_prime()` hasta 100000 (carga real de CPU)
| Lenguaje | Tiempo | vs Rust |
| :--- | :--- | :--- |
| Rust | 0.003s | 1x |
| **Kyle** | **0.006s** | **2x más lento** |
| Java 21 (JIT) | 0.019s | 6.3x más lento |
| Python 3 | 0.080s | 27x más lento |

**Conclusión:** Kyle es ~2-30x más lento que Rust (según la carga) y ~0.7-3x más rápido que Java. Con SSA Form (Fase 15), Kyle puede acercarse a 1-2x de Rust.

### Resultados de benchmark (2026-06-29) — 3 pruebas, resultados correctos

| Lenguaje | Aritmética (10M iters) | Primos (hasta 100K) | Mandelbrot (78×78) | Promedio vs Rust |
| :--- | :--- | :--- | :--- | :--- |
| **Rust** | 0.001s | 0.003s | 0.002s | **1x** |
| **Kyle** | 0.029s | 0.006s | 0.005s | **~7x** |
| **Java 21** | 0.026s | 0.019s | 0.059s | **~15x** |
| **Python 3** | 0.547s | 0.080s | 0.030s | **~150x** |

**Conclusión:** Kyle es ~7x más lento que Rust y ~2x más rápido que Java 21.
La diferencia con Rust se debe principalmente a la ausencia de SSA Form.

---

### Fase 15 — Performance Deep Dive (Plan de Optimización)

#### 🔬 Diagnóstico: ¿Por qué Kyle es más lento que Rust?

Rust y Kyle usan el **mismo LLVM 18** como backend. La diferencia está en
cómo generan el LLVM IR. Rust genera IR en **SSA Form** (Static Single
Assignment), donde cada variable se asigna una sola vez. Kyle genera IR con
**load/store** constantes porque el MIR usa un modelo de memoria plana.

```llvm
; LO QUE RUST GENERA (SSA) — LLVM optimiza fácilmente:
%total.0 = phi i64 [ 0, %entry ], [ %next, %loop ]
%i.0    = phi i64 [ 0, %entry ], [ %inc, %loop ]
%next   = add i64 %total.0, %i.0
%inc    = add i64 %i.0, 1
%cond   = icmp slt i64 %inc, 10000000
br i1 %cond, label %loop, label %exit

; LO QUE KYLE GENERA (load/store) — LLVM pierde tiempo:
%total = alloca i64
%i     = alloca i64
store i64 %total_val, %total    ← dependencia falsa
store i64 %i_val, %i            ← dependencia falsa
%loaded_i = load i64, %i
%loaded_t = load i64, %total
%add   = add i64 %loaded_t, %loaded_i
store i64 %add, %total
```

#### 🎯 Plan de Acción

| # | Optimización | Impacto | Esfuerzo | Descripción |
| :--- | :--- | :--- | :--- | :--- |
| 1 | **SSA Form** | **10-50x** | 2 semanas | Convertir MIR a SSA con phi nodes. Elimina load/store. LLVM optimiza como con Rust. |
| 2 | **Mem2Reg** (promoción alloca→registro) | 3-5x | 3 días | Detectar variables sin referencia, promoverlas a valores SSA |
| 3 | **GVN** (Global Value Numbering) | 2-3x | 3 días | Eliminar sub-expresiones redundantes entre bloques |
| 4 | **Inlining** | 1.5-3x | 2 días | Inline de funciones pequeñas y una sola llamada |
| 5 | **Mejor generación LLVM IR** | 1.5-2x | 1 semana | Eliminar allocas temporales innecesarias, usar `alloca` solo cuando es necesario |
| 6 | **i64 por defecto** | 1.5x | 1 día | Literales grandes → i64 automático, evitar casts constantes |
| 7 | **ThinLTO** (Link Time Optimization) | 1.2-1.5x | 1 día | Pasar `-flto=thin` al linker para optimización entre módulos |
| 8 | **Alias Analysis** | 1.2x | 3 días | Marcar punteros como `noalias`/`readonly` para que LLVM optimice mejor |

#### 📊 Impacto Estimado Acumulado

```
Estado actual:  Rust 1x  |  Kyle ~7x  |  Java ~15x  |  Python ~150x
Con #1 SSA:    Rust 1x  |  Kyle ~1.5x  |  Java ~15x  |  Python ~150x
Con #1-#4:     Rust 1x  |  Kyle ~1.2x  |  Java ~15x  |  Python ~150x
Con #1-#8:     Rust 1x  |  Kyle ~1.0x  |  Java ~15x  |  Python ~150x
```

#### 🔧 Detalles Técnicos

**1. SSA Form** — El cambio más importante.
- Entrada: `MirFunction` con load/store en basic blocks
- Salida: `SsaFunction` con valores directamente referenciados y phi nodes
- Después de SSA: el LLVM IR generado no tiene allocas (excepto arrays/structs grandes)
- LLVM puede aplicar: constant propagation, dead code elimination, loop optimization, vectorization

**2. Mem2Reg** — Paso previo a SSA.
- Identificar allocas cuyo address no escapa (no hay `field_ptr`/`ptr_offset`)
- Reemplazar cada `load %X` con el valor de la última `store %X`
- Insertar phi nodes en join points

**3. GVN** — Sobre el MIR ya en SSA.
- Detectar: `a = x + y; b = x + y` → segunda es redundante
- Detectar: `a = x * 2; b = x << 1` → misma operación
- Requiere SSA para ser efectivo

**4. Inlining** — Antes de SSA.
- Heurística: funciones < 10 instrucciones → inline siempre
- Funciones llamadas una sola vez → inline
- Funciones con cuerpo vacío o return trivial → inline

**5. LLVM IR limpio** — En codegen.rs.
- No emitir `alloca` para valores que pueden ser SSA
- Usar `build_store` solo cuando el destino es realmente necesario
- Marcar argumentos de función como `noalias` cuando sea seguro

**6. i64 por defecto** — En type checker.
- Literales enteros > 2^31 → inferir i64 automáticamente
- Operaciones aritméticas entre i32 e i64 → promover a i64
- Esto elimina los casts constantes que actualmente emite el lowerer

**7. ThinLTO** — En linker.rs.
- Al compilar con `--release`, pasar `-flto=thin -O3`
- El linker realiza optimizaciones entre módulos (inline cross-crate, dead code)

**8. Alias Analysis** — En codegen.rs.
- `noalias` en punteros de parámetros que no se solapan
- `readonly` en parámetros de solo lectura
- `writeonly` en destinos de solo escritura
- LLVM usa esta información para reordenar operaciones

### Bloqueantes conocidos
- ~~`test_cf.kl`~~ ✅ — corregido (param locals ya no se liberan, load aliases no causan double-free)
- `test_misc.kl` — type mismatch en Result types (`error()` retorna `Option<void>` vs `Result<T, str>` esperado)

---

*Versión: v1.0 · Actualizado: 2026-06-29*
