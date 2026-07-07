# Kyle — Type Inventory

> Auditoría completa de tipos. Documenta cada tipo, su estado real,
> y qué hace falta para tener un lenguaje base completo y funcional.
>
> **Leyenda:**
> - ✅ **Completo** — Funciona sin problemas
> - ⚠️ **Buggy** — Existe pero tiene bugs conocidos
> - 🔶 **Parcial** — Existe incompleto (sin runtime, sin codegen, sin sintaxis)
> - ❌ **No existe** — No implementado
> - 🔜 **Runtime listo** — El runtime Rust tiene la impl pero no es tipo Kyle
> - 📦 **Package** — Existe como package (`from X import Y`), debe pasar a nativo

---

## Convenciones del lenguaje

### Ownership

```ky
x = 1           # inmutable, COPY (i32)
x: ^T = v       # mutable, OWNED
y = x           # MOVE (tipos no-Copy)
y = x.clone()   # COPY explícita
f(&x)           # BORROW
f(^&x)          # MUT BORROW
f(x)            # MOVE (default params)
fn f(x: &T)     # BORROW param
fn f(x: ^&T)    # MUT BORROW param
```

Copy types (`y = x` no mueve): `i8-u64`, `f32-f64`, `bool`, `char`, `ptr`

### Naming (camelCase)

| Regla | Ejemplos |
|-------|----------|
| Tipos 1 palabra | `str`, `regex`, `url`, `uuid`, `bytes`, `mutex`, `future`, `box` |
| Tipos multi-palabra | `str_builder`, `atomic_i64`, `atomic_bool`, `big_int`, `date_time` |
| Funciones | `spawn_thread`, `join_thread`, `parallel_for`, `fetch_add`, `to_str` |
| Constructores tipo | `regex("[0-9]+")`, `box(42)`, `channel<i32>(16)` |
| Constantes (UPPER) | `MAX_SIZE := 1024` |

---

## 1. Primitivos

| # | Tipo | Semántica | Estado | Uso | Notas |
|---|------|-----------|--------|-----|-------|
| 1 | `bool` | Copy | ✅ | `true`/`false` | 1 byte, `i1` en LLVM |
| 2 | `char` | Copy | ⚠️ | `'a'` | Se infiere como i32. Bug en type checker |
| 3 | `byte` | Copy | ❌ | — | Usar `u8` |
| 4 | `str` | Move | ✅ | `"hello"` | Heap-allocated, null-terminated |
| 5 | `i8` | Copy | ✅ | `x: i8 = 127` | Signed 8-bit |
| 6 | `i16` | Copy | ✅ | `x: i16 = 32767` | Signed 16-bit |
| 7 | `i32` | Copy | ✅ | `x = 42` | Default integer literal |
| 8 | `i64` | Copy | ✅ | `x: i64 = 42` | Signed 64-bit |
| 9 | `u8` | Copy | 🔶 | — | **Sin MirType ni codegen.** Existe en Type enum pero no compila como variable |
| 10 | `u16` | Copy | 🔶 | — | Ídem |
| 11 | `u32` | Copy | 🔶 | — | Ídem |
| 12 | `u64` | Copy | 🔶 | — | Ídem |
| 13 | `f32` | Copy | ✅ | `x: f32 = 3.14` | 32-bit float |
| 14 | `f64` | Copy | ✅ | `x = 3.14` | Default float literal |
| 15 | `void` | — | 🔶 | `fn foo() void:` | Solo como retorno. No instanciable |
| 16 | `never` | — | ❌ | — | `!` type para funciones divergentes |
| 17 | `ptr` | Copy | ✅ | `p = 0 as ptr` | Raw pointer, FFI/unsafe |

---

## 2. Compuestos (Estructuras de datos)

| # | Tipo | Semántica | Estado | Uso | Notas |
|---|------|-----------|--------|-----|-------|
| 18 | `final class` | Move | ✅ | `final class Point:` | Struct ligero |
| 19 | `class` | Move | ✅ | `class Cat :: Animal:` | Herencia vía `::` |
| 20 | `abstract class` | Move | ✅ | `abstract class Shape:` | No instanciable |
| 21 | `contract` | Move | ✅ | `contract Drawable:` | Trait/interface |
| 22 | `enum` | Move | ✅ | `enum Color:` | Tagged union con payload |
| 23 | `tuple` | Move | 🔶 | `(1, "a")` | **Sin MirType ni codegen.** Parser-only |
| 24 | `[T; N]` | Move | ✅ | `[1, 2, 3]` / `[0; 100]` | Stack array, GEP directo |
| 25 | `{T}` | Move | ✅ | `{1, 2, 3}` | Heap list, dinámico |
| 26 | `{K: V}` | Move | ✅ | `{"k": 1}` | Heap dict |
| 27 | `set<T>` | Move | 🔶 | — | **Dead enum variant.** No parser, no runtime |
| 28 | `Queue<T>` | Move | ❌ | — | Pendiente |
| 29 | `Stack<T>` | Move | ❌ | — | Pendiente |
| 30 | `Deque<T>` | Move | ❌ | — | Pendiente |
| 31 | `LinkedList<T>` | Move | ❌ | — | Pendiente |
| 32 | `slice` | Move | ❌ | — | Vista de array (como Rust `&[T]`) |

---

## 3. Opcionales / Resultado

| # | Tipo | Semántica | Estado | Uso | Notas |
|---|------|-----------|--------|-----|-------|
| 33 | `T?` / `Option<T>` | Move | ⚠️ | `name: str? = None` | Bug: `str?` causa type mismatch. `none` debe ser `None` |
| 34 | `T!` / `Result<T, E>` | Move | ⚠️ | `fn div(a,b): i32!` | Bug: `-> T!` syntax no funciona. `ok()`/`error()` sí |

---

## 4. Ownership / Referencias

| # | Tipo | Semántica | Estado | Uso | Notas |
|---|------|-----------|--------|-----|-------|
| 35 | `^T` (mutable) | — | ✅ | `x: ^str = "hola"` | Marcador compile-time, cero overhead |
| 36 | `&T` (borrow) | Copy | ✅ | `fn f(x: &str)` | Borrow inmutable |
| 37 | `^&T` (mut borrow) | Copy | ✅ | `fn f(x: ^&str)` | Borrow mutable |
| 38 | `box<T>` | Move | ❌ | — | Heap pointer |
| 39 | `rc<T>` | Copy | ❌ | — | Single-thread refcount |
| 40 | `arc<T>` | Copy | ❌ | — | Multi-thread refcount |
| 41 | `weak<T>` | Copy | ❌ | — | weak ref, evita ciclos |

---

## 5. Concurrencia / Async

| # | Tipo | Semántica | Estado | Uso | Notas |
|---|------|-----------|--------|-----|-------|
| 42 | `async fn` | — | ✅ | `async fn f()` | Thread pool |
| 43 | `await` | — | ✅ | `await task` | |
| 44 | `async:` block | — | ✅ | `t = async: ...` | |
| 45 | `future<T>` | Move | ❌ | `t: future<str> = async: ...` | No existe. Async retorna i64 opaco |
| 46 | `channel<T>` | Move | 🔶 | `channel<i64>(16).send(42)` | Runtime listo, falta tipo Kyle |
| 47 | `select` | — | ❌ | — | Multiplexor de canales |
| 48 | `mutex<T>` | Move | ❌ | `m: mutex<i32>(0)` | Exclusión mutua |
| 49 | `RwLock<T>` | Move | ❌ | — | Readers-writer lock |
| 50 | `AtomicI64` | Copy | ❌ | `counter: AtomicI64 = 0` | Solo interno en runtime Rust |
| 51 | `AtomicBool` | Copy | ❌ | `flag: AtomicBool = false` | Solo interno |
| 52 | `Barrier` | — | ❌ | — | Sincronización de threads |
| 53 | `Condvar` | — | ❌ | — | Condition variable |
| 54 | `iterator` | Move | 🔶 | `iter.map(fn).filter(fn)` | Runtime listo (KlIter), falta tipo Kyle |

---

## 6. Especializados — DEBEN SER NATIVOS

> **Filosofía:** Todos estos tipos deben ser nativos (`ky_*` runtime + compilador),
> NO packages. Solo HTTP/Postgres/SQLite son packages. El resto es infraestructura base.

| # | Tipo | Ahora | Debe ser | Uso | Runtime |
|---|------|-------|----------|-----|---------|
| 55 | `date_time` | 📦 `from datetime import datetime` | ✅ Nativo | `dt = date_time.now()` | `chrono` crate ✅ |
| 56 | `duration` | 📦 `from datetime import duration` | ✅ Nativo | `d = duration.from_secs(60)` | `chrono` ✅ |
| 57 | `date` | 📦 `from date import date` | ✅ Nativo | `d = date.today()` | ✅ |
| 58 | `time` | 📦 `from date import time` | ✅ Nativo | `t = time.now()` | ✅ |
| 59 | `bytes` | 📦 `from bytes import bytes` | ✅ Nativo | `b = bytes.new(1024)` | ✅ |
| 60 | `decimal` | 📦 `from decimal import decimal` | ✅ Nativo | `d = decimal.from_str("3.14")` | ✅ |
| 61 | `uuid` | 📦 `from uuid import uuid` | ✅ Nativo | `id = uuid.v4()` | ✅ |
| 62 | `url` | 📦 `from url import url` | ✅ Nativo | `u = url.parse("https://...")` | ✅ |
| 63 | `regex` | 📦 `from regex import regex` | ✅ Nativo | `re = regex("[0-9]+")` | ✅ |
| 64 | `env` | 📋 `ky_getenv/setenv` | ✅ Nativo | `value = env("PATH")` | ✅ runtime |
| 65 | `file` | ❌ fd i32 | ✅ Nativo | `f = file.open(path, "r")` | 🔶 parcial |
| 66 | `socket` | ❌ fd i32 | ✅ Nativo | `s = socket.tcp_listen(8080)` | 🔶 parcial |
| 67 | `path` | ❌ str | ✅ Nativo | `p = path("/a/b/c")` | ❌ |
| 68 | `json` | ❌ functions | ✅ Nativo | `json.parse(str)` | ❌ |
| 69 | `big_int` | ❌ | ❌ | — | ❌ |
| 70 | `xml` | ❌ | ❌ | — | ❌ |
| 71 | `Tensor` | ❌ | ❌ | — | ❌ |
| 72 | `DataFrame` | ❌ | ❌ | — | ❌ |

---

## 7. Paquetes (SOLO estos)

| Package | Estado | Archivos | Notas |
|---------|--------|----------|-------|
| `http` | ✅ | `packages/http/` | client + Server + router + websocket |
| `sqlite` | 🔶 | `packages/sqlite/` | En desarrollo |
| `postgres` | 📅 | Planeado | Pendiente |

---

## 8. Funciones / Callables

| # | Tipo | Estado | Uso |
|---|------|--------|-----|
| 73 | `fn(...) T` | ✅ | `fn add(a: i32, b: i32) i32:` |
| 74 | `async fn(...) T` | ✅ | `async fn fetch(url: &str) str:` |
| 75 | Closure | 🔶 | `(x: i32): x * 2` |
| 76 | `static fn` | ⚠️ | Bug en parser (espera LParen antes de Static) |
| 77 | `str_builder` | 🔜 | `str_builder(50000).append("x")` (clase nativa) |

---

## Prioridades

| Prioridad | Área | Items |
|-----------|------|-------|
| **P0** | Renombrar existente | `ky_*` → namespaces (`parallel.for`, `thread.spawn`, `thread.join`). Ver ROADMAP.md |
| **P1** | Arreglar bugs | `u8-u64` codegen, `tuple` codegen, `char` type inference, `T?`/`T!` type checker |
| **P2** | Hacer nativos tipos package | `date_time`, `duration`, `date`, `time`, `bytes`, `decimal`, `uuid`, `url`, `regex`, `env` |
| **P3** | Tipos I/O nativos | `file`, `socket`, `path`, `json` |
| **P4** | Estructuras datos faltantes | `set<T>`, `slice` |
| **P5** | Concurrencia nativa | `channel<T>`, `mutex<T>`, `atomic_i64`, `future<T>`, `iterator`, `select` |
| **P6** | Smart pointers | `box<T>`, `rc<T>`, `arc<T>`, `weak<T>` |
| **P7** | Avanzados | `big_int`, `deque`, `linkedList` |

> **Nota:** Queue y Stack NO tienen tipos dedicados. Usar `{T}` con `push()`/`pop()` (stack) o `push()`/`pop_first()` (queue).

## Optimizaciones futuras (postergadas)

> Ver `ROADMAP.md` sección "Optimizaciones" para detalle completo.

| # | Mejora | Impacto | Benchmarks |
|---|--------|---------|------------|
| 1 | Register alloc para `^i32/^i64` | 1.6× → 1.0× | Fib |
| 2 | `list.reserve(n)` + batch push | 2.7× → 1.5× | Primes |
| 3 | Arrays `[T;N]` pass-by-reference | 7.8× → 1.0× | Matmul |
| 4 | str_builder inline hints | 1.1× → 0.5× | Concat |
