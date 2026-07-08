# Roadmap

## Platform Architecture

Kyle is a layered platform. Each layer only knows the layer below it.

```
Applications (IDE, Desktop, KYOS...)
    │
Kyle UI (widgets, navigation)
    │
Kyle Scene (scene graph, layout)
    │
Kyle Graphics (canvas, GPU)
    │
Kyle Windowing (windows, events)
    │
Kyle Platform (FS, net, threads, audio, sensors)
    │
Kyle Runtime (memory, strings, collections)
    │
Kyle Language (compiler)
```

The **compiler and runtime are complete** (Phases 1-17).  
The **upper layers (Windowing → Applications) are aspirational — sin fecha estimada**.  
El foco actual son **packages backend** (HTTP, SQLite, Postgres, websocket).

### Current crate structure

```
kyc_core/          ✅ foundation
kyc_frontend/      ✅ lexer + parser
kyc_hir/           ✅ desugaring
kyc_semantic/      ✅ type checker, scope
kyc_mir/           ✅ MIR lowering, SSA
kyc_backend/       ✅ LLVM codegen
kyc_driver/        ✅ pipeline
kyc_cli/           ✅ CLI binary
kyc_runtime/       ✅ runtime (memory, strings, lists, dicts)
kyc_tools/         ✅ LSP, formatter, package manager
```

### Planned crate additions

| Crate | When | Purpose |
|-------|------|---------|
| `kyc_platform` | After Phase 0 | Platform API (FS, net, time, env) — Rust crate |
| `kyc_platform_macos` | After Phase 0 | macOS platform adapter |
| `kyc_platform_linux` | future | Linux platform adapter |
| Various `ky-*` | After Phase 0 | Kyle packages (HTTP, SQLite, JSON, etc.) |
| `kyc_graphics` | Aspiracional | Canvas, GPU rendering |
| `kyc_ui` | Aspiracional | Widget library |
| `kyc_scene` | Aspiracional | Scene graph |
| `kyc_platform_macos` | Aspiracional | macOS platform adapter |
| `kyc_platform_linux` | Aspiracional | Linux platform adapter |

---

## ✅ Completed — Phases 1–17

| Phase | Description | Status |
|-------|-------------|--------|
| 1–2 | Documentation and specification | ✅ |
| 3 | Lexer | ✅ |
| 4 | Parser | ✅ |
| 5 | HIR + desugaring | ✅ |
| 6 | Semantic analysis (type checker, scope) | ✅ |
| 7 | Borrow semantics (ownership, move, copy, borrow checker) | ✅ **v0.6** |
| 8 | Backend release mode (LLVM, binary output) | ✅ |
| 9 | Async scheduler V2 (thread pool) | ✅ |
| 10 | Iterators — 17 list methods (map, filter, fold, etc.) | ✅ |
| 11 | Package manager (registry, cache, lock, publish, login) | ✅ |
| 12 | Tooling (LSP, VS Code extension, formatter, test framework) | ✅ |
| 13 | Complete syntax (generics, ranges, match, operator overloading, is, ptr, for-else, static fn, **) | ✅ |
| 14 | References and borrow checker (`^T` mutable, `&T` borrow, `^&T` mut borrow, field mutability) | ✅ **v0.6** |
| 15 | SSA form (mem2reg, phi nodes, GVN, dominator fix) | ✅ |
| 16 | LLVM IR quality (nsw, TBAA, inbounds, readonly, noalias, noundef, !range, align, lifetime) | ✅ |
| 17 | Optimization pipeline (O3, constant folding, alloca elimination, nsw verification) | ✅ |

**Benchmarks (Kyle vs Rust vs C) — actualizado Jul 2026:**

Ejecutado en Apple M4 (ARM64). Compiladores: `clang -O3`, `rustc -O`, `ky build`.

| Benchmark | Kyle | Rust | C | Relación (K/C) |
|-----------|:----:|:----:|:-:|:--------------:|
| Primes 3M | 672ms | 467ms | 444ms | 1.51x |
| Fibonacci 40 | 406ms | 423ms | 410ms | **0.99x** |
| Mandelbrot (float) | 294ms | 274ms | 271ms | 1.08x |
| String concat 100k | 1.98s | 272ms | 238ms | 8.3x |
| **Compile memory** | **7.0MB** | 18.9MB | 2.6MB | **2.7x vs Rust** |

Kyle compite directamente con C y Rust en CPU-bound (primes, fib, mandelbrot).
La brecha en strings se debe a que Kyle usa `str_concat` (nueva alloc + copy en cada `+`).
Rust usa `String::push` que amplía el buffer existente.

Kyle usa 2.7x menos RAM que Rust durante compilación (sin LTO).

Ver `BENCHMARK.md` para el suite completo y `docs/` para features faltantes.

---

## 🔜 Immediate Next — Phase 0: FFI Foundation

**Estimated: 2.5 days**  
**Crate restructuring: NOT required for Phase 0** — all changes are in existing compiler crates.

Goal: Enable packages written in 100% Kyle without Rust.

| Step | Task | Files | Est. |
|------|------|-------|------|
| 0.1 | `extern fn` declaration — parser, semantic, MIR, codegen | `parser.rs`, `type_checker.rs`, `lower.rs`, `codegen.rs` | 1 day |
| 0.2 | `@link` directive — parser + linker integration | `parser.rs`, `pipeline.rs` | 0.5 day |
| 0.3 | `ptr` type complete — load, store, offset, arithmetic | `lower.rs`, `codegen.rs` | 1 day |

After Phase 0: packages like `ky-http`, `ky-sqlite`, `ky-json` can be written in pure Kyle with FFI to C libraries.

---

## Runtime Rewrite Analysis — What Kyle Can and Cannot Do

The runtime (`libkyc_runtime.a`) has **88 `extern "C"` functions**. Here's exactly what can be rewritten in pure Kyle (+ `extern fn` to libc) and what cannot.

### ✅ Can rewrite NOW (65 functions — 74%)

These use only arithmetic, raw pointers, and libc FFI. All doable with current Kyle features.

| Module | Count | Examples |
|--------|:-----:|----------|
| `string.rs` | 20 | `ky_strlen`, `ky_concat`, `ky_str_to_i64`, `ky_str_contains`, `ky_substr`, etc. |
| `list.rs` | 28 | `ky_list_new/push/get/set/pop/len`, `ky_list_map/filter/fold`, `ky_range`, etc. |
| `io.rs` | 10 | `ky_print/println`, `ky_open/read/write/close`, `ky_sleep/now` |
| `lib.rs` | 4 | `ky_pow`, `ky_add_pct`, `ky_sub_pct`, `ky_mul_pct` |
| `assert.rs` | 1 | `ky_assert` |
| `async_.rs` | 1 | `ky_yield` |

**Total: 65 functions. Estimated: 2-3 days.**

### 🔶 Needs missing Kyle feature (12 functions — 14%)

| Function | What's Missing | Workaround |
|----------|---------------|------------|
| `ky_alloc/ky_free` | **Heap allocator** | Add `extern fn malloc(size: i64) ptr` ✅ already possible |
| `ky_retain/ky_release` | **Atomic operations** | Needs LLVM `atomicrmw` instruction or `__sync_fetch_and_add` via extern |
| `ky_iter_new/next/map/filter/free` (5) | **`box::new` + function ptr transmute** | Use `extern fn malloc` for heap + manual vtable |
| `ky_f64_to_str` | **f64 formatting** | Use `extern fn snprintf(buf, size, fmt, val)` from libc |
| `ky_assert_eq/assert_ne` | **i64 error message formatting** | Use `ky_i64_to_str` (from ✅ group) |

**Estimated: 2-3 days (after adding missing extern fn declarations).**

### ❌ Cannot rewrite (12 functions — 14%)

| Module | Count | Why |
|--------|:-----:|-----|
| `dict.rs` | 10 | **Needs hash map** — all 8 dict functions + 2 unimplemented (`ky_dict_contains`, `ky_dict_remove`). Dict is `HashMap<String, i64>` from Rust std. Kyle needs a hash table implementation (FNV-1a + open addressing, ~200 lines of Kyle). |
| `thread.rs` | 2 | **Needs OS threads** — `ky_spawn_thread` needs `pthread_create` with a C-compatible trampoline. `ky_join_thread` needs `pthread_join`. The trampoline requires codegen support for `extern "C"` closures. |
| `async_.rs` | 2 | **Needs async executor** — `ky_spawn_task` and `ky_await_task` depend on threads, channels, mutexes, atomics, and a global executor singleton. This is the most complex piece. |

**Estimated: 2-3 weeks (hash map: 1 week, threads: 1-2 weeks, async: depends on threads).**

### Summary

| Status | Count | % |
|--------|:-----:|:-:|
| ✅ Can rewrite NOW | 65 | 74% |
| 🔶 Needs minor feature | 12 | 14% |
| ❌ Needs major feature | 12 | 14% |
| **Total** | **88** | **100%** |

---

## Runtime Rewrite Plan — ⏸️ PAUSADO

> **Decisión:** La reescritura del runtime Rust a Kyle está pausada hasta que Kyle se use en proyectos reales.  
> El archivo `crates/kyc_runtime/src/string.ky` existe con 18 funciones pero permanece inerte.  
> Reactivar cuando: (1) proyectos reales requieran auto-suficiencia, (2) el equipo lo considere necesario.

| Phase | Descripción | Estado |
|-------|-------------|--------|
| A | Rewrite 65 functions (string, list, io) | ⏸️ Pausado |
| B | Missing extern fn declarations | ⏸️ Pausado |
| C | Hash map (Dict) en Kyle puro | ⏸️ Pausado |
| D | Threading & Async | ⏸️ Pausado |
| E | Self-hosting compiler | 📅 Baja prioridad |

---

## Implementation Order

```
AHORA → Backend Packages + websocket/WASM — ✅ ENFOQUE ACTUAL
   ↓
      Runtime reescritura en Kyle — ⏸️ PAUSADA (hasta tener proyectos reales)
         ↓
            Windowing, Graphics, Scene, UI — 📅 ASPIRACIONAL (sin fecha)
               ↓
                  KYOS (sistema operativo) — 📅 ASPIRACIONAL (sin fecha)
                     ↓
                        Self-hosting — 📅 BAJA PRIORIDAD
```

### Fases HTTP/JSON

| Fase | Descripción | Depende de | Estado |
|------|-------------|------------|--------|
| 1 | Function pointers (`fn()` como tipo de primera clase) | Compiler | ✅ |
| 2 | `JsonValue` type + auto-serialize de `final class` | Union types | ✅ |
| 3 | HTTP client: `client.post(url, class)` auto-JSON | Fase 2 | ✅ |
| 4 | HTTP Server: callbacks, `{id:i32}` params, middleware | Fase 1 + 3 | 🔜 |
| 5 | websocket + SSE sobre Server | Fase 4 | 🔜 |

**Current state:** Packages work (http client, json, sqlite, env) in 100% Kyle with FFI. HTTP Server TCP accept working. Function pointers implemented. Benchmarks completados (Kyle vs Rust vs C). Bugs recientes arreglados: constructor sin `fn`, `@link` propagation, `from X import a, b, c`, tuple destructuring, `close()` name conflict.

**Runtime reescritura en Kyle: PAUSADA.** El archivo `crates/kyc_runtime/src/string.ky` existe con 18 funciones pero NO se usa. Reactivar solo cuando Kyle se use en proyectos reales y la auto-suficiencia sea necesaria. Mientras tanto, el runtime Rust funciona y es estable.

### Bugs conocidos (no críticos)

| Bug | Impacto | Workaround |
|-----|---------|------------|
| Closures como argumentos truncados a i32 | Bajo | Usar funciones nombradas |

## Package Registry

Kyle packages are distributed via **GitHub Pages registry** at `https://IT-KYNERA.github.io/KYLE/docs`. No `KL_REGISTRY` needed.

### Available packages

| Package | Registry | Status |
|---------|----------|--------|
| `http` | GitHub Pages | ✅ v3.0 (cliente) |
| `json` | GitHub Pages | ✅ v0.1.0 |
| `sqlite` | GitHub Pages | ✅ v0.1.0 |
| `env` | GitHub Pages | ✅ v0.1.0 |

### Development

```bash
# Local — apuntar al file registry del repo
export KL_REGISTRY=file:///Users/me/KYLE/registry
ky add http
```

See `docs/05-packages/registry.md` for full documentation.

---

## Estado real del compilador (verificado 2026-07-04)

### ✅ Todo funciona — sin bugs conocidos

| Feature | Ejemplo |
|---------|---------|
| Funciones importadas de módulos | `from http.server import send_json` ✅ |
| Clases importadas de módulos | `from http.server import router` ✅ |
| Struct return con strings | `fn json_res(data: str) Res: Res { body: data }` ✅ |
| Generic function calls con `<T>` | `identity<i32>(42)`, `deserialize<LoginReq>(body)` ✅ |
| Generic function/class decls con `<T>` | `fn identity<T>(x: T) T:`, `class box<T>:` ✅ |
| Type alias con `<T>` | `type Maybe<T> = T?` ✅ |
| `serialize(val)` / `deserialize<T>(str)` | Builtin global ✅ |
| Clases + métodos | `client.get(url)` ✅ |
| Struct literal con type args | `Container<i32> { value: 42 }` ✅ |
| Multi-level module imports | `from http.server import ...` ✅ |
| Package @link propagation | `@link "c"` se mergea al programa automáticamente ✅ |
| **Implicit main** | `ky run app.ky` sin `fn main()` — código módulo ejecutado automáticamente ✅ |
| **Closures multi-line** | `(req, client):\n    send_json(...)` con bloque indentado ✅ |
| **Function pointers** | `handler as ptr`, llamado vía `fn_ptr(args)` con CallIndirect ✅ |
| **router HTTP alto nivel** | `app.get("/path", handler)` con `get` como método (keyword) ✅ |
| **Packages sin std/** | `from http.server import ...` busca directamente en `packages/http/src/` ✅ |

### 🔜 En desarrollo

| Feature | Ejemplo | Estado |
|---------|---------|--------|
| `res.json()` | `res.json(serialize(data), 200)` | ✅ Implementado |
| `req.param()` | `req.param("id")` | ✅ Implementado |
| `req.body<T>()` | `req.body<CreateUser>()` | Requiere generic method call |
| `obj.method<T>(args)` | Llamada a método con tipo genérico | Parser + lowerer |

---

## Tipos faltantes — Plan de implementación

> La especificación completa de cada tipo está en [`docs/03-language-reference/types.md`](docs/03-language-reference/types.md) sección "Platform types (futuro)".
> Todos los nombres de tipo van en **minúscula**: `datetime`, `bytes`, `decimal`, `uuid`, `url`, `regex`, `duration`.

### Tipos actuales de Kyle

| Categoría | Tipos |
|-----------|-------|
| **Primitivos** | `i8`, `i16`, `i32`, `i64`, `f32`, `f64`, `bool`, `char`, `str` |
| **Compuestos** | `list<T>`, `dict<K,V>`, `ptr` |
| **Funciones** | `fn(T) U` (function pointers) |
| **Usuario** | `class`, `struct`, `enum`, `type alias` |
| **Genéricos builtin** | `Option<T>` (`T?`), `Result<T, str>` (`T!`) |

### Tipos faltantes con prioridad alta (necesarios para PostgreSQL)

| Tipo | Descripción | Subtipos/Operaciones | Depende de |
|------|-------------|---------------------|------------|
| `datetime` | Fecha y hora | `date`, `time` — sumar/restar días/horas, formatear, parsear | Runtime en Rust |
| `bytes` | Datos binarios | `len()`, `[i]`, `slice()`, `hex()`, `base64()` | Runtime en Rust |
| `decimal` | Precisión fija | `round()`, `truncate()`, operaciones aritméticas | Runtime en Rust |

### Tipos faltantes con prioridad media

| Tipo | Descripción | Operaciones |
|------|-------------|-------------|
| `uuid` | Identificador único | `uuid.v4()`, `uuid.parse()`, `uuid.to_string()` |
| `url` | Localizador | `url.scheme()`, `url.host()`, `url.path()` |
| `regex` | Expresión regular | `regex.match()`, `regex.find()`, `regex.replace()` |
| `duration` | Intervalo de tiempo | `duration.seconds()`, `duration.hours()`, `a + b` |

### 🏷️ Convención de nombres (camelCase)

| Regla | Ejemplos |
|-------|----------|
| Tipos 1 palabra | `regex`, `url`, `uuid`, `bytes`, `decimal`, `mutex`, `future`, `box` |
| Tipos multi-palabra | `str_builder`, `atomic_i64`, `atomic_bool`, `big_int`, `date_time` |
| Funciones | `spawn_thread`, `join_thread`, `parallel_for`, `fetch_add`, `to_str` |
| Constructores tipo | `regex("[0-9]+")`, `box(42)`, `channel<i32>(16)` |
| Constantes | `MAX_SIZE := 1024` (UPPER) |

### Nombres a renombrar (código existente)

| Actual | Nuevo | Archivos |
|--------|-------|----------|
| `ky_parallel_for` | `ky_parallel_for` (futuro: `parallel.for`) | runtime, symbol_table, lowerer |
| `ky_spawn_thread` | `ky_spawn_thread` (futuro: `thread.spawn`) | runtime, symbol_table |
| `ky_join_thread` | `ky_join_thread` (futuro: `thread.join`) | runtime, symbol_table |
| `ky_str_builder_*` | `str_builder` clase | ✅ ya wrapper |
| Funciones package | camelCase | packages/http/, packages/json/ |

### Decisión: Stack y Queue no son tipos dedicados

| Tipo | Decisión | Razón |
|------|----------|-------|
| **Stack** | ❌ No hay tipo dedicado | `{T}` con `push()`/`pop()` ya es stack. Go y JS hacen lo mismo. |
| **Queue** | ❌ No hay tipo dedicado | `{T}` con `push()`/`pop_first()` cubre FIFO. |
| **set** | ✅ Sí, tipo dedicado | Requiere hash set (sin duplicados, O(1) lookup). |

## ⚠️ Option<T> y Result<T,E> — ¿Qué significa?

`Option<T>` (`T?`) y `Result<T, str>` (`T!`) existen como **enums genéricos** del compilador:

```kyle
x = maybe_value?          # T? = Option<T>
y = fallible_operation!   # T! = Result<T, str>

if x is Some(val):        # pattern matching en Option
    print(val)
```

Están implementados con `Option.Some(value)` / `Option.None` y `Result.Ok(value)` / `Result.Error(msg)`. La sintaxis `T?` y `T!` es azúcar del compilador.

**Limitaciones:** No tienen métodos ricos como `map()`, `and_then()`, `unwrap_or()` (típicos de Rust). Solo pattern matching básico y el operador `??` (default).

### Plan de fases

| Fase | Tipos | Archivos | Esfuerzo |
|------|-------|----------|----------|
| **1** | `datetime` + `duration` (runtime) | `kyc_runtime/src/datetime.rs` + FFI | 3-4 días |
| **2** | `bytes` (runtime) | `kyc_runtime/src/bytes.rs` + FFI | 2-3 días |
| **3** | `decimal` (runtime) | `kyc_runtime/src/decimal.rs` + FFI | 2-3 días |
| **4** | `uuid` (runtime) | `kyc_runtime/src/uuid.rs` + FFI | 1-2 días |
| **5** | Métodos ricos en `Option<T>`/`Result<T,E>` | Compiler + runtime | 2-3 días |

Cada tipo nuevo necesita:
1. **Struct en runtime** (`kyc_runtime/src/`) con operaciones básicas
2. **Extern functions** (`@link "c"` + `extern fn`)
3. **Package Kyle** (`packages/<type>/`) con API de alto nivel
4. **Tests** en el package

---

## Self-Hosting — Codegen Analysis

To compile Kyle with Kyle, the compiler's codegen (`kyc_backend/src/codegen.rs`, ~2,400 lines of Rust) must be rewritten in Kyle. It currently uses **inkwell** (Rust wrapper for LLVM C API).

### What would be needed

| Component | Lines | Difficulty |
|-----------|:-----:|:----------:|
| LLVM C API `extern fn` declarations | ~85 functions | 🟢 Easy (one-time typing) |
| LLVM type wrappers (`LLVMValueRef`, etc.) | ~100 | 🟢 Easy (all `ptr`) |
| Codegen logic (translate MIR → LLVM IR) | ~2,400 | 🔴 Hard (complex dispatch logic) |
| SSA construction | ~920 | 🟡 Medium (already pure algorithms) |
| Linker driver | ~150 | 🟢 Easy (`system("clang ...")` already works) |

### LLVM is NOT replaced

LLVM stays as the machine-code backend. The same way we call libcurl via `extern fn`, we would call LLVM via `extern fn LLVMBuildAdd(...)`. The ~85 LLVM C API functions map 1:1 to the existing inkwell calls.

**Verdict:** Technically feasible. Not a priority — the Rust compiler is stable and the codegen doesn't need to be self-hosted to ship packages.

### Plan de auto-hospedaje (Self-Hosting)

```kyle
# El objetivo: ky compilado por ky
ky build compiler.ky  # actual: Rust → binario
                     # futuro: Kyle → binario
```

#### Fase 1 — LLVM C API bindings (`packages/llvm`)

Declarar las ~85 funciones de la LLVM C API como `extern fn` en Kyle:

```kyle
@link "LLVM"
extern fn LLVMContextCreate() ptr
extern fn LLVMModuleCreateWithName(name: ptr) ptr
extern fn LLVMAddFunction(mod: ptr, name: ptr, type_: ptr) ptr
# ... ~82 más
```

| Archivo | Funciones | Esfuerzo |
|---------|-----------|----------|
| `packages/llvm/src/core.ky` | Context, Module, Type, Value | 40 funciones |
| `packages/llvm/src/builder.ky` | IRBuilder (BuildAdd, BuildCall...) | 30 funciones |
| `packages/llvm/src/target.ky` | TargetMachine, emit object | 15 funciones |

#### Fase 2 — MIR types en Kyle (`packages/mir`)

Los tipos del compilador (hoy en Rust) se definen en Kyle:

```kyle
final class MirType:
    kind: i32  # 0=i32, 1=str, 2=struct...
    name: str

final class MirInst:
    opcode: i32
    dest: i32
    args: &[i32]
```

| Archivo | Contenido |
|---------|-----------|
| `packages/mir/src/types.ky` | MirType, MirValue, MirConstant |
| `packages/mir/src/inst.ky` | MirInst (25 variants) |
| `packages/mir/src/function.ky` | MirFunction, MirBasicBlock |

#### Fase 3 — Codegen en Kyle (`packages/codegen`)

Traducir `codegen.rs` (~2,400 líneas Rust → Kyle):

```kyle
from llvm import core, builder
from mir import types, inst, function

fn compile_function(f: MirFunction) ptr:
    fn_type = llvm_fn_type(f.return_type, f.params)
    llvm_fn = LLVMAddFunction(module, f.name, fn_type)
    # ...
```

| Módulo | Líneas | Dificultad |
|--------|:------:|:----------:|
| Type lowering | ~300 | 🟢 |
| Function compilation | ~800 | 🟡 |
| Instruction compilation | ~1,000 | 🔴 |
| SSA construction | ~300 | 🟡 |

#### Fase 4 — Bootstrap

```bash
# 1. Compilar el codegen Kyle usando el compilador Rust
ky build codegen.ky -o ky-step1

# 2. Compilar el codegen Kyle usando ky-step1
./ky-step1 codegen.ky -o ky-step2

# 3. Verificar que ky-step2 produce el mismo output
diff <(ky-step1) <(ky-step2)  # bootstrap completado!
```

#### Timeline estimado

| Fase | Descripción | Dependencias | Esfuerzo |
|------|-------------|-------------|----------|
| 1 | LLVM C API bindings | — | 1 día |
| 2 | MIR types en Kyle | — | 2 días |
| 3 | Codegen en Kyle | Fase 1 + 2 | 2 semanas |
| 4 | Bootstrap | Fase 3 | 1 día |
| 5 | Rewrite frontend (lexer+parser) en Kyle | Fase 4 | 4 semanas |

---

## 📅 Post-v1.0 Features

### Phase 18 — Zero-Cost Abstractions

| # | Task | Priority |
|---|------|----------|
| 18.1 | Escape analysis: `final class` on stack instead of heap | ⭐⭐⭐⭐ |
| 18.2 | Small string optimization (SSO): strings < 15 bytes inline | ⭐⭐⭐ |
| 18.3 | Inlining `.map()`/`.filter()`/`.fold()` — zero overhead | ⭐⭐⭐ |
| 18.4 | Verified monomorphization — no boxing for generics | ⭐⭐⭐ |
| 18.5 | Array optimizations — small arrays on stack | ⭐⭐⭐ |
| 18.6 | Vtable elimination — direct dispatch for non-virtual | ⭐⭐⭐ |
| 18.7 | Return value optimization (RVO) — avoid copies | ⭐⭐ |
| 18.8 | Devirtualization — speculative devirt for methods | ⭐⭐ |

### Async V3 — State Machine

| # | Task | Priority |
|---|------|----------|
| 9.1–5 | Replace thread pool with state machine V3 | ⭐⭐⭐ |
| 9.6–8 | Work-stealing scheduler | ⭐⭐⭐ |
| 9.9–11 | Non-blocking I/O (timers, signals, async read/write) | ⭐⭐ |

### Iterators Advanced

| # | Task | Priority |
|---|------|----------|
| 10.1–5 | Functional closures (first-class fn pointers) | ⭐⭐⭐ |
| 10.6–9 | Lazy evaluation — `iter()` trait, lazy chains | ⭐⭐ |

### Alternative Backends

| Backend | Purpose |
|---------|---------|
| Cranelift | Faster compilation (debug mode), no LLVM dependency |
| WASM | Compile Kyle for browser and WebAssembly targets |

---

## ✅ Completado — v0.6 Nuevo modelo ownership

| Fase | Descripción | Estado |
|------|-------------|--------|
| 1 | Parser: `^` mutable, `&` borrow, move default | ✅ |
| 2 | Semantic: type inference para nuevo ownership | ✅ |
| 3 | HIR + MIR: lowering con move por defecto | ✅ |
| 4 | Borrow checker: use-after-move, one mut XOR many immut | ✅ |
| 5 | Benchmarks/packages actualizados | ✅ |
| 6 | `^T` move antiguo eliminado | ✅ |

### Resumen de cambios v0.6

| Antes | Después |
|-------|---------|
| `y = x` = borrow | `y = x` = **move** |
| `fn f(s: str)` = borrow | `fn f(s: str)` = **move** |
| `fn f(^s: str)` = move | Eliminado (move es default) |
| `fn f(&s: str)` = mutable | `fn f(s: &str)` = **borrow** |
| `x: &T = val` = mutable | `x: ^T = val` = **mutable** |
| `f(&x)` = mutable ref | `f(&x)` = **borrow**, `f(^&x)` = mutable borrow |

---

## 📊 Fases de Madurez del Lenguaje (Benchmark gaps)

Features identificadas en los benchmarks que Kyle necesita para ser competitivo como lenguaje de bajo nivel:

| Feature | Importancia | Dependencia | ETA |
|---------|-------------|-------------|-----|
| **Native Arrays `[T; N]`** | ⭐⭐⭐⭐⭐ | Parser + typeck + MIR + codegen | 🚀 **En progreso** |
| **Threads + Concurrency** | ⭐⭐⭐⭐⭐ | Runtime | 📅 Fase D |
| **Async/Await en Kyle** | ⭐⭐⭐⭐ | Runtime + compiler | 📅 Fase D |
| **HashMap completo** (String→any) | ⭐⭐⭐⭐ | Runtime | 📅 Fase C |
| **Networking TCP/UDP** | ⭐⭐⭐⭐ | Runtime + packages | 📅 Fase 4-5 |
| **websocket/SSE** | ⭐⭐⭐ | Packages | 📅 Fase 5 |
| **regex** | ⭐⭐⭐ | Package | 📅 Futuro |
| **Crypto** | ⭐⭐⭐ | Package | 📅 Futuro |
| **Compression** (gzip, brotli) | ⭐⭐⭐ | Package | 📅 Futuro |
| **PGO** (Profile Guided Optimization) | ⭐⭐⭐ | Toolchain | 📅 Futuro |
| **Cache Miss / IPC profiling** | ⭐⭐ | Toolchain | 📅 Futuro |
| **Arena/pool allocators** | ⭐⭐ | Runtime | 📅 Futuro |
| **Vectorization control** (LLVM) | ⭐⭐ | Compiler | 📅 Futuro |
| **Zero-cost Array/List/Dict syntax** | ⭐⭐⭐⭐⭐ | Parser + typeck + MIR | 🚀 **En progreso** |

## Cambios de sintaxis (v0.6)

| Antes | Después | Tipo |
|-------|---------|------|
| `[1, 2, 3]` | `[1, 2, 3]` | Array (antes era list, ahora es array nativo) |
| `[1, 2, 3]` | `{1, 2, 3}` | List (cambia de `[]` a `{}`) |
| `list<T>` | `{T}` | List type (antes `[T]`, ahora `{T}`) |
| — | `[T; N]` | Array type (nuevo, repite valor) |
| `{"key": val}` | `{key: val}` | Dict literal (se puede omitir quotes) |
| `fn f(^s: str)` = move | `y = x` = move por defecto | Ownership: move implícito |
| `y = x` = borrow | `fn f(s: &str)` = borrow | Ownership: `&` es borrow |
| `x: &T = v` = mutable | `x: ^T = v` = mutable | Ownership: `^` es mutable |
| `fn f(&s: str)` no existe | `fn f(s: ^&str)` = mut borrow | Ownership: `^&` compone |

### Nuevo modelo de ownership

| Sintaxis | Significado | Ejemplo |
|----------|-------------|---------|
| `x = v` | Variable inmutable (default) | `x = 42` |
| `x: ^T = v` | Variable mutable | `x: ^str = "hola"` |
| `y = x` | **MOVE** (no-Copy) | `t = s` → `s` inválido |
| `y = x.clone()` | **COPY** explícita | `t = s.clone()` → ambos vivos |
| `f(x)` | MOVE (owned) | `print(s)` → `s` se mueve |
| `f(&x)` | BORROW | `print(&s)` → `s` prestado |
| `f(^&x)` | MUT BORROW | `append(^&s)` → `s` prestado mutable |

Ver `docs/03-language-reference/ownership.md` para especificación completa.

---

## 📈 Optimizaciones futuras (postergadas)

Mejoras identificadas en benchmarks que cierran la brecha con C/Rust:

| # | Mejora | Benchmarks | Impacto estimado | Esfuerzo |
|---|--------|-----------|-----------------|----------|
| 1 | **Register alloc para `^i32`/`^i64`** — LLVM mem2reg no promueve allocas simples con múltiples BB. Solución: identificar `^i32`/`^i64` en codegen y emitir valores LLVM directo sin alloca | Fib (1.6× → 1.0×) | ⭐⭐⭐ | 1-2 días |
| 2 | **`list.reserve(n)` + batch push** — `reserve(n)` pre-asigna capacidad (✅ implementado). Batch push reduce N FFI calls a 1 | Primes (2.7× → 1.5×) | ⭐⭐ | 1 día |
| 3 | **Arrays `[T; N]` pass-by-reference** — Los arrays hoy son copy-by-value. `fn f(a: &[i64; N])` evitaría copiar 80KB en cada acceso. Necesita `&[T; N]` como tipo de parámetro | Matmul (7.8× → 1.0×) | ⭐⭐⭐⭐⭐ | 3-4 días |
| 4 | **str_builder inline hints** — Marcar append con inlinehint para eliminar overhead de FFI call | Concat (1.1× → 0.5×) | ⭐ | 0.5 día |

### Notas de implementación

**#3 Arrays pass-by-reference** es la mejora más importante (matmul es el benchmark más representativo de cómputo real). Hoy cada `a[i]` en `[i64; 10000]`:
1. (innecesario) Load del array completo (80KB)
2. GEP sobre el resultado
3. Load del elemento

Con pass-by-reference:
1. GEP directo sobre el puntero al array original
2. Load del elemento

Requiere nuevo `MirType::Slice` o modificar `MirInst::ArrayElemPtr` para aceptar referencias.

---

## 📋 Plan de completitud de tipos

Kyle debe tener **todos los tipos importantes como nativos** (no packages).
Solo HTTP/Postgres/SQLite son packages. El resto es infraestructura base.

### Fase 1: Arreglar bugs existentes

| Bug | Archivos | Impacto |
|-----|----------|---------|
| `u8`/`u16`/`u32`/`u64` sin MirType ni codegen | `mir.rs`, `codegen.rs`, `lower.rs` | No se pueden declarar variables unsigned |
| `tuple` sin MirType ni codegen | `mir.rs`, `codegen.rs` | Existe en parser pero no compila |
| `char = 'a'` type mismatch | `type_checker.rs` | Bug conocido y documentado |
| `T?` type mismatch con `str?` | `type_checker.rs` | `str?` causa "expects 1 arg, got 2" |
| `T!` con `-> T!` syntax | `parser.rs`, `type_checker.rs` | Arrow syntax no funciona |

### Fase 2: Migrar packages a nativos

Cada tipo package → integración nativa requiere:
1. Tipo Kyle (`final class`) directamente en runtime
2. Builtins registrados en compilador (sin `extern fn` manuales)
3. Sin `from X import Y` — disponibles globalmente

| Package actual | Tipo nativo | Runtime status |
|---------------|-------------|----------------|
| `from datetime import datetime` | `date_time` | ✅ `kyc_runtime/src/datetime.rs` |
| `from datetime import duration` | `duration` | ✅ en datetime.rs |
| `from date import date` | `Date` | ✅ `kyc_runtime/src/date.rs` |
| `from date import time` | `Time` | ✅ en date.rs |
| `from bytes import bytes` | `bytes` | ✅ `kyc_runtime/src/bytes.rs` |
| `from decimal import decimal` | `decimal` | ✅ `kyc_runtime/src/decimal.rs` |
| `from uuid import uuid` | `uuid` | ✅ `kyc_runtime/src/uuid.rs` |
| `from url import url` | `url` | ✅ `kyc_runtime/src/url.rs` |
| `from regex import regex` | `regex` | ✅ `kyc_runtime/src/regex.rs` |
| `ky_getenv`/`ky_setenv` | `Env` | ✅ `kyc_runtime/src/string.rs` |

### Fase 3: Tipos I/O nativos

| Tipo | Estado | Necesita |
|------|--------|----------|
| `file` | ❌ fd i32 | `final class file` + métodos read/write/close/seek |
| `socket` | ❌ fd i32 | `final class socket` + listen/accept/connect |
| `path` | ❌ str | `final class path` + join/dirname/basename/exists |
| `json` | ❌ functions | `final class json` + parse/stringify methods |

### Fase 4: Colecciones faltantes

| Tipo | Estado | Notas |
|------|--------|-------|
| `set<T>` | 🔶 | Type enum existe. Falta: parser, MirType, runtime hash set |
| `Queue<T>` | ❌ | FIFO. Runtime simple (ring buffer) |
| `Stack<T>` | ❌ | LIFO. `{T}` con push/pop ya es stack |
| `slice` | ❌ | Vista de array existente `&[T]`. Necesario para pasar arrays sin copiar |

### Fase 5: Concurrencia nativa

| Tipo | Estado | Notas |
|------|--------|-------|
| `channel<T>` | 🔶 | Runtime listo. Falta tipo Kyle genérico |
| `mutex<T>` | ❌ | Para threads. Runtime Rust ya tiene |
| `AtomicI64` / `AtomicBool` | ❌ | Operaciones lock-free |
| `future<T>` | ❌ | Handle tipado para async |
| `iterator` | 🔶 | KlIter existe en runtime. Falta tipo Kyle |
| `select` | ❌ | Multiplexor de canales |

### Fase 6: Smart pointers

| Tipo | Notas |
|------|-------|
| `box<T>` | Heap pointer simple. Ya existe `ky_alloc` para raw |
| `rc<T>` | Single-thread reference counting |
| `arc<T>` | Multi-thread atomic refcount |
| `weak<T>` | weak reference (evita ciclos rc/arc) |

---

## Estado actual del lenguaje

| Categoría | Completado | En progreso | Pendiente |
|-----------|:----------:|:-----------:|:---------:|
| Primitivos | 10/17 | 4 (u8-u64) | 3 (byte, void, never) |
| Compuestos | 9/15 | 1 (tuple) | 5 (set, Queue, Stack, Deque, LinkedList) |
| Ownership | 3/7 | 0 | 4 (box, rc, arc, weak) |
| Concurrencia | 3/13 | 2 (channel, iterator) | 8 (future, select, mutex, RwLock, Atomic*2, Barrier, Condvar) |
| Especializados nativos | 0/15 | 10 (migrar de packages) | 5 (file, socket, path, json, big_int) |
| **Total** | **~25** | **~17** | **~25** |
