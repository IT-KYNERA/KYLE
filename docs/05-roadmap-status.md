# Roadmap & Status — v0.5 → v0.6

> **Hoja de ruta oficial del lenguaje Kyle.** Este documento es la única fuente
> de verdad sobre el estado actual del proyecto (v0.5), las prioridades de
> implementación y el checklist para v0.6. Las decisiones aquí reflejadas son
> **firmes** y no cambiarán sin un major version bump.
>
> **Objetivo de rendimiento:** Kyle debe competir con C, C++ y Rust en velocidad
> de ejecución. Hoy está a ~2.6-100× de Rust (según carga). La causa raíz es que
> el backend genera 22+ allocas por función — cada operación pasa por memoria RAM
> en vez de registros. Las fases 15 (SSA), 16 (atributos LLVM) y 17 (optimization
> pipeline) cierran esta brecha llevando a Kyle a 1-3× de Rust.

---

## 1. Los 5 Pilares Inmutables de Kyle (La Identidad)

Cada decisión de diseño debe alinearse con estos principios. Si no los cumple,
**no entra al lenguaje**.

| Pilar | Definición |
| :--- | :--- |
| **1. Legibilidad Extrema** | El código se lee como prosa. SIN `;`, SIN `{}`, SIN `let`/`var`/`const`. Solo indentación (4 espacios) y asignación directa. |
| **2. Tipado Fuerte con Inferencia** | El compilador conoce todos los tipos en compile-time. El programador escribe el mínimo indispensable. |
| **3. Simplicidad Radical** | Una única forma de hacer cada cosa. Sin excepciones. `for`, `while`, `loop`. |
| **4. Rendimiento Zero-Cost** | Borrow semantics por defecto, ownership via `^`. Sin GC. Sin refcounting implícito. `Rc`/`Arc` en stdlib. |
| **5. Coherencia Sintáctica** | Lo que parece igual se comporta igual. `T?` reemplaza `Option<T>`. `final class` reemplaza `struct`. |

---

## 2. Decisiones Concretas y Eliminaciones (Frozen)

### 2.1 Variables — NO existe `let`/`var`/`mut`/`const`

| Forma | Sintaxis | Mutabilidad |
| :--- | :--- | :--- |
| Inmutable | `nombre = valor` | ❌ |
| Mutable | `nombre: &T = valor` / `x = &valor` | ✅ (el `&` en tipo o valor) |
| Constante | `NOMBRE := valor` | ❌ (compile-time, mayúsculas por convención) |

> **Nota:** `::=` fue eliminado. Las constantes ahora usan `:=`. Las variables
> mutables usan `&T` en el tipo o `&expr` como sugar. El operador walrus `:=`
> ya no declara variables mutables — solo constantes.

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
Fase 7:    Borrow Semantics      ✅ 13/13 (refactorizado: default = préstamo, no move)
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
                                    ✅ Pipeline SSA activo en release mode
                                    ✅ Benchmarks correctos (debug + release)
                                    ✅ PHI node bug fix — fallback values para predecessors sin entrada
Fase 16:   LLVM IR Quality       ✅ COMPLETADO (excepto 16.1 nsw/nuw diferido)
                                    ✅ 16.0 — Fix release mode hang
                                    ✅ 16.2 — inbounds en GEPs
                                    ✅ 16.3 — readonly/readnone
                                    ✅ 16.4 — noalias en parámetros
                                    ✅ 16.5 — Align en allocas
                                    ✅ 16.6 — noundef en parámetros
                                    ✅ 16.7 — !range metadata en bool
                                    ✅ 16.8 — lifetime.start/end
                                    ✅ 16.9 — TBAA metadata
                                    🔶 16.1 — nsw/nuw flags (implementado vía build_int_nsw_add,
                                    pero no se reflejan en el IR generado — bug de inkwell/codegen)
Fase 11:   Package Manager      ✅ COMPLETADO (resolver, registry, cache, publish, login, update, outdated, import)
  Fase 12:   Tooling              ✅ COMPLETADO (LSP, VS Code ext, test framework, formatter, completions, debug adapter, color theme)
Fase 13:   Sintaxis Restante    🔜 EN CURSO (rangos, is, for-else, static fn, **, +% ✅ — falta genéricos, ptr, op overload, etc.)
Fase 14:   References & Borrow Checker ✅ COMPLETADO
Fase 17:   Optimization Pipeline 🔜 v0.6 — Ejecutar pases LLVM (mem2reg,
             gvn, licm, sccp) en el módulo antes de emitir código.
             Objetivo: cerrar el gap de rendimiento con Rust.
Fase 17:   Optimization Pipeline 🔜 v0.6 — cerrar gap rendimiento
Fase 18:   Zero-Cost Abstractions 📅 (post-v1.0)
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

- `Walrus` (`:=` para constantes, reemplaza `::=`) · `Abstract` · `Final`
- `At` (`@`) · `DotDotEquals` (`..=`) · `DotDotLess` (`..<`)
- `mut` keyword eliminado
- `##` doc comments

---

### Fase 4: Parser ✅

- `name = expr` · `name: &T = expr` · `NAME := expr` (constante)
- `final class Name:` · `abstract class Name:` · `struct` alias temporal
- `T?` / `T!` postfix types · `&T` mutable type · `^T` move type
- Destructuring `(x, y) = expr`
- `if nombre = expr` (BindingIf) · `while nombre = expr` (WhileBind)
- Error recovery (modo pánico)

---

### Fase 5: HIR + Desugaring ✅

- Crate `kyc_hir` creado
- Desugaring: `T?` → `Option<T>`, `T!` → `Result<T, str>`
- Integrado en pipeline (`kyc_driver`)

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
- Constant checking (`NAME := expr`)
- Abstract method enforcement
- Match guard MIR lowering (Fase 6 crossover con MIR)
- Default params type checking

**Pendiente:**
- [x] ⭐⭐⭐⭐ **Or-patterns (`a | b`)** — AST + Parser + Type Checker + MIR lowering
- [x] ⭐⭐⭐ **Properties (get/set)** — parser completo + MIR field routing (backing field `_name`)
- [x] ⭐⭐⭐ **Default params MIR lowerer** — sustituir defaults en call-sites

---

### Fase 7: Borrow Semantics ✅ (13/13, refactorizado)

El comportamiento por defecto de los parámetros cambió de **move** a
**préstamo inmutable** (`s: T` → borrowed, no moved). Ahora:

- `s: T` — préstamo inmutable (default)
- `s: &T` — préstamo mutable
- `^s: T` — ownership transfer (move explícito)

**Completado (fase original):**
- Copy vs Move type classification
- Replace `ownership` (refcounting) pass → `move_analysis.rs`
- Forward dataflow analysis con intersección en joins
- Use-after-move detection
- `.clone()` para Str/List/Dict (runtime `ky_clone_*`)
- Heap-allocation de string literals
- Borrowing functions (`print`, `println`, `strlen`, etc.)
- Pipeline integrado: build falla en use-after-move
- `ownership.rs` eliminado
- `ky_release` declaration removed from codegen

**Refactorizado para Fase 14:**
- Default parameter: move → borrow inmutable
- `&T` en parámetros = borrow mutable
- `^T` en parámetros = move explícito
- Eliminación de lista blanca de borrowing functions
- Actualización de 9 tests end-to-end

**Pendiente:**
- [x] ⭐⭐⭐ **Memory safety tests** — 9 tests end-to-end en `kyc_driver`

---

### Fase 8: Backend — Release Mode ✅

- [x] ⭐⭐⭐⭐⭐ Conectar `--release` a `OptimizationLevel::Aggressive`
- [x] `ky build --release` y `ky run --release` funcionan correctamente

---

### Fase 9: Async Scheduler 🔜 (thread pool V2 — funcional, faltan optimizaciones)

#### ✅ Completado (Thread Pool V2)

| Feature | Archivo | Detalle |
|---------|---------|---------|
| `async fn name():` parsing | `parser.rs` | ✅ Sintaxis `async fn foo():` parseada |
| `async expr` + `await expr` | `parser.rs`, `codegen.rs` | ✅ Evaluación async y espera de tareas |
| `ky_spawn_task(ptr, i64) -> i64` | `runtime/async_.rs` | ✅ Spawnea tarea en thread pool |
| `ky_await_task(i64) -> i64` | `runtime/async_.rs` | ✅ Bloquea hasta que la tarea termina |
| `ky_yield()` | `runtime/async_.rs` | ✅ Cooperativo: cede el thread temporalmente |
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
| 9.1 | HIR-level: detectar `async fn` y generar tipo enum con variante por yield point | `kyc_hir/src/` | ⭐⭐⭐ |
| 9.2 | MIR-level: transformar `await` en cambio de estado + salto a bloque correcto | `kyc_mir/src/lower.rs` | ⭐⭐⭐ |
| 9.3 | Runtime: `TaskStateMachine` struct con puntero a función + estado actual | `kyc_runtime/src/async_.rs` | ⭐⭐⭐ |
| 9.4 | Codegen: emitir switch-case en el bucle de polling de la state machine | `kyc_backend/src/codegen.rs` | ⭐⭐⭐ |
| 9.5 | Benchmarks: 100K tareas concurrentes sin thread pool | `examples/bench/` | ⭐⭐ |

**Dependencia:** Fase 15 (SSA) — necesario para que la state machine sea eficiente.

#### 🔜 Pendiente — Work-Stealing Scheduler

**Objetivo:** Cuando hay MULTIPLES hilos, balancear carga dinámicamente.

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 9.6 | Cada thread tiene su cola de tareas local (deque) | `kyc_runtime/src/async_.rs` | ⭐⭐⭐ |
| 9.7 | Thread inactivo roba tareas del deque de otro thread | `kyc_runtime/src/async_.rs` | ⭐⭐⭐ |
| 9.8 | `global_queue` como respaldo para tareas nuevas | `kyc_runtime/src/async_.rs` | ⭐⭐ |

**Dependencia:** State Machine V3 (9.1-9.5) — work-stealing solo tiene sentido con state machines.

#### 🔜 Pendiente — Non-blocking I/O

**Objetivo:** `ky_open()`, `ky_read_str()`, `ky_write_str()` sin bloqueo.

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 9.9 | I/O ring / epoll/kqueue integration | `kyc_runtime/src/io.rs` | ⭐⭐ |
| 9.10 | `async fn read_file()` → no bloquea el thread | `kyc_runtime/src/io.rs` | ⭐⭐ |
| 9.11 | Socket API async (`async fn connect()`, `async fn accept()`) | nuevo crate | ⭐⭐ |

**Dependencia:** Work-Stealing (9.6-9.8) — I/O async necesita un scheduler que no bloquee hilos.

---

### Fase 10: Iteradores 🔜 (runtime listo, falta lazy evaluation + closures)

#### ✅ Completado — Métodos de Agregación (runtime C-ABI + lowering)

| Feature | Runtime | MIR Lowering | LLVM Codegen | Estado |
|---------|---------|--------------|--------------|--------|
| `items.sum()` | `ky_list_sum` | `lower.rs` | `codegen.rs` | ✅ |
| `items.product()` | `ky_list_product` | `lower.rs` | `codegen.rs` | ✅ |
| `items.max()` | `ky_list_max` | `lower.rs` | `codegen.rs` | ✅ |
| `items.min()` | `ky_list_min` | `lower.rs` | `codegen.rs` | ✅ |
| `items.reverse()` | `ky_list_reverse` | `lower.rs` | `codegen.rs` | ✅ |
| `items.len()` | `ky_list_len` | `lower.rs` | `codegen.rs` | ✅ |
| `items.add()` | `ky_list_push` | `lower.rs` | `codegen.rs` | ✅ |
| `items.pop()` | `ky_list_pop` | `lower.rs` | `codegen.rs` | ✅ |
| `items.contains()` | `ky_list_contains` | `lower.rs` | `codegen.rs` | ✅ |
| `items.insert()` | `ky_list_insert` | `lower.rs` | `codegen.rs` | ✅ |
| `items.remove_at()` | `ky_list_remove_at` | `lower.rs` | `codegen.rs` | ✅ |
| `items.clear()` | `ky_list_clear` | `lower.rs` | `codegen.rs` | ✅ |
| `items.clone()` | `ky_clone_list` | `lower.rs` | `codegen.rs` | ✅ |
| `items.map(fn)` | `ky_list_map` | `lower.rs` | `codegen.rs` | ✅ (vía fn ptr C-ABI) |
| `items.filter(fn)` | `ky_list_filter` | `lower.rs` | `codegen.rs` | ✅ (vía fn ptr C-ABI) |
| `items.fold(init, fn)` | `ky_list_fold` | `lower.rs` | `codegen.rs` | ✅ (vía fn ptr C-ABI) |
| `items.reduce(fn)` | `ky_list_reduce` | `lower.rs` | `codegen.rs` | ✅ (vía fn ptr C-ABI) |

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
| 10.6 | `Iter[T]` struct en runtime con next(), map(), filter(), fold() | `kyc_runtime/src/list.rs` | ⭐⭐ |
| 10.7 | `items.iter()` → `Iter[T]` | `lower.rs`, `codegen.rs` | ⭐⭐ |
| 10.8 | Chain `.map().filter()` sin listas intermedias | `runtime/list.rs` | ⭐⭐ |
| 10.9 | Lazy fold/reduce sobre iterador | `runtime/list.rs` | ⭐⭐ |

---

### Fase 11: Package Manager ✅ (registry, resolución, publicación)

**Filosofía:** Tan simple como Cargo pero SIN la complejidad de workspaces multi-crate
y SIN build scripts. Un package = un proyecto. Una dependencia = una línea en `ky.toml`.

**Estado general:** ✅ 11.1-11.5 implementados. Foundation completa: resolver, registry client,
cache, lock file, CLI commands, import resolution from cached packages.

---

#### 11.1 — Manifest (`ky.toml`) completo ✅

**Estado actual:** ✅ `ky.toml` soporta formato plano y `[project]` table. Validación completa
con mensajes de error con sugerencias. `dev-dependencies` parseadas. Main field usado.

| # | Tarea | Archivo | Estado |
|---|-------|---------|--------|
| 11.1.1 | Validación completa de campos `[project]` (name, version, authors, description, license, edition) | `kyc_tools/src/package/manifest.rs` | ✅ |
| 11.1.2 | Validación de versiones semver (major.minor.patch con pre-release opcional) | `kyc_tools/src/package/manifest.rs` | ✅ (vía crate `semver`) |
| 11.1.3 | Campo `[project] main` para entry point distinto de `src/main.ky` | `kyc_tools/src/package/manifest.rs` | ✅ (resolved via `project_main()`) |
| 11.1.4 | Sección `[dev-dependencies]` para dependencias de testing | `kyc_tools/src/package/manifest.rs` | ✅ |
| 11.1.5 | Sección `[target]` para configuraciones por plataforma (opcional) | `kyc_tools/src/package/manifest.rs` | ⏳ (baja prioridad) |
| 11.1.6 | Error claro y con sugerencias si `ky.toml` falta o está mal formado | `kyc_tools/src/package/manifest.rs` | ✅ |

**Ejemplo de `ky.toml` (ambos formatos soportados):**

Formato `[project]` (nuevo, recomendado):
```toml
[project]
name = "myapp"
version = "0.1.0"
authors = ["Tu Nombre"]
description = "Mi aplicación Kyle"
license = "MIT"
edition = "2024"
main = "src/main.ky"

[compiler]
optimization = "O2"
target = "native"

[dependencies]
math = "1.0.0"
json = "2.1.0"

[dev-dependencies]
testing = "1.0.0"
```

Formato plano (backward compatible):
```toml
name = "myapp"
version = "0.1.0"
edition = "2024"

[compiler]
optimization = "O2"
target = "native"

[dependencies]
math = "1.0.0"
```

---

#### 11.2 — Resolución de dependencias (semver) ✅

**Estado actual:** ✅ Resolución semver completa con algoritmo greedy, resolución transitiva,
detección de conflictos, y orden topológico. Integrado con registry + cache + lock file.

| # | Tarea | Archivo | Estado |
|---|-------|---------|--------|
| 11.2.1 | Parseo completo de semver: `^1.2.3`, `~1.2`, `>=1.0 <2.0`, `*`, `1.x` | `kyc_core/src/semver.rs` (vía crate `semver`) | ✅ |
| 11.2.2 | Comparación de versiones (major.minor.patch.pre) | `kyc_core/src/semver.rs` | ✅ |
| 11.2.3 | Algoritmo de resolución: greedy (elige la mayor compatible) | `kyc_core/src/resolver.rs` | ✅ |
| 11.2.4 | Generación de `ky.lock` con versiones resueltas + orden topológico | `kyc_tools/src/package/lock.rs` | ✅ |
| 11.2.5 | Cache de dependencias descargadas en `~/.ky/cache/` | `kyc_tools/src/package/cache.rs` | ✅ |
| 11.2.6 | `ky update` para actualizar `ky.lock` a últimas versiones compatibles | `kyc_cli/src/main.rs` | ✅ |
| 11.2.7 | `ky outdated` para listar dependencias desactualizadas | `kyc_cli/src/main.rs` | ✅ |
| 11.2.8 | Resolución de dependencias transitivas (dep de dep) | `kyc_core/src/resolver.rs` | ✅ |

**Algoritmo de resolución implementado:**
```
1. Leer ky.toml
2. Para cada dep en [dependencies]:
   a. Consultar registry (RegistryBackend trait) por todas las versiones del paquete
   b. Filtrar por la restricción semver
   c. Elegir la versión más alta que cumpla
   d. Obtener dependencias transitivas del registry
   e. Repetir recursivamente (con detección de ciclos y profundidad máxima)
3. Si hay conflictos → error claro
4. Escribir ky.lock con orden topológico
5. Descargar tarballs faltantes al cache
```

---

#### 11.3 — Registry (cliente HTTP) ✅

**Estado actual:** ✅ Cliente HTTP+JSON para registry REST API. Cache local.
File registry para testing offline.

| # | Tarea | Archivo | Estado |
|---|-------|---------|--------|
| 11.3.1 | Diseñar estructura del registry (API HTTP REST) | — | ✅ (API definida en `registry.rs`) |
| 11.3.2 | Cliente HTTP para consultar registry (via `ureq`) | `kyc_tools/src/package/registry.rs` | ✅ |
| 11.3.3 | Descarga y extracción de paquetes (.tar.gz) | `kyc_tools/src/package/registry.rs` + `cache.rs` | ✅ |
| 11.3.4 | Cache local en `~/.ky/cache/<pkg>-<ver>/` | `kyc_tools/src/package/cache.rs` | ✅ |
| 11.3.5 | `ky publish` — empaquetar y subir paquete al registry | `kyc_cli/src/main.rs` | ✅ |
| 11.3.6 | `ky login` — autenticación con API key | `kyc_cli/src/main.rs` | ✅ |
| 11.3.7 | Verificación de integridad (SHA256 checksums) | `kyc_tools/src/package/cache.rs` | ✅ |
| 11.3.8 | File-based registry para testing offline | `kyc_tools/src/package/registry.rs` | ✅ |

**API del registry (esperada por el cliente):**
```
GET  /v1/packages/:name          → { versions: [{ version, yanked }] }
GET  /v1/packages/:name/:ver/dependencies → { dependencies: [{ name, version }] }
GET  /v1/packages/:name/:ver/download     → binary .tar.gz
GET  /v1/packages/:name/:ver/ky.toml      → raw ky.toml
PUT  /v1/packages/:name/:ver/upload       ← binary .tar.gz (para publish)
GET  /v1/auth/verify                      → 200 OK (con Bearer token)
```

---

#### 11.4 — Importación desde paquetes ✅

**Estado actual:** ✅ El pipeline resuelve imports desde caché de paquetes
automáticamente. `resolve_imports()` en pipeline.rs agrega search paths desde
`ky.lock` y desde `~/.ky/cache/`. Orden de resolución implementado.

| # | Tarea | Archivo | Estado |
|---|-------|---------|--------|
| 11.4.1 | `import math` busca primero en paquetes instalados, luego en locales | `kyc_driver/src/pipeline.rs` | ✅ |
| 11.4.2 | `import mypkg.str` — importar submódulo de un paquete | `kyc_frontend/src/parser.rs` | ✅ (existente) |
| 11.4.3 | Resolver `import json` a `~/.ky/cache/json-2.1.0/src/lib.ky` | `kyc_driver/src/pipeline.rs` | ✅ |
| 11.4.4 | Compilar dependencias ANTES que el proyecto principal | `kyc_driver/src/pipeline.rs` | ✅ (vía search paths) |
| 11.4.5 | Cache de compilación: no recompilar dependencias si no cambiaron | `kyc_driver/src/pipeline.rs` | ⏳ (futuro) |

**Orden de resolución de imports:**
```
import math → busca en:
  1. Paquetes instalados en ~/.ky/cache/math-*/src/ (desde ky.lock + cache scan)
  2. Directorio del archivo actual
  3. src/ del proyecto
  4. std/ (librería estándar)
```

---

#### 11.5 — Comandos del package manager ✅

| Comando | Estado | Detalle |
|---------|--------|---------|
| `ky new <name>` | ✅ | Template con `[project]` table, src/, tests/, .vscode/ |
| `ky add <dep>@<ver>` | ✅ | Modifica ky.toml + resuelve + descarga inmediatamente |
| `ky remove <dep>` | ✅ | Modifica ky.toml |
| `ky build` | ✅ | Resuelve deps + descarga + compila |
| `ky run` | ✅ | Resuelve deps + descarga + compila + ejecuta |
| `ky check` | ✅ | Resuelve deps antes de type-check |
| `ky test` | ✅ | Resuelve deps antes de testear |
| `ky info` | ✅ | Muestra metadata + lock info + cache status |
| `ky publish` | ✅ | Empaqueta .tar.gz + sube al registry |
| `ky login` | ✅ | Verifica API key + guarda en ~/.ky/config.toml |
| `ky update` | ✅ | Re-resuelve y actualiza ky.lock |
| `ky outdated` | ✅ | Compara lock vs registry, lista desactualizadas |
| `ky doc` | ❌ | Futuro (Fase 12) |

#### Lo que falta de Phase 11 (no implementado, fuera del alcance del compilador)

| Item | Razón |
|------|-------|
| `[target]` section en manifest | Baja prioridad, opcional para cross-compilación |
| Yank de versiones (server-side) | Es responsabilidad del servidor registry, no del cliente |
| Registry server HTTP | El cliente está listo, el server es un proyecto aparte |
| Cache de compilación incremental | Optimización futura (no recompilar deps si no cambiaron) |

---

### Fase 12: Tooling 🔜 (LSP, VS Code, tests, formatter)

**Filosofía:** Kyle debe tener un entorno de desarrollo moderno desde el día 1.
Esto significa: tests integrados, formatter, LSP con autocompletado real,
y extensión de VS Code que funcione out-of-the-box.

---

#### 12.1 — Test Framework

**Estado actual:** 🔶 `ky test` existe pero solo type-checkea archivos en `tests/`.

**Objetivo:** Sistema de testing integrado como Rust, pero más simple:
sin macros de procedimiento, sin `#[should_panic]`, sin fixtures complejas.

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 12.1.1 | Parser: `#[test]` attribute antes de `fn` | `kyc_frontend/src/parser.rs` | ✅ |
| 12.1.2 | `#[test]` fn debe: no tener parámetros, retornar `void` o `i32` | `kyc_semantic/src/type_checker.rs` | ✅ |
| 12.1.3 | `ky test` compila y ejecuta cada `#[test]` fn individualmente | `kyc_cli/src/main.rs` | ✅ |
| 12.1.4 | Reporte de resultados: `PASS`, `FAIL`, total, tiempo | `kyc_cli/src/main.rs` | ✅ |
| 12.1.5 | `assert(cond)`, `assert_eq(a, b)`, `assert_ne(a, b)` como builtins | `kyc_runtime/src/lib.rs` | ✅ |
| 12.1.6 | `assert_throws(fn, expected_error)` para testear errores | `kyc_runtime/src/lib.rs` | ⭐⭐ |
| 12.1.7 | `#[test] ignore` para saltar tests | `kyc_frontend/src/parser.rs` | ⭐⭐ |
| 12.1.8 | `ky test <filtro>` para ejecutar solo tests que coincidan | `kyc_cli/src/main.rs` | ⭐⭐ |
| 12.1.9 | Test con salida: capturar `print()` durante tests | `kyc_cli/src/main.rs` | ⭐⭐ |

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

**Estado actual:** ✅ `ky lsp` implementado completo. Tiene diagnósticos incrementales,
autocompletado con builtins+símbolos+keywords, dot completions contextual,
hover con docs, go-to-definition, find references, document symbols,
signature help, code actions, formatting, rename, semantic tokens.

**Lo que falta:**

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 12.2.1 | **Diagnósticos en tiempo real**: errores de sintaxis y tipo mientras se escribe | `kyc_tools/src/lsp.rs` | ✅ |
| 12.2.2 | **Diagnósticos incrementales**: solo re-analizar archivo modificado, no todo el proyecto | `kyc_tools/src/lsp.rs` | ✅ |
| 12.2.3 | **Autocompletado completo**: builtins, símbolos del proyecto, keywords actualizados | `kyc_tools/src/lsp.rs` | ✅ |
| 12.2.4 | **Autocompletado contextual**: dot completions con tipos conocidos (struct/class/enum) | `kyc_tools/src/lsp.rs` | ✅ |
| 12.2.5 | **Go-to-definition mejorado**: saltar a definición de función/clase en archivos del proyecto | `kyc_tools/src/lsp.rs` | ✅ |
| 12.2.6 | **Go-to-definition en dependencias**: saltar a definición dentro de paquetes instalados | `kyc_tools/src/lsp.rs` | ⭐⭐ |
| 12.2.7 | **Find references**: encontrar todas las referencias a un símbolo | `kyc_tools/src/lsp.rs` | ✅ |
| 12.2.8 | **Hover mejorado**: mostrar documentación de `##` comments + tipo inferido | `kyc_tools/src/lsp.rs` | ✅ |
| 12.2.9 | **Code actions**: sugerencias automáticas (ej: "añadir import faltante") | `kyc_tools/src/lsp.rs` | ✅ |
| 12.2.10 | **Document symbols**: lista de funciones/clases en el archivo actual | `kyc_tools/src/lsp.rs` | ✅ |
| 12.2.11 | **Rename symbol**: refactorización segura (F2) | `kyc_tools/src/lsp.rs` | ✅ |
| 12.2.12 | **Format on save**: ejecutar `ky fmt` al guardar | `kyc_tools/src/lsp.rs` | ✅ |
| 12.2.13 | **Inlay hints**: mostrar tipos inferidos en variables | `kyc_tools/src/lsp.rs` | ✅ |
| 12.2.14 | **Diagnósticos en `ky.toml`**: validar el manifest | `kyc_tools/src/lsp.rs` | ⭐⭐ |
| 12.2.15 | **Code lens**: "Run test" button encima de `#[test]` fn | `kyc_tools/src/lsp.rs` + `extension/src/extension.ts` | ✅ |

---

#### 12.3 — VS Code Extension

**Estado actual:** ✅ Completa — syntax highlighting, LSP, snippets, testing UI, debug adapter, tasks, problems panel, color theme, packaging.

**Lo que falta:**

| # | Tarea | Archivo | Prioridad |
|---|-------|---------|-----------|
| 12.3.1 | **Syntax highlighting completo**: resaltar toda la sintaxis Kyle correctamente | `extension/syntaxes/kl.tmLanguage.json` | ✅ |
| 12.3.2 | **Icono de lenguaje**: icono para archivos `.ky` | `vscode-ky/icons/` | ⭐⭐ | ✅ |
| 12.3.3 | **Task provider**: botones "Run", "Build", "Test" en la barra de estado | `vscode-ky/src/extension.ts` | ✅ | ✅ |
| 12.3.4 | **Problemas en tiempo real**: mostrar errores del LSP en el panel de problemas | `vscode-ky/src/extension.ts` | ✅ | ✅ |
| 12.3.5 | **Snippets actualizados**: snippets para toda la sintaxis moderna | `vscode-ky/snippets/kl.json` | ✅ | ✅ |
| 12.3.6 | **Debug adapter**: DAP server para step-through debugging | `vscode-ky/src/debugger.ts` | ⭐ | ✅ |
| 12.3.7 | **Testing UI**: mostrar tests en el panel de Testing de VS Code | `vscode-ky/src/testUI.ts` | ✅ | ✅ |
| 12.3.8 | **Extension packaging**: script para generar `.vsix` automáticamente | `vscode-ky/scripts/build-extension.sh` | ⭐⭐ | ✅ |
| 12.3.9 | **Marketplace metadata**: README, CHANGELOG, icono | `vscode-ky/` | ⭐⭐ | ✅ |
| 12.3.10 | **Tema de color Kyle**: "Kyle Pastel" dark theme | `vscode-ky/themes/kl-color-theme.json` | ⭐ | ✅ |

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

#### 12.4 — Formatter (`ky fmt`)

**Estado actual:** ✅ Formateador completo con sintaxis moderna y configuración.

| # | Tarea | Archivo | Prioridad | Estado |
|---|-------|---------|-----------|--------|
| 12.4.1 | Formatear toda la sintaxis moderna: `:=`, `&T`, `^T`, `T?`, `final class`, etc. | `kyc_tools/src/formatter.rs` | ⭐⭐⭐ | ✅ |
| 12.4.2 | Formatear patterns de match (or-patterns, guards, destructuring) | `kyc_tools/src/formatter.rs` | ⭐⭐ | ✅ |
| 12.4.3 | Formatear closures inline `(x) => x * 2` | `kyc_tools/src/formatter.rs` | ⭐⭐ | ✅ |
| 12.4.4 | Formatear imports (orden alfabético, agrupados) | `kyc_tools/src/formatter.rs` | ⭐⭐ | ✅ |
| 12.4.5 | `ky fmt --check` + project mode (`ky fmt` sin args) | `kyc_cli/src/main.rs` | ⭐⭐ | ✅ |
| 12.4.6 | Configuración de formato en `ky.toml` (`[format]` section) | `kyc_tools/src/package/manifest.rs` | ⭐⭐ | ✅ |

**Reglas de formato (v1.0):**
- Indentación: 4 espacios (obligatorio)
- Longitud máxima de línea: 100 caracteres (configurable)
- Saltos de línea después de `:` en headers (fn, class, if, while, for, match)
- Un solo espacio después de `#` en comentarios
- Sin espacios al final de línea
- Un salto de línea al final del archivo

---

#### 12.5 — Shell Completions

**Estado actual:** ✅ Completions para bash, zsh, fish, y powershell.

| # | Tarea | Archivo | Prioridad | Estado |
|---|-------|---------|-----------|--------|
| 12.5.1 | `ky completions zsh` | `kyc_cli/src/main.rs` | ⭐⭐ | ✅ |
| 12.5.2 | `ky completions fish` | `kyc_cli/src/main.rs` | ⭐⭐ | ✅ |
| 12.5.3 | `ky completions powershell` | `kyc_cli/src/main.rs` | ⭐ | ✅ |
| 12.5.4 | Autocompletado de nombres de dependencias en `ky add` | `kyc_cli/src/main.rs` | ⭐⭐ | ✅ |

---

### Fase 13: Sintaxis Restante 🔜 (características del lenguaje que faltan)

**Objetivo:** Implementar toda la sintaxis documentada que aún no funciona.

| # | Tarea | Sintaxis | Prioridad | Estado |
|:--|:------|:---------|:----------|:-------|
| 13.1 | **Genéricos en clases**: `final class Stack<T>:` | `final class Nombre<T>:` | ⭐⭐⭐⭐⭐ | 🔶 Parsing parcial, falta monomorphization |
| 13.2 | **Rangos completos**: `0..=5`, `0..<5` | `..=`, `..<` como operadores | ⭐⭐⭐ | ✅ |
| 13.3 | **`is` type checking**: `x is str` → true/false | `expr is Type` | ⭐⭐⭐ | ✅ |
| 13.4 | **`ptr` type completo**: aritmética de punteros para FFI | `ptr` como tipo usable | ⭐⭐ | ❌ |
| 13.5 | **`null` literal**: valor nulo para `ptr` | `null` | ⭐⭐ | ❌ |
| 13.6 | **Operator overloading**: `op_+(other)`, `op_-(other)`, `op_*(other)` | `fn op_+(other: T) T:` | ⭐⭐ | ❌ |
| 13.7 | **`for-else:`**: bloque else si loop no hizo break | `for x in items: ... else: ...` | ⭐⭐ | ✅ |
| 13.8 | **Loop labels**: `break <label>`, `continue <label>` | `label for ...` / `label while ...` | ⭐⭐ | ✅ Decisión cerrada |
| 13.9 | **Match destructuring**: `match pair: (x, y) => ...` | patterns en match | ⭐⭐ | ✅ Or-patterns ya funciona |
| 13.10 | **Match guard**: `match x: n if n > 0 => ...` | guard condicional | ⭐⭐ | ✅ |
| 13.11 | **Enum methods**: `fn name():` dentro de `enum` | métodos en enum | ⭐⭐ | ❌ |
| 13.12 | **`super.method()`**: llamar método padre sobreescrito | `super.nombre()` | ⭐⭐ | 🔶 Parcial |
| 13.13 | **`static fn`**: métodos estáticos en clases | `static fn name():` | ⭐⭐ | ✅ |
| 13.14 | **`abstract fn`**: funciones abstractas en abstract class | `abstract fn name():` | ⭐⭐ | ❌ |
| 13.15 | **`@` attribute token**: `#[attr]` sintaxis completa | `@` como token + parsing | ⭐⭐ | ❌ |
| 13.16 | **`??` default operator**: `expr ?? default` | `??` para T? | ⭐ | 🔶 Lexer/Parser/AST, MIR stub |
| 13.17 | **`**` power operator correcto**: codegen real (hoy es mul incorrecto) | `a ** b` | ⭐⭐ | ✅ |
| 13.18 | **`+%`, `-%`, `*%` percentage ops**: via ky_add_pct/ky_sub_pct/ky_mul_pct | `x +% 10` | ⭐ | ✅ |

**NOTA:** Or-patterns (`a | b`), Properties (get/set), y Default params ya están ✅ implementados.

---

### Fase 14: References & Borrow Checker ✅ COMPLETADO — &T codegen, mutable fields, field defaults, borrow whitelist eliminada, region inference no aplica

**Objetivo:** Reemplazar la lista blanca de borrowing functions con un
sistema completo de referencias con préstamo por defecto.

**Nueva sintaxis:**

| Concepto | Sintaxis | Ejemplo |
|----------|----------|---------|
| Préstamo inmutable | `s: T` | `fn read(s: str)` |
| Préstamo mutable | `s: &T` | `fn append(s: &str)` |
| Move (ownership) | `^s: T` | `fn consume(^s: str)` |
| Variable inmutable | `x = expr` | `name = "Ana"` |
| Variable mutable | `x: &T = expr` / `x = &expr` | `age: &i32 = 25` |
| Constante | `X := expr` | `VERSION := "1.0"` |
| Campo inmutable | `name: T` | `field: str` |
| Campo mutable | `name: &T` | `field: &i32` |
| Call site (mutación) | `f(&x)` | `append(&name)` |
| Call site (move) | `f(^x)` | `consume(^name)` |

**Items de implementación:**

- [x] `&T` como tipo mutable en variables, campos, y parámetros
- [x] `^T` como tipo de ownership transfer en parámetros
- [x] Refactor: default de parámetros de move → borrow inmutable
- [x] `&expr` como sugar para mutabilidad en declaración
- [x] Reglas de borrowing en call site (`f(&x)` coercion + codegen)
- [x] Renombrar `move_analysis.rs` → `borrow_analysis.rs`
- [x] `&T` paso por referencia en codegen (puntero, no valor) ← NUEVO
- [x] Borrow tracking básico (is_alive check en MutableBorrow params) ← NUEVO
- [x] ⭐ Mutable fields: `name: &type` en clases  ✅ (type checker rejects assign to immutable fields, codegen handles uniformly)
- [x] ⭐ Field defaults con `=`: `name: &type = value`  ✅ (type checker validates types, MIR StructLiteral lowering applies defaults for missing fields)
- [x] ⭐ Eliminar lista blanca de borrowing functions  ✅ (data-driven via func_map; unknown functions = borrow-by-default)
- [x] ⭐ Inferencia de regiones (sin anotaciones de lifetime)  ✅ (no aplica — `&T` params son call-by-reference sincrónico, borrow analysis cubre safety; no hay tipos referencia persistente ni return references)

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
| **Baja** | ⭐ | Mejoras a largo plazo (backends alternativos). |

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
| `name: &T = value` / `x = &value` mutable | 1-2 | ✅ |
| `name := value` constante | 1-2 | ✅ |
| `T?` optional type | 1-2 | ✅ |
| `T!` error-returning type | 1-2 | ✅ |
| `final class` | 3-4 | ✅ |
| `abstract class` | 3-4 | ✅ |
| `if nombre = expr` (BindingIf) | 3-4 | ✅ |
| `while nombre = expr` (WhileBind) | 3-4 | ✅ |
| Destructuring `(x, y) = expr` | 3-4 | ✅ |
| Error recovery (parser) | 3-4 | ✅ |
| HIR + Desugaring (`T?` → `Option<T>`) | 5 | ✅ |
| Constant evaluation (`:=` at module scope) | 6 | ✅ |
| Abstract method enforcement | 6 | ✅ |
| Match guard (MIR lowering) | 6 | ✅ |
| Default params type-checking | 6 | ✅ |
| Borrow semantics (dataflow, borrow-by-default, use-after-move, clone) | 7 | ✅ 13/13 |
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
| | | **⚠️ Bug conocido: missing extern decls (ky_list_pop_first, etc.)** |
| **LLVM IR Quality** | **16** | **✅ COMPLETADO (excepto 16.1 nsw/nuw diferido)** |
| | | **✅ 16.3 readonly/readnone en runtime externs (memory("read")/memory("none"))** |
| | | **✅ 16.0 Fix release mode hang** |
| | | **✅ 16.2 inbounds en GEPs** |
| | | **✅ 16.4 noalias en parámetros** |
| | | **✅ 16.9 TBAA metadata** |
| | | **✅ 16.5-16.8 align/noundef/!range/lifetime** |
| | | **🔶 16.1 nsw/nuw (code implementado, flags no aparecen en IR)** |
| **Package manager** | **11** | **✅ 11.1-11.5 (resolver, registry client, cache, lock, publish, login, update, outdated, import desde paquetes)** |
| **Tooling** | **12** | **✅ COMPLETADO (LSP, VS Code, test framework, formatter, completions, debug adapter, color theme)** |
| **Sintaxis Restante** | **13** | **🔜 DETALLADO (genéricos, rangos, is, ptr, etc.)** |
| **References & Borrow Checker** | **14** | **✅ COMPLETADO** |
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
    └── 0001-move-semantics.md    # ✅ RFC de borrow semantics (refactorizado)
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
- [x] Walrus (`:=` para constantes), Abstract, Final, DotDotEquals, DotDotLess tokens
- [x] `=`, `: &T =`, `NAME :=` a nivel declaración y statement
- [x] `&T` mutable type, `^T` move type
- [x] `abstract class`, `final class`
- [x] `T?`, `T!` postfix
- [x] Destructuring, BindingIf, WhileBind
- [x] Error recovery
- [ ] `@` (At) token para atributos
- [ ] Todos los ejemplos `.ky` reescritos con nueva sintaxis

### HIR
- [x] Crate `kyc_hir` creado
- [x] Desugaring `T?` → `Option<T>`, `T!` → `Result<T, str>`
- [x] Integrado en pipeline

### Semantic Analysis
- [x] `T?`, `:=`, destructuring, BindingIf/WhileBind type-checking
- [x] Return, Constant, Class/AbstractClass type-checking
- [x] Constant evaluation checking (`NAME := expr`)
- [x] `&T` mutable type checking
- [x] `^T` move type checking
- [x] Abstract method enforcement
- [x] Match guard lowering
- [x] Default params type-checking
- [x] Or-patterns (`a | b`)
- [x] Properties (get/set) — MIR lowering
- [x] Default params MIR lowerer

### Borrow Semantics (Fase 7 + 14)
- [x] Copy/Move classification
- [x] Use-after-move detection
- [x] Dataflow analysis (forward, intersection at joins)
- [x] `.clone()` para Str/List/Dict
- [x] Borrowing functions (print, println, strlen)
- [x] Heap-alloc string literals
- [x] ownership.rs eliminado
- [x] Pipeline integrado
- [x] Memory safety tests automatizados
- [x] **Borrow-by-default** (parámetros `s: T` ya no mueven, prestan)
- [x] `&T` mutable borrow en parámetros (✅ codegen: paso por referencia)
- [x] `^T` move explícito en parámetros
- [x] `&T` en tipos de variables (`name: &str = "x"`)
- [x] `&` sugar en valores (`x = &expr`)
- [x] Mutable fields `name: &type` en clases
- [x] Field defaults `name: &type = value`
- [x] Reglas de borrowing en call site (`f(&x)` coercion, type checker + codegen)
- [x] Eliminar lista blanca de borrowing functions

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

### Fase 16 — LLVM IR Quality ✅ COMPLETADO (excepto 16.1 nsw/nuw diferido)
[x] ✅ 🔴 **16.0 — Fix release mode hang** (PRIORIDAD #0, bloqueante)
- [x] ✅ **16.3 — `readonly`/`readnone` en runtime externs** — `memory("read")` en 13 funciones, `memory("none")` en 7 funciones
[x] ✅ ⭐⭐⭐⭐⭐ **16.2 — `inbounds` en GEPs** (crítico: 2-3×, podría arreglar release hang)
[x] ✅ ⭐⭐⭐⭐ **16.4 — `noalias` en parámetros puntero** (alto: 1.5-3×)
[x] ✅ ⭐⭐⭐ **16.9 — TBAA metadata para alias analysis** (alto: 1.5-2×)
- [x] ✅ **16.5 — `align` explícito en allocas** (set_alignment(8) en todos los allocas)
[x] ✅ ⭐⭐ **16.6 — `noundef` en parámetros** (medio: 1.1-1.3×)
[x] ✅ ⭐⭐ **16.7 — `!range` metadata en bool y tipos acotados** (medio: 1.1-1.3×)
- [x] ✅ **16.8 — `lifetime.start`/`lifetime.end`** (en todos los allocas)
- [x] ✅ **16.1 — `nsw`/`nuw` flags** — implementado vía `build_int_nsw_add/mul/sub`
  y VERIFICADO en IR generado: `add nsw i32 %a, %b` aparece correctamente.

**LOGRO 16.3-16.9 (LLVM IR Quality Completa):** Verificado en IR generado:
```llvm
attributes #0 = { "memory"="read" }   ; 13 funciones readonly
attributes #1 = { "memory"="none" }   ; 7 funciones readnone (pure)
```

### Fase 17 — Optimization Pipeline ✅ COMPLETADO
**Objetivo:** Ejecutar pases de optimización LLVM para cerrar el gap de rendimiento.

| # | Tarea | Impacto | Prioridad | Estado |
|---|-------|---------|-----------|--------|
| 17.0 | **Arreglar SSA** — fix intersect() dominators, phi fallbacks, param seeding, stack restoration | 🔴 BLOQUEANTE | ⭐⭐⭐⭐⭐ | ✅ **COMPLETADO** |
| 17.1 | **Mem2reg (vía LLVM O3)** — promueve allocas a SSA, elimina load/store | 5-10× en arithmetic | ⭐⭐⭐⭐⭐ | ✅ **CUBIERTO por default\<O3\>** |
| 17.2 | **GVN + LICM + SCCP (vía LLVM O3)** — elimina redundancias, mueve invariantes | 1.5-3× | ⭐⭐⭐⭐ | ✅ **CUBIERTO por default\<O3\>** |
| 17.3 | **nsw/nuw flags** — `add nsw i32` verificado en IR generado | 1.5-3× | ⭐⭐⭐⭐ | ✅ **COMPLETADO** (funciona correctamente) |
| 17.4 | **Ejecutar -O3 sobre el módulo LLVM completo** | 1.5-5× | ⭐⭐⭐ | ✅ **COMPLETADO** `run_passes("default<O3>")` |
| 17.5 | **Eliminar allocas temporales en backend no-SSA** — stores/loads redundantes → valores directos | 2-3× | ⭐⭐⭐ | ✅ **COMPLETADO** (de 15 a 2 allocas por función) |
| 17.6 | **Constant folding + propagation en LLVM IR** | — | ⭐⭐ | ✅ **COMPLETADO** |

### Fase 18 — Zero-Cost Abstractions (renumerada) 📅
- [ ] ⭐⭐⭐⭐ 18.1 — Stack allocation para `final class` pequeños (escape analysis)
- [ ] ⭐⭐⭐ 18.2 — Inlining completo de `.map()`/`.filter()`/`.fold()`
- [ ] ⭐⭐⭐ 18.3 — Monomorfización verificada en LLVM IR
- [ ] ⭐⭐⭐ 18.4 — Eliminación de vtables para clases sin herencia
- [ ] ⭐⭐ 18.5 — Devirtualización de métodos

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

### Fase 11 — Package Manager ✅
- [x] 11.1 Manifest completo (validación, versiones, dev-deps, [project] table)
- [x] 11.2 Resolución semver + lock file + cache + resolución transitiva
- [x] 11.3 Registry (cliente HTTP con ureq, descarga, extract, publish, login, file registry)
- [x] 11.4 Importación desde paquetes resueltos (pipeline + search paths)
- [x] 11.5 Comandos: `ky add` real (resuelve inmediatamente), `ky publish`, `ky login`, `ky update`, `ky outdated`

### Fase 12 — Tooling ✅
- [x] 12.1 Test framework (`#[test]`, assert builtins, ky test)
- [x] 12.2.1-12.2.12 LSP features principales (diagnósticos, autocompletado, go-to-def, hover, find refs, rename, formatting)
- [x] 12.2.13 Inlay hints (tipos inferidos en variables + return types)
- [x] 12.2.14 Diagnostics en ky.toml
- [x] 12.2.15 Code lens "Run test" (LSP + VS Code command)
- [x] 12.2.6 Go-to-definition en dependencias
- [x] 12.3.1 Syntax highlighting — sintaxis Kyle v0.4.0
- [x] 12.3.2 Language icon
- [x] 12.3.3 Task provider (Run/Build/Check/Test)
- [x] 12.3.4 Problems panel
- [x] 12.3.5 Snippets actualizados
- [x] 12.3.6 Debug adapter (DAP)
- [x] 12.3.7 Testing UI (VS Code TestController + #[test] discovery)
- [x] 12.3.8 Extension packaging (scripts/build-extension.sh)
- [x] 12.3.9 Marketplace metadata (README, CHANGELOG)
- [x] 12.3.10 Color theme ("Kyle Pastel")
- [x] 12.4 Formatter completo (`ky fmt --check`, project mode, [format] config, sintaxis moderna)
- [x] 12.5 Shell completions (zsh, fish, powershell + `ky add` dynamic completion)

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
| `kyc_frontend` unit tests | 82 | ✅ All passing |
| `kyc_semantic` unit tests | 17 | ✅ All passing |
| `kyc_mir` unit tests | 11 | ✅ All passing |
| `kyc_tools` unit tests | 24 | ✅ All passing |
| `kyc_runtime` unit tests | 0 | n/a (C-ABI) |
| `kyc_backend` unit tests | 0 | n/a |
| `kyc_core` unit tests | 10 | ✅ All passing (new resolver tests) |
| `kyc_driver` unit tests | 9 | ✅ All passing |
| `kyc_cli` unit tests | 0 | n/a |
| End-to-end `ky test` | 12 | 11/12 passing (1 pre-existing failure: test_misc.ky) |
| **Total Rust unit tests** | **157** | **✅ All passing** |

---

## 10. Estado Actual (v0.4.0)

### Completado desde v0.3.0

| Componente | Estado |
|------------|--------|
| Fase 5: HIR crate + desugaring | ✅ |
| Fase 6: Semantic Analysis (13/13) | ✅ |
| Fase 7: Move Semantics (13/13) | ✅ |
| Fase 8: Release mode (`OptimizationLevel::Aggressive`) | ✅ SSA + O3 funcionando correctamente |
| Fase 9: Async Thread Pool V2 (`async fn`, `async expr`, `await expr`) | ✅ |
| Fase 10: Iterators — 17 métodos de lista | ✅ |
| Fase 15: SSA Form — Mem2Reg, Phi, GVN, benchmarks correctos (debug) | ✅ |
| Fase 16.3: `readonly`/`readnone` en runtime externs | ✅ |
| Fase 11: Package Manager — resolver, registry client, cache, lock, publish, login, update, outdated, import desde paquetes | ✅ |
| **157 tests Rust** (↑ desde 123, +34 nuevos de Phase 11) | ✅ |
| `ownership.rs` y `ky_release` declaration removidos | ✅ |
| `print_int`/`println_int` builtins removidos → ahora `print(42)` | ✅ |
| List borrowing fix — `ky_list_push/get/set/len` en borrowing funcs | ✅ |
| Built-in type methods: `add/pop/len/upper/lower/trim/contains/replace` | ✅ |
| Proyecto de prueba `examples/src/main.ky` con 41 secciones | ✅ |
| `.map()`, `.filter()`, `.fold()`, `.reduce()` como métodos (vía fn ptr C-ABI) | ✅ |
| Bugs SSA fix: `const_values` en Call, CondBr trunc i1 | ✅ |

### Pendiente inmediato (tras Fase 11)
| Prioridad | Tarea | Fase |
|-----------|-------|------|
| 🔴 CRÍTICO | **SSA release hang FIXED** — cross-block value lookup en ssa_read! | 16.0 ✅ |
| 🔴 CRÍTICO | LLVM optimization passes (run_passes("default<O3>")) | 17.4 ✅ |
| 🟢 MEDIO | Registry server implementation (servidor HTTP para paquetes) | 11.3 (server) |
| 🟢 MEDIO | `packages/` directory + path dependencies en ky.toml | 11.6 |
| ✅ RESUELTO | ~~15.B2~~ Missing extern `ky_list_*`/`ky_iter_*` — ya están todas declaradas | ✅ |
| ✅ RESUELTO | ~~15.B3~~ Duplicate `ky_dict_*` — cada una aparece 1 vez, sin duplicados | ✅ |
| ✅ RESUELTO | ~~test_misc.ky~~ — archivo no existe, 0 test failures | ✅ |
| ✅ COMPLETADO | `nsw`/`nuw` flags — `add nsw i32` verificado en IR | 16.1 |
| ✅ IMPLEMENTADO | Release mode fix — SSA cross-block phi values (ssa_read! busca en todos los block_vals) | ✅ |
| ✅ IMPLEMENTADO | Return type coercion en SSA path | ✅ |
| ✅ IMPLEMENTADO | LLVM -O3 passes vía new pass manager (run_passes) | ✅ |
| ✅ IMPLEMENTADO | param_values.clear() entre funciones SSA | ✅ |
| ✅ IMPLEMENTADO | 17.0 — SSA fix intersect() dominators (BUG7), phi fallbacks, param seeding, stack restoration | ✅ |
| ✅ IMPLEMENTADO | 17.3 — nsw/nuw flags verificados en IR (`add nsw i32`) | ✅ |
| ✅ IMPLEMENTADO | 17.5 — Eliminar allocas temporales en backend no-SSA (de 15 a 2 allocas) | ✅ |

### Bugs encontrados y arreglados

| Bug | Síntoma | Fix | Archivo |
|-----|---------|-----|---------|
| `as f64` casting | `(39 as f64)` producía ~3.4e-309 | Agregar pattern `IntValue → FloatType` | `codegen.rs` |
| `const_values` en Call | Argumentos constantes en llamadas a función (ej: `print("A")`) producían output nulo | Insertar entrada en `const_values` para argumentos | `ssa.rs` |
| CondBr i1 trunc | Comparaciones devuelven i1 pero CondBr comparaba con i32 0 | Truncar a i1 si `bit_width > 1` | `codegen.rs` |

### Issues Conocidos

| Issue | Síntoma | Causa raíz | Estado |
|-------|---------|------------|--------|
| ~~15.B2~~ | ~~Faltaban ky_list_*/ky_iter_*~~ | ~~declare_runtime_externs() incompleto~~ | ✅ YA NO APLICA — todas declaradas |
| ~~15.B3~~ | ~~ky_dict_* duplicadas~~ | ~~Refactor incompleto~~ | ✅ YA NO APLICA — sin duplicados |
| ~~`nsw`/`nuw` (16.1)~~ | ~~No aparecían en IR~~ | ~~Posible bug en inkwell~~ | ✅ VERIFICADO — `add nsw i32` OK |
| ~~test_misc.ky~~ | ~~Pre-existing failure~~ | ~~Archivo no existe~~ | ✅ YA NO APLICA — 0 failures |

### Resultados de benchmark (2026-07-02) — FINAL, SSA + LLVM -O3

> **Fix aplicado:** SSA cross-block value lookup (ssa_read! busca en todos los block_vals)
> + LLVM `run_passes("default<O3>")` vía new pass manager.
> Todos los resultados correctos, Kyle iguala a C/Rust en user time.

**Prueba 1: Primos** — `is_prime()` hasta 3,000,000
| Lenguaje | Tiempo (user) | vs Rust | vs Python |
| :--- | :--- | :--- | :--- |
| **Kyle (SSA+O3)** | **0.19s** | **1.0×** 🏆 | **46×** |
| C (gcc -O3) | 0.19s | 1.0× | 46× |
| Rust (rustc -O) | 0.19s | 1.0× | 46× |
| C# .NET 10 | 0.20s | 1.1× | 44× |
| Java 26 | 0.22s | 1.2× | 40× |
| Python 3 | 8.70s | 46× | 1× |

**Prueba 2: Aritmética** — `total = total + i * 2 - 1` (500M iteraciones, i32 wrap)
| Lenguaje | Tiempo (user) | vs Rust | vs Python |
| :--- | :--- | :--- | :--- |
| **Kyle (SSA+O3)** | **0.00s*** | **1×** 🏆 | **∞** |
| C (gcc -O3) | 0.00s* | 1× | ∞ |
| Rust (rustc -O) | 0.00s* | 1× | ∞ |
| Java 26 | 0.14s | — | 175× |
| C# .NET 10 | 0.25s | — | 98× |
| Python 3 | 24.54s | — | 1× |

\* Loop optimizado completamente por LLVM/GCC (const-folding)

**Prueba 3: Mandelbrot** — 390×390 grid, 100 max iter (punto flotante)
| Lenguaje | Tiempo (user) | vs Rust | vs Python |
| :--- | :--- | :--- | :--- |
| **Kyle (SSA+O3)** | **0.01s** | **1×** 🏆 | **41×** |
| C (gcc -O3) | 0.01s | 1× | 41× |
| Rust (rustc -O) | 0.01s | 1× | 41× |
| C# .NET 10 | 0.03s | 3× | 14× |
| Java 26 | 0.03s | 3× | 14× |
| Python 3 | 0.41s | 41× | 1× |

**Resumen — User Time:**
| Benchmark | **Kyle** | **C** | **Rust** | **C# .NET 10** | **Java 26** | **Python** |
|-----------|:-------:|:----:|:--------:|:--------------:|:----------:|:----------:|
| Primes 3M | **0.19s** | 0.19s | 0.19s | 0.20s | 0.22s | 8.70s |
| Arithmetic 500M | **0.00s*** | 0.00s* | 0.00s* | 0.25s | 0.14s | 24.54s |
| Mandelbrot | **0.01s** | 0.01s | 0.01s | 0.03s | 0.03s | 0.41s |

**Conclusión:**
- **Kyle = C = Rust** en user time para todos los benchmarks computacionales.
- C# .NET 10 y Java 26 quedan ~1.5–3× detrás de Kyle/C/Rust.
- Python es 25–50× más lento que Kyle.
- La diferencia en wall time (startup overhead del runtime dinámico) no afecta
  el rendimiento CPU real.
- **Pipeline completo:** SSA elimina allocas → LLVM -O3 optimiza →
  rendimiento nativo a la par con C/Rust.
  de PHI nodes que impiden su uso en producción.
- **Fase 17 (Optimization Pipeline)** debe ejecutar pases LLVM como `mem2reg`,
  `gvn`, `licm` para cerrar el gap, además de arreglar el SSA.

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
| 15.B1 | **Release mode hang** | `ky build --release` produce binarios que cuelgan (no terminan) | SSA pipeline + `OptimizationLevel::Aggressive` genera IR que LLVM optimiza incorrectamente (posiblemente por falta de atributos como `inbounds`/`readonly`/`noalias` que causan loops infinitos post-optimización) | 🔴 CRÍTICO |
| 15.B2 | **Missing extern declarations** | Funciones como `ky_list_pop_first`, `ky_list_clear`, `ky_list_contains`, `ky_list_insert`, `ky_list_remove_at`, `ky_list_map`, `ky_list_filter`, `ky_list_fold`, `ky_list_reduce`, `kl_iter_new`, `kl_iter_next`, `kl_iter_map`, `kl_iter_filter`, `kl_iter_collect` existen en `lower.rs` y runtime pero NO están declaradas como LLVM externs en `declare_runtime_externs()` de `codegen.rs` | `codegen.rs` olvidó declararlas | 🟡 ALTO |
| 15.B3 | **Duplicate extern declarations** | `ky_dict_new`, `ky_dict_free`, `ky_dict_get`, `ky_dict_set`, `ky_dict_len` están declaradas DOS VECES en `declare_runtime_externs()` | Refactor incompleto | 🟢 BAJO |
| 15.B4 | **`ky_retain`/`ky_release` sin usar** | Las funciones existen en runtime pero no son llamadas por el compilador | Move semantics reemplazó refcounting | 🟢 BAJO |

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
| 2.4 | Loops simples verificados: `while i < N: total = total + i; i = i + 1` | `ssa_test.ky` | ✅ |
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

### Fase 16 — LLVM IR Quality ✅ COMPLETADO (excepto 16.1 nsw/nuw diferido + 16.8 lifetime.start/end desactivado)

#### 🔬 Estado Real (Julio 2026)

| Sub-fase | Estado | Detalle |
|----------|--------|---------|
| **16.0 — Fix release hang** | ✅ COMPLETADO | Benchmarks a la par con C/Rust (primes 0.19s, mandelbrot 0.01s) |
| **16.2 — inbounds en GEPs** | ✅ COMPLETADO | `build_in_bounds_gep` en 5 ubicaciones (`codegen.rs:643,662,1820,1845,1856`) |
| **16.3 — readonly/readnone** | ✅ COMPLETADO | 13 funciones `memory("read")` + 7 funciones `memory("none")` |
| **16.4 — noalias en parámetros** | ✅ COMPLETADO | Ambos paths SSA y non-SSA (`codegen.rs:288-300, 1284-1296`) |
| **16.5 — align en allocas** | ✅ COMPLETADO | `set_alignment(8)` en todos los allocas |
| **16.6 — noundef en parámetros** | ✅ COMPLETADO | Aplicado junto con noalias |
| **16.7 — !range metadata en bool** | ✅ COMPLETADO | `add_bool_range` method (`codegen.rs:42-52`) |
| **16.9 — TBAA metadata** | ✅ COMPLETADO | Árbol TBAA completo con nodos int/float/ptr (`codegen.rs:119-156`) |
| **16.8 — lifetime.start/end** | 🔶 **DESACTIVADO** | Código existe (`codegen.rs:174-191`) pero **comentado** (líneas 358, 1375): `// DEBUG: disabled for mem2reg test` |
| **16.1 — nsw/nuw flags** | ✅ COMPLETADO | `build_int_nsw_add/sub/mul` generan `add nsw i32` correctamente en IR. Verificado en SSA y non-SSA |

**Nota:** 16.8 disabled por diseño — lifetime.start/end interferían con debugging de mem2reg. Reactivar si es necesario ahora que 17.5 está implementado.

---

### Fase 17 — Optimization Pipeline ✅ COMPLETADO

**Objetivo:** Ejecutar pases de optimización LLVM para cerrar el gap de 
rendimiento con Rust. El gap original (2.6-100×) se debía a IR pobre con
múltiples allocas por función.

**Resultado:** Todos los items implementados y verificados. Kyle iguala a C/Rust
en todos los benchmarks computacionales.

| # | Tarea | Archivo(s) | Prioridad | Estado |
|---|-------|-----------|-----------|--------|
| 17.0 | Arreglar SSA backend — fix intersect() dominators (BUG7 root cause), phi fallbacks (BUG3), param seeding (BUG5), stack restoration (BUG4) | `ssa.rs`, `codegen.rs` | ⭐⭐⭐⭐⭐ | ✅ **COMPLETADO** — todos los bugs corregidos y verificados |
| 17.1 | Ejecutar `mem2reg` de LLVM en el módulo | `pipeline.rs:365-379` | ⭐⭐⭐⭐⭐ | ✅ **CUBIERTO** por `default<O3>` |
| 17.2 | GVN + LICM + SCCP optimization passes | `pipeline.rs:365-379` | ⭐⭐⭐⭐ | ✅ **CUBIERTO** por `default<O3>` |
| 17.3 | nsw/nuw flags — `add nsw i32` verificado en IR | `codegen.rs:195-207, 1744-1748, 2374-2376` | ⭐⭐⭐⭐ | ✅ **COMPLETADO** — flags generados correctamente |
| 17.4 | Ejecutar `-O2`/`-O3` completo sobre el módulo LLVM | `pipeline.rs:365-379` | ⭐⭐⭐⭐⭐ | ✅ **COMPLETADO** `run_passes("default<O3>")` |
| 17.5 | Eliminar allocas temporales en backend no-SSA — análisis de single-block locals | `codegen.rs:1400-1480, 1509-1513, 1596-1605` | ⭐⭐⭐ | ✅ **COMPLETADO** (de 15 a 2 allocas por función) |
| 17.6 | Constant folding para literales grandes | `optimize.rs:554-571`, `codegen.rs:2287-2301` | ⭐⭐ | ✅ **COMPLETADO** |

**Resumen:** Fase 17 completada al 100%. Todos los items implementados, verificados
en IR generado, y tests pasando en modo SSA (release) y no-SSA (debug).

### Fase 18 — Zero-Cost Abstractions 📅 (renumerada)

**Objetivo:** Garantizar que las construcciones de alto nivel (clases, genéricos,
iteradores, closures) tengan CERO sobrecarga en tiempo de ejecución.

| # | Tarea | Prioridad | Depende de |
|---|-------|-----------|------------|
| 18.1 | Stack allocation para `final class` pequeños (hoy heap) | ⭐⭐⭐⭐ | — |
| 18.2 | Inlining completo de `.map()`/`.filter()`/`.fold()` en código máquina | ⭐⭐⭐ | Fase 17 |
| 18.3 | Monomorfización de genéricos verificada en LLVM IR | ⭐⭐⭐ | — |
| 18.4 | Eliminación de vtables para clases sin herencia | ⭐⭐⭐ | — |
| 18.5 | Devirtualización de llamadas a métodos (speculative devirt) | ⭐⭐ | Fase 14 |

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

*Versión: v1.0 · Actualizado: 2026-07-02 — Fases 14-17 COMPLETADAS*
