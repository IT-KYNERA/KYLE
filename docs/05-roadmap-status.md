# Roadmap & Status — v1.0 FINAL

> **Hoja de ruta oficial del lenguaje Kyle.** Este documento es la única fuente
> de verdad sobre el estado actual del proyecto, las prioridades de
> implementación y el checklist para v1.0. Las decisiones aquí reflejadas son
> **firmes** y no cambiarán sin un major version bump.
>
> **Objetivo de rendimiento:** Kyle debe competir con C, C++ y Rust en velocidad
> de ejecución. Hoy está a ~3-106× de Rust (según carga) debido a que el LLVM IR
> generado carece de atributos de optimización críticos. Las fases 15 y 16
> cierran esta brecha.

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
Fase 9:    Async Scheduler      ✅ Thread pool V2 (async expr, await expr, async fn)
                                 🔜 State machine V3, work-stealing, non-blocking I/O
Fase 10:   Iterators            ✅ 17 métodos de agregación listos (runtime + lowering)
                                 🔜 Closures funcionales (fn ptr primera clase) + Lazy iterators
Fase 15:   SSA Form              ✅ COMPLETADO (100%)
                                   ✅ i64 default literal type
                                   ✅ ThinLTO -flto (GCC LTO)
                                   ✅ Alias Analysis (readonly/noalias)
                                   ✅ SSA Form (SsaFunction + Mem2Reg + Phi)
                                   ✅ Dominator fix (intersect infinite loop)
                                   ✅ GVN on SSA (Global Value Numbering)
                                   ✅ SSA Codegen: phi nodes + block_vals directos
                                   ✅ Pipeline SSA activo en release mode (⚠️ hang)
                                   ✅ Benchmarks correctos (debug): ssa_test, ssa_loop2, arithmetic
                                   ⚠️ Issue conocido: release mode hang (15.B1)
                                   ⚠️ Issue conocido: missing extern decls (15.B2)
Fase 16:   LLVM IR Quality       🔜 EN CURSO — ~8%
                                   ✅ 16.3 — readonly/readnone en runtime externs
                                   🔲 16.0 — Fix release mode hang (PRIORIDAD #0)
                                   🔲 16.2 — inbounds en GEPs (PRIORIDAD #1, podría arreglar hang)
                                   🔲 16.4 — noalias en parámetros puntero
                                   🔲 16.9 — TBAA metadata
                                   🔲 16.5 — Align explícito en loads/stores/allocas
                                   🔲 16.6 — noundef en parámetros
                                   🔲 16.7 — !range metadata en bool
                                   🔲 16.8 — lifetime.start/lifetime.end
                                   ⏳ 16.1 — nsw/nuw flags (DIFERIDO: requiere range analysis)
Fase 11:   Package Manager      🔜 DETALLADO (registry, semver, publish, lock)
Fase 12:   Tooling              🔜 DETALLADO (LSP, VS Code, tests, formatter)
Fase 13:   Sintaxis Restante    🔜 DETALLADO (genéricos, rangos, is, ptr, etc.)
Fase 14:   Borrow Checker       📅 (post-v1.0)
Fase 15:   Alternative Backends 📅 (post-v1.0)
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

### Fase 9: Async Scheduler 🔜 (thread pool V2 — funcional, faltan optimizaciones)

#### ✅ Completado (Thread Pool V2)

| Feature | Archivo | Detalle |
|---------|---------|---------|
| `async fn name():` parsing | `parser.rs` | ✅ Sintaxis `async fn foo():` parseada |
| `async expr` + `await expr` | `parser.rs`, `codegen.rs` | ✅ Evaluación async y espera de tareas |
| `kl_spawn_task(ptr, i64) -> i64` | `runtime/async_.rs` | ✅ Spawnea tarea en thread pool |
| `kl_await_task(i64) -> i64` | `runtime/async_.rs` | ✅ Bloquea hasta que la tarea termina |
| `kl_yield()` | `runtime/async_.rs` | ✅ Cooperativo: cede el thread temporalmente |
| Thread pool global (lazy_static) | `runtime/async_.rs` | ✅ Pool de threads reutilizables |

#### 🔜 Pendiente — State Machine V3

**Objetivo:** Reemplazar el thread pool por generación de state machines.
Una `async fn` se compila a una máquina de estados que:
- NO requiere heap allocation por tarea
- NO necesita hilos del sistema
- Permite cientos de miles de tareas concurrentes
- Es compatible con single-thread y multi-thread

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 9.1 | HIR-level: detectar `async fn` y generar tipo enum con variante por yield point | `klc_hir/src/` | ⭐⭐⭐ |
| 9.2 | MIR-level: transformar `await` en cambio de estado + salto a bloque correcto | `klc_mir/src/lower.rs` | ⭐⭐⭐ |
| 9.3 | Runtime: `TaskStateMachine` struct con puntero a función + estado actual | `klc_runtime/src/async_.rs` | ⭐⭐⭐ |
| 9.4 | Codegen: emitir switch-case en el bucle de polling de la state machine | `klc_backend/src/codegen.rs` | ⭐⭐⭐ |
| 9.5 | Benchmarks: 100K tareas concurrentes sin thread pool | `examples/bench/` | ⭐⭐ |

**Dependencia:** Fase 15 (SSA) — necesario para que la state machine sea eficiente.

#### 🔜 Pendiente — Work-Stealing Scheduler

**Objetivo:** Cuando hay MULTIPLES hilos, balancear carga dinámicamente.

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 9.6 | Cada thread tiene su cola de tareas local (deque) | `klc_runtime/src/async_.rs` | ⭐⭐⭐ |
| 9.7 | Thread inactivo roba tareas del deque de otro thread | `klc_runtime/src/async_.rs` | ⭐⭐⭐ |
| 9.8 | `global_queue` como respaldo para tareas nuevas | `klc_runtime/src/async_.rs` | ⭐⭐ |

**Dependencia:** State Machine V3 (9.1-9.5) — work-stealing solo tiene sentido con state machines.

#### 🔜 Pendiente — Non-blocking I/O

**Objetivo:** `kl_open()`, `kl_read_str()`, `kl_write_str()` sin bloqueo.

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 9.9 | I/O ring / epoll/kqueue integration | `klc_runtime/src/io.rs` | ⭐⭐ |
| 9.10 | `async fn read_file()` → no bloquea el thread | `klc_runtime/src/io.rs` | ⭐⭐ |
| 9.11 | Socket API async (`async fn connect()`, `async fn accept()`) | nuevo crate | ⭐⭐ |

**Dependencia:** Work-Stealing (9.6-9.8) — I/O async necesita un scheduler que no bloquee hilos.

---

### Fase 10: Iteradores 🔜 (runtime listo, falta lazy evaluation + closures)

#### ✅ Completado — Métodos de Agregación (runtime C-ABI + lowering)

| Feature | Runtime | MIR Lowering | LLVM Codegen | Estado |
|---------|---------|--------------|--------------|--------|
| `items.sum()` | `kl_list_sum` | `lower.rs` | `codegen.rs` | ✅ |
| `items.product()` | `kl_list_product` | `lower.rs` | `codegen.rs` | ✅ |
| `items.max()` | `kl_list_max` | `lower.rs` | `codegen.rs` | ✅ |
| `items.min()` | `kl_list_min` | `lower.rs` | `codegen.rs` | ✅ |
| `items.reverse()` | `kl_list_reverse` | `lower.rs` | `codegen.rs` | ✅ |
| `items.len()` | `kl_list_len` | `lower.rs` | `codegen.rs` | ✅ |
| `items.add()` | `kl_list_push` | `lower.rs` | `codegen.rs` | ✅ |
| `items.pop()` | `kl_list_pop` | `lower.rs` | `codegen.rs` | ✅ |
| `items.contains()` | `kl_list_contains` | `lower.rs` | `codegen.rs` | ✅ |
| `items.insert()` | `kl_list_insert` | `lower.rs` | `codegen.rs` | ✅ |
| `items.remove_at()` | `kl_list_remove_at` | `lower.rs` | `codegen.rs` | ✅ |
| `items.clear()` | `kl_list_clear` | `lower.rs` | `codegen.rs` | ✅ |
| `items.clone()` | `kl_clone_list` | `lower.rs` | `codegen.rs` | ✅ |
| `items.map(fn)` | `kl_list_map` | `lower.rs` | `codegen.rs` | ✅ (vía fn ptr C-ABI) |
| `items.filter(fn)` | `kl_list_filter` | `lower.rs` | `codegen.rs` | ✅ (vía fn ptr C-ABI) |
| `items.fold(init, fn)` | `kl_list_fold` | `lower.rs` | `codegen.rs` | ✅ (vía fn ptr C-ABI) |
| `items.reduce(fn)` | `kl_list_reduce` | `lower.rs` | `codegen.rs` | ✅ (vía fn ptr C-ABI) |

#### 🔜 Pendiente — Closures Funcionales (necesario para map/filter/fold inline)

**Problema:** Hoy `items.map(fn)` requiere pasar una función nombrada (`fn`), no un closure inline.
Para closures como `items.map(fn(x) x * 2)` se necesita:

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 10.1 | Parseo de closures: `fn(params) body` como expresión | `parser.rs` | ⭐⭐⭐ |
| 10.2 | Tipo `fn(T) U` como variable de primera clase con capturas | `semantic/type_checker.rs` | ⭐⭐⭐ |
| 10.3 | Codegen: closure → función anónima + environment struct en stack | `codegen.rs` | ⭐⭐⭐ |
| 10.4 | Runtime: closures con capture-by-move (env == datos capturados) | `codegen.rs` | ⭐⭐⭐ |
| 10.5 | `.map(fn)` aceptando closure como argumento (hoy solo fn nombrada) | `lower.rs` | ⭐⭐⭐ |

**Dependencia:** Fase 15 (SSA) para que closures con captures sean eficientes.

#### 🔜 Pendiente — Lazy Evaluation / `iter()` trait

**Objetivo:** `items.map(fn).filter(fn).fold(init, fn)` sin crear listas intermedias.
Hoy cada `.map()` crea una lista nueva (eager). Con lazy evaluation, solo se itera una vez.

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 10.6 | `Iter[T]` struct en runtime con next(), map(), filter(), fold() | `klc_runtime/src/list.rs` | ⭐⭐ |
| 10.7 | `items.iter()` → `Iter[T]` | `lower.rs`, `codegen.rs` | ⭐⭐ |
| 10.8 | Chain `.map().filter()` sin listas intermedias | `runtime/list.rs` | ⭐⭐ |
| 10.9 | Lazy fold/reduce sobre iterador | `runtime/list.rs` | ⭐⭐ |

---

### Fase 11: Package Manager 📅 (registry, resolución, publicación)

**Filosofía:** Tan simple como Cargo pero SIN la complejidad de workspaces multi-crate
y SIN build scripts. Un package = un proyecto. Una dependencia = una línea en `kl.toml`.

---

#### 11.1 — Manifest (`kl.toml`) completo

**Estado actual:** ✅ `kl.toml` existe con campos `[project]` y `[dependencies]` básicos.
El compilador lee `name` y `dependencies` pero no valida semver ni resuelve nada.

**Lo que falta:**

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 11.1.1 | Validación completa de campos `[project]` (name, version, authors, description, license, edition) | `klc_core/src/manifest.rs` | ⭐⭐ |
| 11.1.2 | Validación de versiones semver (major.minor.patch con pre-release opcional) | `klc_core/src/manifest.rs` | ⭐⭐⭐ |
| 11.1.3 | Campo `[project] main` para entry point distinto de `src/main.kl` | `klc_driver/src/pipeline.rs` | ⭐⭐ |
| 11.1.4 | Sección `[dev-dependencies]` para dependencias de testing | `klc_core/src/manifest.rs` | ⭐⭐ |
| 11.1.5 | Sección `[target]` para configuraciones por plataforma (opcional) | `klc_core/src/manifest.rs` | ⭐ |
| 11.1.6 | Error claro y con sugerencias si `kl.toml` falta o está mal formado | `klc_core/src/manifest.rs` | ⭐⭐ |

**Ejemplo de `kl.toml` final:**
```toml
[project]
name = "myapp"
version = "0.1.0"
authors = ["Tu Nombre"]
description = "Mi aplicación Kyle"
license = "MIT"
edition = "2024"
main = "src/main.kl"

[dependencies]
math = "1.0.0"
json = "2.1.0"

[dev-dependencies]
testing = "1.0.0"
```

---

#### 11.2 — Resolución de dependencias (semver)

**Estado actual:** ❌ No existe. `kl.toml` declara dependencias pero no se resuelven.

**Objetivo:** Dado `json = "2.1.0"` en `kl.toml`, el compilador debe:
1. Consultar el registry
2. Encontrar la versión `2.1.0` (o `^2.1.0` compatible)
3. Descargar el paquete
4. Compilarlo como módulo importable

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 11.2.1 | Parseo completo de semver: `^1.2.3`, `~1.2`, `>=1.0 <2.0`, `*`, `1.x` | `klc_core/src/semver.rs` (nuevo) | ⭐⭐⭐ |
| 11.2.2 | Comparación de versiones (major.minor.patch.pre) | `klc_core/src/semver.rs` (nuevo) | ⭐⭐⭐ |
| 11.2.3 | Algoritmo de resolución: greedy (simple, elige la mayor compatible) | `klc_core/src/resolver.rs` (nuevo) | ⭐⭐⭐ |
| 11.2.4 | Generación de `kl.lock` con versiones resueltas | `klc_core/src/manifest.rs` | ⭐⭐⭐ |
| 11.2.5 | Cache de dependencias descargadas en `~/.kl/cache/` | `klc_core/src/cache.rs` (nuevo) | ⭐⭐⭐ |
| 11.2.6 | `kl update` para actualizar `kl.lock` a últimas versiones compatibles | `klc_cli/src/main.rs` | ⭐⭐ |
| 11.2.7 | `kl outdated` para listar dependencias desactualizadas | `klc_cli/src/main.rs` | ⭐⭐ |
| 11.2.8 | Resolución de dependencias transitivas (dep de dep) | `klc_core/src/resolver.rs` | ⭐⭐⭐ |

**Algoritmo de resolución (propuesto):**
```
1. Leer kl.toml
2. Para cada dep en [dependencies]:
   a. Consultar registry por todas las versiones del paquete
   b. Filtrar por la restricción semver (ej: "2.1.0" → >=2.1.0 <3.0.0)
   c. Elegir la versión más alta que cumpla
   d. Descargar el kl.toml de esa versión para obtener sus dependencias
   e. Repetir recursivamente
3. Si hay conflictos (dos deps requieren versiones incompatibles del mismo paquete):
   - Reportar error claro con el árbol de dependencias conflictivas
4. Escribir kl.lock con todas las versiones resueltas
```

---

#### 11.3 — Registry (servidor de paquetes)

**Estado actual:** ❌ No existe.

**Objetivo:** Un registry simple basado en Git (como Cargo) pero sin la complejidad
de crates.io. Alternativa: registry plano basado en archivos JSON + tarballs.

**Opción recomendada — Registry basado en Git:**
```
# Estructura del registry:
index.git/
├── config.json           # Metadata del registry
├── pa/
│   └── package-name      # Archivo con lista de versiones
└── pa/
    └── package-name-1.0.0.crate  # Tarball del paquete (en almacenamiento)
```

**Opción alternativa — Registry HTTP+JSON:**
```
GET https://registry.kyle-lang.org/v1/packages/math
→ { "versions": [{ "version": "1.0.0", "yank": false }, ...] }

GET https://registry.kyle-lang.org/v1/packages/math/1.0.0/download
→ binary .tar.gz
```

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 11.3.1 | Diseñar estructura del registry (API HTTP REST) | docs/ (nuevo) | ⭐⭐⭐ |
| 11.3.2 | Cliente HTTP para consultar registry | `klc_core/src/registry.rs` (nuevo) | ⭐⭐⭐ |
| 11.3.3 | Descarga y extracción de paquetes (.tar.gz) | `klc_core/src/registry.rs` | ⭐⭐⭐ |
| 11.3.4 | Cache local en `~/.kl/cache/<pkg>-<ver>/` | `klc_core/src/cache.rs` | ⭐⭐⭐ |
| 11.3.5 | `kl publish` — empaquetar y subir paquete al registry | `klc_cli/src/main.rs` | ⭐⭐⭐ |
| 11.3.6 | `kl login` — autenticación con API key | `klc_cli/src/main.rs` | ⭐⭐ |
| 11.3.7 | Verificación de integridad (SHA256 checksums) | `klc_core/src/registry.rs` | ⭐⭐ |
| 11.3.8 | Yank de versiones (marcar una versión como no disponible) | registry server | ⭐⭐ |

---

#### 11.4 — Importación desde paquetes

**Estado actual:** `import math` busca archivos locales. No entiende paquetes.

**Lo que falta:**

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 11.4.1 | `import math` busca primero en paquetes instalados, luego en locales | `klc_frontend/src/parser.rs` | ⭐⭐⭐ |
| 11.4.2 | `import mypkg.str` — importar submódulo de un paquete | `klc_frontend/src/parser.rs` | ⭐⭐⭐ |
| 11.4.3 | Resolver `import json` a `~/.kl/cache/json-2.1.0/src/lib.kl` | `klc_driver/src/pipeline.rs` | ⭐⭐⭐ |
| 11.4.4 | Compilar dependencias ANTES que el proyecto principal | `klc_driver/src/pipeline.rs` | ⭐⭐⭐ |
| 11.4.5 | Cache de compilación: no recompilar dependencias si no cambiaron | `klc_driver/src/pipeline.rs` | ⭐⭐ |

**Orden de resolución de imports (actualizado):**
```
import math → busca en:
  1. Paquetes instalados en ~/.kl/cache/math-*/src/
  2. Directorio del archivo actual
  3. src/ del proyecto
  4. std/ (librería estándar)
```

---

#### 11.5 — Comandos del package manager

| Comando | Estado actual | Lo que falta |
|---------|---------------|-------------|
| `kl new <name>` | ✅ Crea proyecto template | Mejorar template con `kl.toml` más completo |
| `kl add <dep>@<ver>` | ✅ Solo modifica kl.toml | Que descargue y resuelva inmediatamente |
| `kl remove <dep>` | ✅ Solo modifica kl.toml | Que actualice kl.lock |
| `kl build` | ✅ Compila | Sin cambios |
| `kl run` | ✅ Ejecuta | Sin cambios |
| `kl test` | 🔶 Solo type-check | Ver Fase 12.1 |
| `kl publish` | ❌ No existe | Empaquetar + subir al registry |
| `kl login` | ❌ No existe | Guardar API key en ~/.kl/config.toml |
| `kl update` | ❌ No existe | Actualizar kl.lock |
| `kl outdated` | ❌ No existe | Listar deps desactualizadas |
| `kl info` | ✅ Muestra metadata | Sin cambios |
| `kl doc` | ❌ No existe | Generar documentación desde `##` comments |

---

### Fase 12: Tooling 🔜 (LSP, VS Code, tests, formatter)

**Filosofía:** Kyle debe tener un entorno de desarrollo moderno desde el día 1.
Esto significa: tests integrados, formatter, LSP con autocompletado real,
y extensión de VS Code que funcione out-of-the-box.

---

#### 12.1 — Test Framework

**Estado actual:** 🔶 `kl test` existe pero solo type-checkea archivos en `tests/`.

**Objetivo:** Sistema de testing integrado como Rust, pero más simple:
sin macros de procedimiento, sin `#[should_panic]`, sin fixtures complejas.

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 12.1.1 | Parser: `#[test]` attribute antes de `fn` | `klc_frontend/src/parser.rs` | ⭐⭐⭐ |
| 12.1.2 | `#[test]` fn debe: no tener parámetros, retornar `void` o `i32` | `klc_semantic/src/type_checker.rs` | ⭐⭐⭐ |
| 12.1.3 | `kl test` compila y ejecuta cada `#[test]` fn individualmente | `klc_cli/src/main.rs` | ⭐⭐⭐ |
| 12.1.4 | Reporte de resultados: `PASS`, `FAIL`, total, tiempo | `klc_cli/src/main.rs` | ⭐⭐⭐ |
| 12.1.5 | `assert(cond)`, `assert_eq(a, b)`, `assert_ne(a, b)` como builtins | `klc_runtime/src/lib.rs` | ⭐⭐⭐ |
| 12.1.6 | `assert_throws(fn, expected_error)` para testear errores | `klc_runtime/src/lib.rs` | ⭐⭐ |
| 12.1.7 | `#[test] ignore` para saltar tests | `klc_frontend/src/parser.rs` | ⭐⭐ |
| 12.1.8 | `kl test <filtro>` para ejecutar solo tests que coincidan | `klc_cli/src/main.rs` | ⭐⭐ |
| 12.1.9 | Test con salida: capturar `print()` durante tests | `klc_cli/src/main.rs` | ⭐⭐ |

**Sintaxis:**
```kyle
#[test]
fn test_suma():
    assert_eq(suma(2, 3), 5)

#[test] ignore
fn test_lento():
    # ...
```

---

#### 12.2 — LSP (Language Server Protocol)

**Estado actual:** 🔶 `kl lsp` existe. Tiene autocompletado básico, go-to-definition,
hover, y semantic tokens. Pero tiene limitaciones importantes.

**Lo que falta:**

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 12.2.1 | **Diagnósticos en tiempo real**: errores de sintaxis y tipo mientras se escribe | `klc_tools/src/lsp.rs` | ⭐⭐⭐ |
| 12.2.2 | **Diagnósticos incrementales**: solo re-analizar archivo modificado, no todo el proyecto | `klc_tools/src/lsp.rs` | ⭐⭐⭐ |
| 12.2.3 | **Autocompletado completo**: snippets, firmas de funciones, tipos | `klc_tools/src/lsp.rs` | ⭐⭐⭐ |
| 12.2.4 | **Autocompletado contextual**: solo mostrar métodos válidos para el tipo | `klc_tools/src/lsp.rs` | ⭐⭐⭐ |
| 12.2.5 | **Go-to-definition mejorado**: saltar a definición de función/clase en archivos del proyecto | `klc_tools/src/lsp.rs` | ⭐⭐⭐ |
| 12.2.6 | **Go-to-definition en dependencias**: saltar a definición dentro de paquetes instalados | `klc_tools/src/lsp.rs` | ⭐⭐ |
| 12.2.7 | **Find references**: encontrar todas las referencias a un símbolo | `klc_tools/src/lsp.rs` | ⭐⭐ |
| 12.2.8 | **Hover mejorado**: mostrar documentación de `##` comments | `klc_tools/src/lsp.rs` | ⭐⭐⭐ |
| 12.2.9 | **Code actions**: sugerencias automáticas (ej: "añadir import faltante") | `klc_tools/src/lsp.rs` | ⭐⭐ |
| 12.2.10 | **Document symbols**: lista de funciones/clases en el archivo actual | `klc_tools/src/lsp.rs` | ⭐⭐ |
| 12.2.11 | **Rename symbol**: refactorización segura (F2) | `klc_tools/src/lsp.rs` | ⭐⭐ |
| 12.2.12 | **Format on save**: ejecutar `kl fmt` al guardar | `klc_tools/src/lsp.rs` | ⭐⭐ |
| 12.2.13 | **Inlay hints**: mostrar tipos inferidos en variables | `klc_tools/src/lsp.rs` | ⭐⭐ |
| 12.2.14 | **Diagnósticos en `kl.toml`**: validar el manifest | `klc_tools/src/lsp.rs` | ⭐⭐ |
| 12.2.15 | **Code lens**: "Run test" button encima de `#[test]` fn | `klc_tools/src/lsp.rs` | ⭐ |

---

#### 12.3 — VS Code Extension

**Estado actual:** ✅ Existe con sintaxis highlighting, snippets, y comandos básicos.
Se conecta al LSP pero las capacidades del LSP son limitadas.

**Lo que falta:**

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 12.3.1 | **Syntax highlighting completo**: resaltar toda la sintaxis Kyle correctamente | `extension/syntaxes/kl.tmLanguage.json` | ⭐⭐⭐ |
| 12.3.2 | **Icono de lenguaje**: icono para archivos `.kl` | `extension/media/` | ⭐⭐ |
| 12.3.3 | **Task provider**: botones "Run", "Build", "Test" en la barra de estado | `extension/src/extension.ts` | ⭐⭐⭐ |
| 12.3.4 | **Problemas en tiempo real**: mostrar errores del LSP en el panel de problemas | `extension/src/extension.ts` | ⭐⭐⭐ |
| 12.3.5 | **Snippets actualizados**: snippets para toda la sintaxis moderna | `extension/snippets/kl.json` | ⭐⭐ |
| 12.3.6 | **Debug adapter**: step-through debugging (DAP) — opcional | `extension/src/debugger.ts` | ⭐ |
| 12.3.7 | **Testing UI**: mostrar tests en el panel de Testing de VS Code | `extension/src/extension.ts` | ⭐⭐ |
| 12.3.8 | **Extension packaging**: script para generar `.vsix` automáticamente | `scripts/build-extension.sh` | ⭐⭐ |
| 12.3.9 | **Publicación en marketplace**: VS Code Marketplace + Open VSX | CI/CD | ⭐⭐ |
| 12.3.10 | **Tema de color Kyle**: theme específico del lenguaje (colores pastel) | `extension/themes/` | ⭐ |

**Estructura de la extensión:**
```
extension/
├── package.json
├── syntaxes/
│   └── kl.tmLanguage.json      ← Gramática TextMate
├── snippets/
│   └── kl.json                 ← Snippets
├── src/
│   ├── extension.ts            ← Entry point
│   ├── lsp.ts                  ← Cliente LSP
│   ├── tasks.ts                ← Task provider (build/run/test)
│   ├── debugger.ts             ← Debug adapter (future)
│   └── testUI.ts               ← Testing integration
├── themes/
│   └── kl-color-theme.json     ← Tema opcional
└── media/
    └── kl-icon.png             ← Icono del lenguaje
```

---

#### 12.4 — Formatter (`kl fmt`)

**Estado actual:** ✅ `kl fmt` existe y formatea lo básico.

**Lo que falta:**

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 12.4.1 | Formatear toda la sintaxis moderna: `:=`, `::=`, `T?`, `final class`, etc. | `klc_tools/src/formatter.rs` | ⭐⭐⭐ |
| 12.4.2 | Formatear patterns de match (or-patterns, guards, destructuring) | `klc_tools/src/formatter.rs` | ⭐⭐ |
| 12.4.3 | Formatear closures inline `fn(x) x * 2` | `klc_tools/src/formatter.rs` | ⭐⭐ |
| 12.4.4 | Formatear imports (orden alfabético, agrupados) | `klc_tools/src/formatter.rs` | ⭐⭐ |
| 12.4.5 | `kl fmt --check` (CI mode: error si el archivo no está formateado) | `klc_cli/src/main.rs` | ⭐⭐ |
| 12.4.6 | Configuración de formato en `kl.toml` (`[format]` section) | `klc_core/src/manifest.rs` | ⭐⭐ |

**Reglas de formato (v1.0):**
- Indentación: 4 espacios (obligatorio)
- Longitud máxima de línea: 100 caracteres (configurable)
- Saltos de línea después de `:` en headers (fn, class, if, while, for, match)
- Un solo espacio después de `#` en comentarios
- Sin espacios al final de línea
- Un salto de línea al final del archivo

---

#### 12.5 — Shell Completions

**Estado actual:** ✅ `kl completions bash` existe y funciona.

**Lo que falta:**

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 12.5.1 | `kl completions zsh` | `klc_cli/src/main.rs` | ⭐⭐ |
| 12.5.2 | `kl completions fish` | `klc_cli/src/main.rs` | ⭐⭐ |
| 12.5.3 | `kl completions powershell` | `klc_cli/src/main.rs` | ⭐ |
| 12.5.4 | Autocompletado de nombres de dependencias en `kl add` | `klc_cli/src/main.rs` | ⭐⭐ |

---

### Fase 13: Sintaxis Restante 🔜 (características del lenguaje que faltan)

**Objetivo:** Implementar toda la sintaxis documentada que aún no funciona.

| # | Tarea | Sintaxis | Prioridad | Depende de |
|---|-------|----------|-----------|------------|
| 13.1 | **Genéricos en clases**: `final class Stack<T>:` | `final class Nombre<T>:` | ⭐⭐⭐⭐⭐ | — |
| 13.2 | **Rangos completos**: `0..5`, `0..=5`, `0..<5`, `3..`, `..3`, `..` | `start..end`, `start..=end`, etc. | ⭐⭐⭐ | — |
| 13.3 | **`is` type checking**: `x is str` → true/false | `expr is Type` | ⭐⭐⭐ | — |
| 13.4 | **`ptr` type completo**: aritmética de punteros para FFI | `ptr` como tipo usable | ⭐⭐ | — |
| 13.5 | **`null` literal**: valor nulo para `ptr` | `null` | ⭐⭐ | 13.4 |
| 13.6 | **Operator overloading**: `op_+(other)`, `op_-(other)`, `op_*(other)` | `fn op_+(other: T) T:` | ⭐⭐ | — |
| 13.7 | **`for-else:`**: bloque else si loop no hizo break | `for x in items: ... else: ...` | ⭐⭐ | — |
| 13.8 | **Loop labels completos**: `break 'label`, `continue 'label` | `'label: for ...` | ⭐⭐ | — |
| 13.9 | **Match destructuring**: `match pair: (x, y) => ...` | patterns en match | ⭐⭐ | — |
| 13.10 | **Match guard**: `match x: n if n > 0 => ...` | guard condicional | ⭐⭐ | — |
| 13.11 | **Enum methods**: `fn name():` dentro de `enum` | métodos en enum | ⭐⭐ | — |
| 13.12 | **`super.method()`**: llamar método padre sobreescrito | `super.nombre()` | ⭐⭐ | — |
| 13.13 | **`static fn`**: métodos estáticos en clases | `static fn name():` | ⭐⭐ | — |
| 13.14 | **`abstract fn`**: funciones abstractas en abstract class | `abstract fn name():` | ⭐⭐ | — |
| 13.15 | **`@` attribute token**: `#[attr]` sintaxis completa | `@` como token + parsing | ⭐⭐ | — |
| 13.16 | **`**: default operator: `expr ?: default` | `?:` | ⭐ | — |
| 13.17 | **`**` power operator correcto**: codegen real (hoy es mul incorrecto) | `a ** b` | ⭐⭐ | — |
| 13.18 | **`+%`, `-%`, `*%` percentage ops**: significado real (hoy parsean solo) | `x +% 10` | ⭐ | — |

**NOTA:** Or-patterns (`a | b`), Properties (get/set), y Default params ya están ✅ implementados.

---

### Fase 14: Borrow Checker 📅 (post-v1.0)

- [ ] ⭐ Referencias `&T` / `&mut T`
- [ ] ⭐ Inferencia de regiones (sin anotaciones de lifetime)

---

### Fase 15: Alternative Backends 📅 (post-v1.0)

- [ ] ⭐ Cranelift backend (compilación más rápida)
- [ ] ⭐ WASM target (web)

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
| **Async scheduler** | **9** | **✅ Thread pool V2 (async expr + await expr + async fn + runtime)** |
| | | **🔜 State machine V3, work-stealing, non-blocking I/O** |
| **Iterators** | **10** | **✅ 17 métodos de lista: sum/product/max/min/reverse/map/filter/fold/reduce/len/add/pop/contains/insert/remove_at/clear/clone** |
| | | **🔜 Closures funcionales (fn ptr primera clase) + lazy iterators** |
| **Performance (SSA Form)** | **15** | **✅ COMPLETADO** |
| | | **⚠️ Bug conocido: release mode hang** |
| | | **⚠️ Bug conocido: missing extern decls (kl_list_pop_first, etc.)** |
| **LLVM IR Quality** | **16** | **🔜 EN CURSO — ~8%** |
| | | **✅ 16.3 readonly/readnone en runtime externs (memory("read")/memory("none"))** |
| | | **🔲 16.0 Fix release mode hang** |
| | | **🔲 16.2 inbounds en GEPs** |
| | | **🔲 16.4 noalias en parámetros** |
| | | **🔲 16.9 TBAA metadata** |
| | | **🔲 16.5-16.8 align/noundef/!range/lifetime** |
| | | **⏳ 16.1 nsw/nuw (diferido: requiere range analysis)** |
| **Package manager** | **11** | **🔜 DETALLADO (registry, semver, publish, lock)** |
| **Tooling** | **12** | **🔜 DETALLADO (LSP, VS Code, tests, formatter)** |
| **Sintaxis Restante** | **13** | **🔜 DETALLADO (genéricos, rangos, is, ptr, etc.)** |
| **Borrow checker** | **14** | **📅 Post-v1.0** |
| **Alternative backends** | **15** | **📅 Post-v1.0** |

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

### Fase 15 — SSA Form ✅ COMPLETADO
- [x] i64 default literal type
- [x] ThinLTO + Alias Analysis
- [x] SSA Form — Mem2Reg + Phi nodes + dominator fix
- [x] GVN on SSA — Global Value Numbering
- [x] Etapa 1: Refactor compile_ssa_function — block_vals directos, sin bv_snapshot
- [x] Etapa 2: Fix cross-block value flow — phi incomings correctos para loops
- [x] Etapa 3: SSA para tipos complejos — Str/List/Dict no promovibles, Ptr/Array promovibles
- [x] Etapa 4: GVN post-SSA + Constant Propagation
- [x] Etapa 5: Pipeline SSA activo en release + fallback
- [x] Etapa 6: Benchmarks — resultados correctos en arithmetic, primes, mandelbrot
- [x] Etapa 7: Edge cases — async, closures, for-range, match

### Fase 16 — LLVM IR Quality 🔜 EN CURSO (~8%)
- [ ] 🔴 **16.0 — Fix release mode hang** (PRIORIDAD #0, bloqueante)
- [x] ✅ **16.3 — `readonly`/`readnone` en runtime externs** — `memory("read")` en 13 funciones, `memory("none")` en 7 funciones
- [ ] ⭐⭐⭐⭐⭐ **16.2 — `inbounds` en GEPs** (crítico: 2-3×, podría arreglar release hang)
- [ ] ⭐⭐⭐⭐ **16.4 — `noalias` en parámetros puntero** (alto: 1.5-3×)
- [ ] ⭐⭐⭐ **16.9 — TBAA metadata para alias analysis** (alto: 1.5-2×)
- [ ] ⭐⭐ **16.5 — `align` explícito en loads/stores/allocas** (medio: 1.1-1.5×)
- [ ] ⭐⭐ **16.6 — `noundef` en parámetros** (medio: 1.1-1.3×)
- [ ] ⭐⭐ **16.7 — `!range` metadata en bool y tipos acotados** (medio: 1.1-1.3×)
- [ ] ⭐ **16.8 — `lifetime.start`/`lifetime.end`** (bajo: 1.05-1.1×)
- [ ] ⏳ **16.1 — `nsw`/`nuw` flags** (DIFERIDO: requiere range analysis)

**NOTA 16.1 (nsw/nuw):** No pueden aplicarse genéricamente porque Kyle define
integer overflow como wrapping (no UB). Requieren un análisis de rangos (range
analysis) para probar que no hay overflow antes de aplicar. Pendiente para futuro.

**LOGRO 16.3 (readonly/readnone):** Verificado en IR generado:
```llvm
attributes #0 = { "memory"="read" }   ; 13 funciones readonly
attributes #1 = { "memory"="none" }   ; 7 funciones readnone (pure)
```

### Fase 17 — Zero-Cost Abstractions 📅
- [ ] ⭐⭐⭐⭐ 17.1 — Stack allocation para `final class` pequeños (escape analysis)
- [ ] ⭐⭐⭐ 17.2 — Inlining completo de `.map()`/`.filter()`/`.fold()`
- [ ] ⭐⭐⭐ 17.3 — Monomorfización verificada en LLVM IR
- [ ] ⭐⭐⭐ 17.4 — Eliminación de vtables para clases sin herencia
- [ ] ⭐⭐ 17.5 — Devirtualización de métodos

### Fase 18 — Memory & Stack Optimizations 📅
- [ ] ⭐⭐⭐⭐ 18.1 — Escape analysis: `final class` en stack si no escapa
- [ ] ⭐⭐⭐ 18.2 — Small string optimization (SSO): strings < 15 bytes inlineados
- [ ] ⭐⭐⭐ 18.3 — Array optimizations: small arrays en stack
- [ ] ⭐⭐ 18.4 — Return value optimization (RVO)

### Fase 9 — Async Scheduler (continuación)
- [ ] 9.1-9.5 State machine V3 (reemplazar thread pool)
- [ ] 9.6-9.8 Work-stealing scheduler
- [ ] 9.9-9.11 Non-blocking I/O

### Fase 10 — Iterators (continuación)
- [ ] 10.1-10.5 Closures funcionales (fn ptr primera clase)
- [ ] 10.6-10.9 Lazy evaluation / `iter()` trait

### Fase 11 — Package Manager 🔜
- [ ] 11.1 Manifest completo (validación, versiones, dev-deps)
- [ ] 11.2 Resolución semver + lock file + cache
- [ ] 11.3 Registry (cliente HTTP, descarga, publish, login)
- [ ] 11.4 Importación desde paquetes resueltos
- [ ] 11.5 Comandos: `kl add` real, `kl publish`, `kl login`, `kl update`, `kl outdated`

### Fase 12 — Tooling 🔜
- [ ] 12.1 Test framework (`#[test]`, assert builtins, kl test)
- [ ] 12.2 LSP completo (diagnósticos, autocompletado contextual, hover con docs, code actions, find refs)
- [ ] 12.3 VS Code extension (task provider, testing UI, packaging, marketplace)
- [ ] 12.4 Formatter completo (`kl fmt --check`, toda la sintaxis)
- [ ] 12.5 Shell completions (zsh, fish, powershell)

### Fase 13 — Sintaxis Restante 🔜
- [ ] 13.1 Genéricos en clases (`final class Stack<T>:`)
- [ ] 13.2 Rangos completos (`0..5`, `..=`, `..<`, `..`, `3..`)
- [ ] 13.3 `is` type checking (`x is str`)
- [ ] 13.4 `ptr` type completo + `null` literal
- [ ] 13.5 Operator overloading (`op_+`, etc.)
- [ ] 13.6 `for-else:` + loop labels
- [ ] 13.7 Match patterns (destructuring, guards)
- [ ] 13.8 Enum methods + `static fn` + `super.method()`
- [ ] 13.9 `**` power operator correcto
- [ ] 13.10 `@` attribute token + `?:` default operator + `+%`/`-%`/`*%`

---

## 9. Test Counts

| Suite | Count | Status |
| :--- | :--- | :--- |
| `klc_frontend` unit tests | 82 | ✅ All passing |
| `klc_semantic` unit tests | 17 | ✅ All passing |
| `klc_mir` unit tests | 11 | ✅ All passing |
| `klc_tools` unit tests | 4 | ✅ All passing |
| `klc_runtime` unit tests | 0 | n/a (C-ABI) |
| `klc_backend` unit tests | 0 | n/a |
| `klc_core` unit tests | 0 | n/a |
| `klc_driver` unit tests | 9 | ✅ All passing |
| `klc_cli` unit tests | 0 | n/a |
| End-to-end `kl test` | 12 | 11/12 passing (1 pre-existing failure: test_misc.kl) |
| **Total Rust unit tests** | **123** | **✅ All passing** |

---

## 10. Estado Actual (v0.4.0)

### Completado desde v0.3.0

| Componente | Estado |
|------------|--------|
| Fase 5: HIR crate + desugaring | ✅ |
| Fase 6: Semantic Analysis (13/13) | ✅ |
| Fase 7: Move Semantics (13/13) | ✅ |
| Fase 8: Release mode (`OptimizationLevel::Aggressive`) | ✅ (⚠️ hang en SSA+release) |
| Fase 9: Async Thread Pool V2 (`async fn`, `async expr`, `await expr`) | ✅ |
| Fase 10: Iterators — 17 métodos de lista | ✅ |
| Fase 15: SSA Form — Mem2Reg, Phi, GVN, benchmarks correctos (debug) | ✅ |
| Fase 16.3: `readonly`/`readnone` en runtime externs | ✅ |
| **123 tests Rust** (↑ desde 101) | ✅ |
| `ownership.rs` y `kl_release` declaration removidos | ✅ |
| `print_int`/`println_int` builtins removidos → ahora `print(42)` | ✅ |
| List borrowing fix — `kl_list_push/get/set/len` en borrowing funcs | ✅ |
| Built-in type methods: `add/pop/len/upper/lower/trim/contains/replace` | ✅ |
| Proyecto de prueba `examples/src/main.kl` con 41 secciones | ✅ |
| `.map()`, `.filter()`, `.fold()`, `.reduce()` como métodos (vía fn ptr C-ABI) | ✅ |
| Bugs SSA fix: `const_values` en Call, CondBr trunc i1 | ✅ |

### Pendiente inmediato
| Prioridad | Tarea | Fase |
|-----------|-------|------|
| 🔴 CRÍTICO | Fix release mode hang | 16.0 |
| 🔴 CRÍTICO | `inbounds` en GEPs (podría arreglar release hang) | 16.2 |
| 🟡 ALTO | `noalias` en parámetros puntero de runtime externs | 16.4 |
| 🟡 ALTO | Missing extern declarations (`kl_list_pop_first`, etc.) | 15.B2 |
| 🟢 MEDIO | TBAA metadata, align, noundef, !range, lifetime | 16.5-16.9 |
| ⏳ FUTURO | nsw/nuw flags (requiere range analysis primero) | 16.1 |

### Bugs encontrados y arreglados

| Bug | Síntoma | Fix | Archivo |
|-----|---------|-----|---------|
| `as f64` casting | `(39 as f64)` producía ~3.4e-309 | Agregar pattern `IntValue → FloatType` | `codegen.rs` |
| `const_values` en Call | Argumentos constantes en llamadas a función (ej: `print("A")`) producían output nulo | Insertar entrada en `const_values` para argumentos | `ssa.rs` |
| CondBr i1 trunc | Comparaciones devuelven i1 pero CondBr comparaba con i32 0 | Truncar a i1 si `bit_width > 1` | `codegen.rs` |

### Issues Conocidos

| Issue | Síntoma | Causa raíz | Estado |
|-------|---------|------------|--------|
| Release mode hang (15.B1) | `kl build --release` produce binarios que cuelgan | SSA + LLVM aggressive optimization sin atributos | 🔴 Sin fix |
| Missing extern decls (15.B2) | Funciones `kl_list_*` existen en lower.rs/runtime pero no en LLVM | `declare_runtime_externs()` incompleto | 🟡 Sin fix |
| Duplicate externs (15.B3) | `kl_dict_new/get/set/len/free` declaradas 2 veces | Refactor incompleto | 🟢 Cosmético |

### Resultados de benchmark (2026-06-30) — hyperfine, shell=none, warmup=5

**Prueba 1: Aritmética** — `total = total + i * 2 - 1` (10M iteraciones, i32 wrap)
| Lenguaje | Media ± σ | vs Rust | vs Python |
| :--- | :--- | :--- | :--- |
| Rust | 208 µs ± 47 µs | 1× | 2966× |
| Java 21 | 17.0 ms ± 2.0 ms | 82× más lento | 36× |
| **Kyle** | **22.0 ms** ± 0.5 ms | 106× más lento | 28× |
| Python 3 | 616 ms ± 12 ms | 2966× más lento | 1× |

**Prueba 2: Primos** — `is_prime()` hasta 100000
| Lenguaje | Media ± σ | vs Rust | vs Python |
| :--- | :--- | :--- | :--- |
| Rust | 2.0 ms ± 0.1 ms | 1× | 33× |
| **Kyle** | **7.3 ms** ± 1.1 ms | 3.7× más lento | 9× |
| Java 21 | 17.3 ms ± 4.5 ms | 8.7× más lento | 3.8× |
| Python 3 | 66.4 ms ± 4.6 ms | 33× más lento | 1× |

**Prueba 3: Mandelbrot** — 78×78 grid, 100 max iter (punto flotante)
| Lenguaje | Media ± σ | vs Rust | vs Python |
| :--- | :--- | :--- | :--- |
| Rust | 612 µs ± 38 µs | 1× | 35× |
| **Kyle** | **2.4 ms** ± 0.3 ms | 3.9× más lento | 9× |
| Java 21 | 23.9 ms ± 5.0 ms | 39× más lento | 0.9× |
| Python 3 | 21.6 ms ± 5.2 ms | 35× más lento | 1× |

**Resumen vs Rust:**
| Benchmark | Rust | **Kyle** | Java | Python |
|-----------|------|----------|------|--------|
| Arithmetic | 0.2 ms | **22 ms** (106×) | 17 ms (82×) | 616 ms (2966×) |
| Primes | 2.0 ms | **7.3 ms** (3.7×) | 17 ms (8.7×) | 66 ms (33×) |
| Mandelbrot | 0.6 ms | **2.4 ms** (3.9×) | 24 ms (39×) | 22 ms (35×) |

**Conclusión:**
- Kyle es ~3.7-106× más lento que Rust en debug mode.
- La brecha de 106× en arithmetic se debe a la FALTA de múltiples atributos LLVM
  (inbounds, readonly, noalias, TBAA) combinada, no solo a nsw/nuw.
- nsw/nuw están **diferidos** porque Kyle define overflow como wrapping (no UB),
  y aplicar nsw genéricamente es incorrecto.
- La prioridad #1 es arreglar el release mode hang (16.0), que bloquea cualquier
  medición de rendimiento en modo optimizado.
- **Kyle vence a Java 21** en primes (7.3ms vs 17ms) y mandelbrot (2.4ms vs 24ms),
  incluso en debug mode.
- Con Fase 16 completa (sin nsw/nuw), Kyle debería operar en 1-3× de Rust,
  superando a Java 21 en todos los benchmarks.

---

### Fase 15 — SSA Form ✅ COMPLETADO (con issues conocidos post-SSA)

#### 🔬 Diagnóstico Final (Junio 2026)

| Componente | Estado | Problema resuelto |
|-----------|--------|-------------------|
| `SsaFunction` struct | ✅ | Correcto |
| Mem2Reg (promoción allocas) | ✅ | Identifica allocas promovibles correctamente |
| Phi placement + incomings | ✅ | Algoritmo de dominación con fix de intersección infinita |
| GVN on SSA | ✅ | Elimina cómputos redundantes entre bloques |
| **compile_ssa_function** | ✅ | Elimina allocas para vars promovidas, usa block_vals directos |
| **Pipeline SSA** | ✅ | Activo en release mode, con fallback a non-SSA |
| **Benchmarks (debug)** | ✅ | ssa_test, ssa_loop2, arithmetic, todos correctos |

#### ⚠️ Issues Conocidos Post-SSA

| # | Issue | Síntoma | Causa Raíz | Prioridad |
|---|-------|---------|------------|-----------|
| 15.B1 | **Release mode hang** | `kl build --release` produce binarios que cuelgan (no terminan) | SSA pipeline + `OptimizationLevel::Aggressive` genera IR que LLVM optimiza incorrectamente (posiblemente por falta de atributos como `inbounds`/`readonly`/`noalias` que causan loops infinitos post-optimización) | 🔴 CRÍTICO |
| 15.B2 | **Missing extern declarations** | Funciones como `kl_list_pop_first`, `kl_list_clear`, `kl_list_contains`, `kl_list_insert`, `kl_list_remove_at`, `kl_list_map`, `kl_list_filter`, `kl_list_fold`, `kl_list_reduce`, `kl_iter_new`, `kl_iter_next`, `kl_iter_map`, `kl_iter_filter`, `kl_iter_collect` existen en `lower.rs` y runtime pero NO están declaradas como LLVM externs en `declare_runtime_externs()` de `codegen.rs` | `codegen.rs` olvidó declararlas | 🟡 ALTO |
| 15.B3 | **Duplicate extern declarations** | `kl_dict_new`, `kl_dict_free`, `kl_dict_get`, `kl_dict_set`, `kl_dict_len` están declaradas DOS VECES en `declare_runtime_externs()` | Refactor incompleto | 🟢 BAJO |
| 15.B4 | **`kl_retain`/`kl_release` sin usar** | Las funciones existen en runtime pero no son llamadas por el compilador | Move semantics reemplazó refcounting | 🟢 BAJO |

**NOTA 15.B1:** El release mode hang NO fue causado por ningún cambio reciente.
Se confirmó haciendo `git stash` (revirtiendo readonly/noalias/readnone) y el
hang persistió. Es un bug pre-existente del pipeline SSA en combinación con
LLVM optimizations.

**Solución propuesta 15.B1:** Antes de activar SSA en release, asegurar que el IR
tenga al menos `inbounds` en GEPs (16.2) y `memory("read")`/`memory("none")` en
runtime externs (16.3) para que LLVM no maloptimice los loops.

#### ✅ Lo que se implementó (Fase 15 completa)

| # | Tarea | Archivo | Estado |
|---|-------|---------|--------|
| 1.1 | Eliminar `bv_snapshot` → usar `block_vals[bi]` mutable con patrón read/write secuencial | `codegen.rs` | ✅ |
| 1.2 | `SsaInst::Store` promovidas: NO emitir LLVM store, solo `alloca_current` | `codegen.rs` | ✅ |
| 1.3 | `SsaInst::Load` promovidas: leer de `alloca_current`, no LLVM load | `codegen.rs` | ✅ |
| 1.4 | `SsaInst::BinaryOp`/`UnaryOp`/`Cast`: operandos de `block_vals[bi]` | `codegen.rs` | ✅ |
| 1.5 | `phi_map` accesible para operandos phi directos | `codegen.rs` | ✅ |
| 2.1 | Seed `alloca_current` con valores phi de cada bloque | `codegen.rs` | ✅ |
| 2.2 | `block_end_values` para snapshots por bloque | `ssa.rs` | ✅ |
| 2.3 | Phi incomings desde `block_end_values[pred_idx]` | `ssa.rs` | ✅ |
| 2.4 | Loops simples verificados: `while i < N: total = total + i; i = i + 1` | `ssa_test.kl` | ✅ |
| 3.1 | No promover `Str`/`List`/`Dict`/`Struct` (escapan via heap) | `ssa.rs` | ✅ |
| 3.2 | SSA para `Ptr` y `Array` (tipos Copy) | `ssa.rs` | ✅ |
| 3.3 | `Memcpy` y `FieldPtr` en codegen SSA | `codegen.rs` | ✅ |
| 4.1 | GVN post-SSA: redundancias entre bloques | `ssa.rs` | ✅ |
| 4.2 | Constant propagation en SSA | `ssa.rs` | ✅ |
| 5.1 | Pipeline SSA activo en release | `pipeline.rs` | ✅ |
| 5.2 | Fallback a non-SSA en errores | `pipeline.rs` | ✅ |
| 5.3 | `cargo test` pasa con SSA activo | — | ✅ |
| 6.1 | Benchmarks arithmetic, primes, mandelbrot correctos (debug) | `examples/bench/` | ✅ |
| 7.1-7.6 | Edge cases: async, closures, for-range, match | varios | ✅ |
| — | Fix: `const_values` para argumentos constantes en Call | `ssa.rs` | ✅ |
| — | Fix: CondBr trunc i1 para comparaciones | `codegen.rs` | ✅ |

---

### Fase 16 — LLVM IR Quality 🔜 EN CURSO (~8%)

#### 🔬 Diagnóstico (Junio 2026)

El pipeline SSA (Fase 15) está completo y produce código correcto en debug mode.
**Sin embargo, el LLVM IR generado carece de atributos de optimización críticos**,
lo que impide que LLVM aplique su pipeline completo. Esto explica la brecha de
rendimiento actual: Kyle está ~3-106× detrás de Rust.

**IR actual de Kyle (para loop aritmético):**
```llvm
%total.0 = phi i32 [0, %entry], [%total.2, %loop]   ; ← SSA correcto
%i.0     = phi i32 [0, %entry], [%i.1, %loop]         ; ← SSA correcto
%tmp     = mul i32 %i.0, 2                            ; ← FALTA nsw
%total.1 = add i32 %total.0, %tmp                     ; ← FALTA nsw
%i.1     = add i32 %i.0, 1                            ; ← FALTA nsw
call i32 @kl_strlen(ptr %s)                           ; ← FALTA memory("read")
call i32 @kl_is_digit(i8 %c)                          ; ← FALTA memory("none")
```

**IR deseado (para competir con Rust):**
```llvm
%total.0 = phi i32 [0, %entry], [%total.2, %loop]
%i.0     = phi i32 [0, %entry], [%i.1, %loop]
%tmp     = mul nsw i32 %i.0, 2       ; nsw → SCEV loop optimization
%total.1 = add nsw i32 %total.0, %tmp ; nsw → inducción de variables
%i.1     = add nsw i32 %i.0, 1       ; nsw → eliminación de variables de inducción
call i32 @kl_strlen(ptr %s) #0       ; #0 = memory("read") → CSE/hoisting
call i32 @kl_is_digit(i8 %c) #1      ; #1 = memory("none") → puede eliminar llamada
```

#### ⚠️ Issue Bloqueante: Release Mode Hang (15.B1)

**Antes de implementar cualquier sub-fase de Fase 16, hay que solucionar el
release mode hang.** La hipótesis actual es que LLVM, al recibir IR sin
atributos de optimización (sin `inbounds`, sin `memory("read")`, sin `noalias`),
aplica transformaciones incorrectas que resultan en loops infinitos.

**Estrategia:** Implementar 16.2 (inbounds) y 16.3 (readonly/readnone) primero,
luego re-testear release mode. Estas sub-fases son seguras (no cambian semántica)
y podrían resolver el hang. Si no, investigar más a fondo.

---

#### 📋 Plan de Trabajo — 9 Sub-fases

##### Sub-fase 16.0: Fix Release Mode Hang 🔜 PRIORIDAD #1
**Objetivo:** Diagnosticar y corregir el hang en `kl build --release`.

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 16.0.1 | Implementar 16.2 (inbounds) + 16.3 (readonly) y re-testear | varios | 🔴 |
| 16.0.2 | Si persiste: reducir `OptimizationLevel::Aggressive` a `Default` en SSA pipeline | `pipeline.rs` | 🔴 |
| 16.0.3 | Si persiste: desactivar SSA en release mode (usar non-SSA siempre) | `pipeline.rs` | 🔴 |
| 16.0.4 | Verificar: `kl build --release && hyperfine` en arithmetic, primes, mandelbrot | — | 🔴 |

---

##### Sub-fase 16.1: `nsw`/`nuw` flags en aritmética 🔜 DIFERIDO (requiere range analysis)
**⚠️ BLOQUEADO:** No se puede aplicar genéricamente.

**Problema:** Kyle define integer overflow como WRAPPING (no UB). El benchmark
`arithmetic.kl` calcula `total + i * 2 - 1` que WRAPEA intencionalmente
(resultado: 256447232). Si se aplica `nsw`, LLVM asume overflow = UB y puede
optimizar incorrectamente (causando el hang observado).

**Solución:** Implementar un Análisis de Rangos (Range Analysis) que demuestre
que ciertas operaciones NUNCA wrappean, y solo entonces aplicar `nsw`/`nuw`.
Ejemplo: `i + 1` donde `i < N` y `N < 2^31` es seguro.

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 16.1.1 | Implementar Range Analysis en MIR (`mir_range` analysis pass) | `klc_mir/src/range.rs` (nuevo) | ⭐⭐⭐⭐⭐ |
| 16.1.2 | Marcar operaciones con rango conocido como `no_overflow` | `klc_mir/src/range.rs` | ⭐⭐⭐⭐⭐ |
| 16.1.3 | Codegen: emitir `nsw` solo si `no_overflow` está marcado | `codegen.rs` | ⭐⭐⭐⭐⭐ |
| 16.1.4 | Emitir `nsw`+`nuw` en `build_left_shift` cuando sea seguro | `codegen.rs` | ⭐⭐⭐⭐ |
| 16.1.5 | Verificar: arithmetic benchmark baja de 22ms (debug) | — | ⭐⭐⭐⭐⭐ |

**API Inkwell (para cuando se implemente):**
- `build_int_add` → usar `build_int_nsw_add(li, ri, "")` (devuelve `Result<IntValue>`)
- `build_int_sub` → usar `build_int_nsw_sub(li, ri, "")`
- `build_int_mul` → usar `build_int_nsw_mul(li, ri, "")`

---

##### Sub-fase 16.2: `inbounds` en GEPs 🆕 PRIORIDAD #2 (CRÍTICO — 2-3×)
**Objetivo:** Emitir `inbounds` en todos los `build_struct_gep` y `build_gep`.
Sin `inbounds`, LLVM asume que cada GEP podría wrappear y aliasear con cualquier
otro puntero, bloqueando alias analysis y SCEV.

**INBOUNDS ES SIEMPRE SEGURO** para GEPs que acceden a memoria válida dentro
de la asignación: Kyle no permite aritmética de punteros arbitraria, y todos
los GEPs son generados por el compilador para acceder a struct fields o arrays
con índices conocidos.

| # | Tarea | Archivo | Líneas | Cómo |
|---|-------|---------|--------|------|
| 16.2.1 | `build_struct_gep` → pasar `inbounds=true` como parámetro | `codegen.rs` | ~10 ubicaciones | Buscar `build_struct_gep(` y agregar argumento |
| 16.2.2 | `build_gep` → pasar `inbounds=true` como parámetro | `codegen.rs` | ~5 ubicaciones | Buscar `build_gep(` y agregar argumento |
| 16.2.3 | Verificar: `cargo test` sigue pasando | — | — | `cargo test --workspace` |
| 16.2.4 | Verificar: release mode hang se resuelve | — | — | `kl build --release examples/bench/ssa_test.kl` |

**API Inkwell:** `builder.build_struct_gep(ptr, idx, name, inbounds)` —
el cuarto parámetro booleano controla `inbounds`.

---

##### Sub-fase 16.3: `readonly`/`readnone` en runtime externs ✅ COMPLETADO

**Objetivo:** Anotar funciones runtime con `memory("read")` (no escribe memoria)
o `memory("none")` (no accede memoria). Sin estos, LLVM no puede CSE, hoistear
llamadas fuera de loops, ni eliminarlas.

**Implementado en:** `codegen.rs` — `declare_runtime_externs()` mediante nuevo
helper `add_runtime_extern()` que acepta pares clave-valor como string attributes.

| Función | Atributo | Justificación |
|---------|----------|---------------|
| `kl_strlen(ptr) -> i32` | `memory("read")` | Solo lee bytes de la string |
| `kl_char_at(ptr, i32) -> i8` | `memory("read")` | Lee 1 byte de la string |
| `kl_eq_str(ptr, ptr) -> i32` | `memory("read")` | Compara bytes de 2 strings |
| `kl_str_contains(ptr, ptr) -> i32` | `memory("read")` | Busca substring |
| `kl_list_len(ptr) -> i64` | `memory("read")` | Lee field `len` del struct |
| `kl_list_get(ptr, i64) -> i64` | `memory("read")` | Lee elemento del array |
| `kl_list_sum(ptr) -> i64` | `memory("read")` | Reduce: solo lee |
| `kl_list_product(ptr) -> i64` | `memory("read")` | Reduce: solo lee |
| `kl_list_max(ptr) -> i64` | `memory("read")` | Reduce: solo lee |
| `kl_list_min(ptr) -> i64` | `memory("read")` | Reduce: solo lee |
| `kl_dict_get(ptr, ptr) -> i64` | `memory("read")` | Busca en HashMap |
| `kl_dict_len(ptr) -> i64` | `memory("read")` | Lee tamaño del HashMap |
| `kl_dict_contains(ptr, ptr) -> i32` | `memory("read")` | Busca key en HashMap |
| `kl_is_digit(i8) -> i32` | `memory("none")` | Operación pura: 0 memoria |
| `kl_is_alpha(i8) -> i32` | `memory("none")` | Operación pura |
| `kl_is_alnum(i8) -> i32` | `memory("none")` | Operación pura |
| `kl_is_whitespace(i8) -> i32` | `memory("none")` | Operación pura |
| `kl_is_upper(i8) -> i32` | `memory("none")` | Operación pura |
| `kl_is_lower(i8) -> i32` | `memory("none")` | Operación pura |
| `kl_ord(i8) -> i32` | `memory("none")` | Operación pura |

**Verificación en IR generado:**
```llvm
attributes #0 = { "memory"="read" }   ; readonly externs
attributes #1 = { "memory"="none" }   ; readnone externs
```

**NOTA:** `kl_now()` NO tiene `memory("read")` porque su valor cambia entre
llamadas aunque la memoria no cambie — readonly permitiría a LLVM CSE dos
llamadas adyacentes, lo cual sería incorrecto.
`kl_i64_to_str()` NO tiene `memory("read")` porque ALLOCA memoria (heap).

---

##### Sub-fase 16.4: `noalias` en parámetros puntero 🆕 PRIORIDAD #3 (ALTO — 1.5-3×)
**Objetivo:** Marcar con `noalias` los parámetros `ptr` de funciones runtime.
`noalias` es el atributo individual más impactante para alias analysis.

**CÓMO:** En LLVM 18, `noalias` es un **parameter attribute** (no function attribute).
Se aplica con `AttributeLoc::Param(idx)`.

**API Inkwell:**
```rust
let noalias_kind = Attribute::get_named_enum_kind_id("noalias");
let noalias_attr = self.context.create_enum_attribute(noalias_kind, 0);
func.add_attribute(AttributeLoc::Param(0), noalias_attr);  // primer ptr
```

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 16.4.1 | `kl_print(ptr, i32)` → param 0 `noalias` | `codegen.rs` | ⭐⭐⭐⭐ |
| 16.4.2 | `kl_println(ptr, i32)` → param 0 `noalias` | `codegen.rs` | ⭐⭐⭐⭐ |
| 16.4.3 | `kl_strlen(ptr)` → param 0 `noalias` | `codegen.rs` | ⭐⭐⭐⭐ |
| 16.4.4 | `kl_str_contains(ptr, ptr)` → ambos params `noalias` | `codegen.rs` | ⭐⭐⭐⭐ |
| 16.4.5 | `kl_eq_str(ptr, ptr)` → ambos params `noalias` | `codegen.rs` | ⭐⭐⭐⭐ |
| 16.4.6 | `kl_concat(ptr, i32, ptr, i32)` → ambos ptr `noalias` | `codegen.rs` | ⭐⭐⭐⭐ |
| 16.4.7 | `kl_list_*(ptr, ...)` → params ptr `noalias` | `codegen.rs` | ⭐⭐⭐⭐ |
| 16.4.8 | `kl_dict_*(ptr, ...)` → params ptr `noalias` | `codegen.rs` | ⭐⭐⭐⭐ |
| 16.4.9 | `kl_alloc(i64) → ptr` → return value `noalias` | `codegen.rs` | ⭐⭐⭐⭐ |
| 16.4.10 | `kl_i64_to_str(i64) → ptr` → return value `noalias` | `codegen.rs` | ⭐⭐⭐⭐ |

**NOTA:** El return `noalias` se aplica con `AttributeLoc::Return`.

---

##### Sub-fase 16.5: `align` explícito en loads/stores/allocas 🆕 PRIORIDAD #4 (MEDIO — 1.1-1.5×)
**Objetivo:** Especificar alignment exacto en cada load/store/alloca según el tipo.

**CÓMO:** 
- `build_alloca(ty, name)` → `build_alloca(ty, name, align)`
- `build_store(val, ptr)` → `build_store(val, ptr).set_alignment(align)`
- `build_load(ty, ptr, name)` → `build_load(ty, ptr, name).set_alignment(align)`

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 16.5.1 | `build_alloca` con align del tipo (i32→4, i64→8, ptr→8) | `codegen.rs` | ⭐⭐ |
| 16.5.2 | `build_store` con align del tipo destino | `codegen.rs` | ⭐⭐ |
| 16.5.3 | `build_load` con align del tipo origen | `codegen.rs` | ⭐⭐ |

**API Inkwell:**
```rust
// align en alloca
let alloca = builder.build_alloca(ty, "name", align);
// align en load/store
let load = builder.build_load(ty, ptr, "name");
load.set_alignment(align).unwrap();
```

---

##### Sub-fase 16.6: `noundef` en parámetros 🆕 PRIORIDAD #5 (MEDIO — 1.1-1.3×)
**Objetivo:** Marcar parámetros como `noundef` (nunca son undef/poison).

**CÓMO:** Aplicar en `declare_function()` y `declare_ssa_function()` cuando
se crean los FunctionValue en LLVM. Todos los parámetros en Kyle son siempre
definidos (el compilador no emite undef).

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 16.6.1 | `noundef` en parámetros de funciones de usuario (non-SSA) | `codegen.rs` | ⭐⭐ |
| 16.6.2 | `noundef` en parámetros de funciones de usuario (SSA) | `codegen.rs` | ⭐⭐ |
| 16.6.3 | `noundef` en parámetros de runtime externs | `codegen.rs` | ⭐⭐ |

**API Inkwell:**
```rust
let noundef_kind = Attribute::get_named_enum_kind_id("noundef");
let attr = self.context.create_enum_attribute(noundef_kind, 0);
func.add_attribute(AttributeLoc::Param(idx), attr);
```

---

##### Sub-fase 16.7: `!range` metadata en bool y tipos acotados 🆕 PRIORIDAD #6 (MEDIO — 1.1-1.3×)
**Objetivo:** Emitir `!range !{i32 0, i32 2}` para valores bool.

**CÓMO:** Kyle representa `bool` como `i32` con valores 0 (false) o 1 (true).
Después de `build_load` de un bool, agregar metadata `!range`.

**API Inkwell:**
```rust
// Crear metadata: !{i32 0, i32 2}
let zero = self.context.i32_type().const_int(0, false);
let two = self.context.i32_type().const_int(2, false);
let range_md = self.context.metadata_node(&[zero.as_metadata_value(), two.as_metadata_value()]);
load.set_metadata(range_md, LLVMDebugVersion);  // o usar metadata kind ID
```

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 16.7.1 | `!range` en loads de valores bool | `codegen.rs` | ⭐⭐ |
| 16.7.2 | `!range` en returns de funciones bool | `codegen.rs` | ⭐⭐ |

---

##### Sub-fase 16.8: `lifetime.start`/`lifetime.end` 🆕 PRIORIDAD #7 (BAJO — 1.05-1.1×)
**Objetivo:** Marcar lifetimes de variables locales para dead store elimination.

**CÓMO:** Al entrar a un bloque (entry), emitir `lifetime.start` con el tamaño
de la variable. Antes de retornar, emitir `lifetime.end`.

**API Inkwell:**
```rust
// lifetime.start(ptr, size_in_bytes)
builder.build_lifetime_start(ptr, size);
// lifetime.end(ptr, size)
builder.build_lifetime_end(ptr, size);
```

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 16.8.1 | `lifetime.start` al inicio del entry block | `codegen.rs` | ⭐ |
| 16.8.2 | `lifetime.end` antes de ret/return | `codegen.rs` | ⭐ |

---

##### Sub-fase 16.9: TBAA metadata 🆕 PRIORIDAD #8 (ALTO — 1.5-2×)
**Objetivo:** Emitir Type-Based Alias Analysis metadata para distinguir accesos
a diferentes tipos.

**CÓMO:** Crear un TBAA tree en el módulo (nodo raíz: "Kyle types", hijos: tipos
primitivos). Emitir `!tbaa` en loads/stores según el tipo.

**API Inkwell:**
```rust
// Crear TBAA metadata
let dbg_mdkind = self.context.get_kind_id("tbaa");
let root = self.context.metadata_node(&[
    self.context.metadata_string("Kyle types"),
    self.context.metadata_string("Kyle types"),
]);
// Para un load i32:
let i32_node = self.context.metadata_node(&[
    self.context.metadata_string("i32"),
    root.as_metadata_value(),
]);
load.set_metadata(i32_node.as_metadata_value(), dbg_mdkind);
```

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 16.9.1 | Crear TBAA tree con tipos básicos (i32, i64, f64, ptr, struct) | `codegen.rs` | ⭐⭐⭐ |
| 16.9.2 | Emitir `!tbaa` en loads/stores de tipos escalares | `codegen.rs` | ⭐⭐⭐ |
| 16.9.3 | Emitir `!tbaa` en loads/stores de struct/class | `codegen.rs` | ⭐⭐⭐ |

---

#### 📊 Proyección de Rendimiento (Fase 16 completa)

| Benchmark | Kyle HOY (debug) | Rust | +16.3 readonly | +16.2 inbounds | +Fase 16 completa | Objetivo |
|-----------|----------|------|---------------|---------------|-------------------|----------|
| Arithmetic (10M) | 22ms | 0.2ms | ~22ms | ~20ms | **0.5-1ms** | **1-2× Rust** |
| Primes (<100K) | 7.3ms | 2.0ms | ~7ms | ~5ms | **2.5-3ms** | **1-1.5× Rust** |
| Mandelbrot (78×78) | 2.4ms | 0.6ms | ~2.4ms | ~2.4ms | **0.8-1ms** | **1-1.5× Rust** |

**NOTA:** Las ganancias reales de 16.2 y 16.3 solo se verán en release mode,
que actualmente está bloqueado por el hang (16.0). Las proyecciones asumen
release mode funcional.

---

#### 📋 Orden de Implementación Recomendado (Actualizado)

| Prioridad | Sub-fase | Ganancia estimada | Esfuerzo | Estado |
|-----------|----------|-------------------|----------|--------|
| 0 | **16.0 — Fix release hang** | 🔴 HABILITADOR | 1-3 días | 🔜 |
| 1 | **16.2 — inbounds en GEPs** | 2-3× (release) | 1 día | 🆕 Pendiente |
| 2 | **16.3 — readonly/readnone** | 1.5-3× (release) | ✅ HECHO | ✅ **COMPLETADO** |
| 3 | **16.4 — noalias en parámetros** | 1.5-3× (release) | 1 día | 🆕 Pendiente |
| 4 | **16.9 — TBAA metadata** | 1.5-2× (release) | 2 días | 🆕 Pendiente |
| 5 | **16.5 — align explícito** | 1.1-1.5× (release) | 1 día | 🆕 Pendiente |
| 6 | **16.6 — noundef** | 1.1-1.3× (release) | 0.5 días | 🆕 Pendiente |
| 7 | **16.7 — !range metadata** | 1.1-1.3× (release) | 0.5 días | 🆕 Pendiente |
| 8 | **16.8 — lifetime markers** | 1.05-1.1× (release) | 1 día | 🆕 Pendiente |
| — | **16.1 — nsw/nuw** | DIFERIDO | Requiere range analysis | ⏳ |

**Cambios respecto a la versión anterior:**
1. 16.0 (Fix release hang) es ahora la PRIORIDAD #0 — bloquea todo lo demás
2. 16.2 (inbounds) sube a #1 porque podría resolver el release hang
3. 16.3 (readonly/readnone) ya está HECHO ✅
4. 16.1 (nsw/nuw) se DIFIERE porque requiere range analysis preventivo
5. 16.9 (TBAA) baja de prioridad porque sin release mode funcional no se puede medir su impacto

---

### Fase 17 — Zero-Cost Abstractions 🔜 PLANIFICADO

**Objetivo:** Garantizar que las construcciones de alto nivel (clases, genéricos,
iteradores, closures) tengan CERO sobrecarga en tiempo de ejecución.

| # | Tarea | Prioridad | Depende de |
|---|-------|-----------|------------|
| 17.1 | Stack allocation para `final class` pequeños (hoy heap) | ⭐⭐⭐⭐ | — |
| 17.2 | Inlining completo de `.map()`/`.filter()`/`.fold()` en código máquina | ⭐⭐⭐ | Fase 15 |
| 17.3 | Monomorfización de genéricos verificada en LLVM IR | ⭐⭐⭐ | — |
| 17.4 | Eliminación de vtables para clases sin herencia | ⭐⭐⭐ | — |
| 17.5 | Devirtualización de llamadas a métodos (speculative devirt) | ⭐⭐ | Fase 14 |

---

### Fase 18 — Memory & Stack Optimizations 🔜 PLANIFICADO

**Objetivo:** Minimizar asignaciones en heap y maximizar uso de pila (stack) para
tipos pequeños, siguiendo la filosofía C++ de "zero-cost abstractions".

| # | Tarea | Prioridad | Depende de |
|---|-------|-----------|------------|
| 18.1 | Escape analysis: `final class` en stack si no escapa | ⭐⭐⭐⭐ | — |
| 18.2 | Small string optimization (SSO): strings < 15 bytes inlineados | ⭐⭐⭐ | — |
| 18.3 | Array optimizations: small arrays en stack | ⭐⭐⭐ | 18.1 |
| 18.4 | Return value optimization (RVO) para clases | ⭐⭐ | 18.1 |

---

*Versión: v1.0 · Actualizado: 2026-06-30*
