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

## Convenciones de ownership

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
| 27 | `Set<T>` | Move | 🔶 | — | **Dead enum variant.** No parser, no runtime |
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
| 38 | `Box<T>` | Move | ❌ | — | Heap pointer |
| 39 | `Rc<T>` | Copy | ❌ | — | Single-thread refcount |
| 40 | `Arc<T>` | Copy | ❌ | — | Multi-thread refcount |
| 41 | `Weak<T>` | Copy | ❌ | — | Weak ref, evita ciclos |

---

## 5. Concurrencia / Async

| # | Tipo | Semántica | Estado | Uso | Notas |
|---|------|-----------|--------|-----|-------|
| 42 | `async fn` | — | ✅ | `async fn f()` | Thread pool |
| 43 | `await` | — | ✅ | `await task` | |
| 44 | `async:` block | — | ✅ | `t = async: ...` | |
| 45 | `Future<T>` | Move | ❌ | `t: Future<str> = async: ...` | No existe. Async retorna i64 opaco |
| 46 | `Channel<T>` | Move | 🔶 | `ky_channel_new/send/recv` | Runtime listo, falta tipo Kyle |
| 47 | `select` | — | ❌ | — | Multiplexor de canales |
| 48 | `Mutex<T>` | Move | ❌ | `m: Mutex<i32>(0)` | Exclusión mutua |
| 49 | `RwLock<T>` | Move | ❌ | — | Readers-writer lock |
| 50 | `AtomicI64` | Copy | ❌ | `counter: AtomicI64 = 0` | Solo interno en runtime Rust |
| 51 | `AtomicBool` | Copy | ❌ | `flag: AtomicBool = false` | Solo interno |
| 52 | `Barrier` | — | ❌ | — | Sincronización de threads |
| 53 | `Condvar` | — | ❌ | — | Condition variable |
| 54 | `Iterator` | Move | 🔶 | `iter.map(fn).filter(fn)` | Runtime listo (KlIter), falta tipo Kyle |

---

## 6. Especializados — DEBEN SER NATIVOS

> **Filosofía:** Todos estos tipos deben ser nativos (`ky_*` runtime + compilador),
> NO packages. Solo HTTP/Postgres/SQLite son packages. El resto es infraestructura base.

| # | Tipo | Ahora | Debe ser | Uso | Runtime |
|---|------|-------|----------|-----|---------|
| 55 | `DateTime` | 📦 `from datetime import datetime` | ✅ Nativo | `dt = DateTime.now()` | `chrono` crate ✅ |
| 56 | `Duration` | 📦 `from datetime import duration` | ✅ Nativo | `d = Duration.from_secs(60)` | `chrono` ✅ |
| 57 | `Date` | 📦 `from date import date` | ✅ Nativo | `d = Date.today()` | ✅ |
| 58 | `Time` | 📦 `from date import time` | ✅ Nativo | `t = Time.now()` | ✅ |
| 59 | `Bytes` | 📦 `from bytes import bytes` | ✅ Nativo | `b = Bytes.new(1024)` | ✅ |
| 60 | `Decimal` | 📦 `from decimal import decimal` | ✅ Nativo | `d = Decimal.from_str("3.14")` | ✅ |
| 61 | `Uuid` | 📦 `from uuid import uuid` | ✅ Nativo | `id = Uuid.v4()` | ✅ |
| 62 | `Url` | 📦 `from url import url` | ✅ Nativo | `u = Url.parse("https://...")` | ✅ |
| 63 | `Regex` | 📦 `from regex import regex` | ✅ Nativo | `re = Regex("[0-9]+")` | ✅ |
| 64 | `Env` | 📋 `ky_getenv/setenv` | ✅ Nativo | `value = env("PATH")` | ✅ runtime |
| 65 | `File` | ❌ fd i32 | ✅ Nativo | `f = File.open(path, "r")` | 🔶 parcial |
| 66 | `Socket` | ❌ fd i32 | ✅ Nativo | `s = Socket.tcp_listen(8080)` | 🔶 parcial |
| 67 | `Path` | ❌ str | ✅ Nativo | `p = Path("/a/b/c")` | ❌ |
| 68 | `Json` | ❌ functions | ✅ Nativo | `Json.parse(str)` | ❌ |
| 69 | `BigInt` | ❌ | ❌ | — | ❌ |
| 70 | `Xml` | ❌ | ❌ | — | ❌ |
| 71 | `Tensor` | ❌ | ❌ | — | ❌ |
| 72 | `DataFrame` | ❌ | ❌ | — | ❌ |

---

## 7. Paquetes (SOLO estos)

| Package | Estado | Archivos | Notas |
|---------|--------|----------|-------|
| `http` | ✅ | `packages/http/` | Client + Server + Router + WebSocket |
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
| 77 | `strBuilder` | 🔜 | `ky_str_builder_new()` (solo FFI, no tipo Kyle) |

---

## Prioridades

| Prioridad | Área | Items |
|-----------|------|-------|
| **P0** | Arreglar bugs | `u8-u64` codegen, `tuple` codegen, `char` type inference, `T?`/`T!` type checker |
| **P1** | Hacer nativos tipos package | `DateTime`, `Duration`, `Date`, `Time`, `Bytes`, `Decimal`, `Uuid`, `Url`, `Regex`, `Env` |
| **P2** | Tipos I/O nativos | `File`, `Socket`, `Path`, `Json` |
| **P3** | Structuras datos faltantes | `Set<T>`, `Queue<T>`, `Stack<T>`, `slice` |
| **P4** | Concurrencia nativa | `Channel<T>` typing, `Mutex<T>`, `Atomic*`, `Future<T>`, `Iterator` |
| **P5** | Smart pointers | `Box<T>`, `Rc<T>`, `Arc<T>` |
| **P6** | Avanzados | `BigInt`, `Deque`, `LinkedList` |

## Optimizaciones futuras (postergadas)

> Ver `ROADMAP.md` sección "Optimizaciones" para detalle completo.

| # | Mejora | Impacto | Benchmarks |
|---|--------|---------|------------|
| 1 | Register alloc para `^i32/^i64` | 1.6× → 1.0× | Fib |
| 2 | `list.reserve(n)` + batch push | 2.7× → 1.5× | Primes |
| 3 | Arrays `[T;N]` pass-by-reference | 7.8× → 1.0× | Matmul |
| 4 | strBuilder inline hints | 1.1× → 0.5× | Concat |
