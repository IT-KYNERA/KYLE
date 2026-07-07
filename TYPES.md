# Kyle — Type Inventory

> Documento de referencia que lista todos los tipos que debería tener Kyle,
> estado actual de cada uno, y qué hace falta implementar/completar.
>
> **Leyenda:**
> - ✅ **Completo** — Funciona sin problemas conocidos
> - ⚠️ **Con bugs** — Existe pero tiene bugs conocidos
> - 🔶 **Parcial** — Existe en el compilador pero incompleto (sin runtime, sin sintaxis, etc.)
> - ❌ **No existe** — No implementado en ninguna capa
> - 🔜 **Runtime listo** — El runtime Rust tiene la implementación pero no está integrado como tipo Kyle

---

## 1. Primitivos

| # | Tipo | Estado | Cómo se usa | Notas |
|---|------|--------|-------------|-------|
| 1 | `bool` | ✅ | `x := true` / `x := false` | 1 byte, `i1` en LLVM |
| 2 | `char` | ⚠️ | `c := 'a'` | Funciona pero `'a'` se infiere como i32 internamente. `char.to_i32()` = 97 |
| 3 | `byte` | ❌ | — | No existe. Usar `u8` |
| 4 | `str` | ✅ | `s := "hello"` | Heap-allocated, null-terminated |
| 5 | `i8` | ✅ | `x: i8 = 127` | Signed 8-bit |
| 6 | `i16` | ✅ | `x: i16 = 32767` | Signed 16-bit |
| 7 | `i32` | ✅ | `x := 42` (default) | Signed 32-bit, default para literales |
| 8 | `i64` | ✅ | `x: i64 = 42` | Signed 64-bit |
| 9 | `u8` | ✅ | `x: u8 = 255` | Unsigned 8-bit |
| 10 | `u16` | ✅ | `x: u16 = 65535` | Unsigned 16-bit |
| 11 | `u32` | ✅ | `x: u32 = 4294967295` | Unsigned 32-bit |
| 12 | `u64` | ✅ | `x: u64 = 18446744073709551615` | Unsigned 64-bit |
| 13 | `f32` | ✅ | `x: f32 = 3.14` | 32-bit float |
| 14 | `f64` | ✅ | `x := 3.14` (default) | 64-bit float, default para literales |
| 15 | `void` | ✅ | `fn foo() void:` | Tipo de retorno vacío |
| 16 | `never` | ❌ | — | No existe. `!` type para funciones que nunca retornan |
| 17 | `ptr` | ✅ | `p := 0 as ptr` | Raw pointer 8 bytes, FFI/unsafe |

---

## 2. Compuestos (Estructuras de datos)

| # | Tipo | Estado | Cómo se usa | Notas |
|---|------|--------|-------------|-------|
| 18 | `struct` (final class) | ✅ | `final class Point:` | Struct ligero, cero overhead |
| 19 | `class` | ✅ | `class Cat :: Animal:` | Con herencia vía `::` |
| 20 | `abstract class` | ✅ | `abstract class Shape:` | Base class, no se puede instanciar |
| 21 | `contract` | ✅ | `contract Drawable:` | Trait/interface |
| 22 | `enum` | ✅ | `enum Color:` | Tagged union con variants + payload |
| 23 | `tuple` | ✅ | `p := (1, "a")` | Acceso `.0`, `.1` |
| 24 | `array` | ✅ | `arr := [1, 2, 3]` | Stack array `[T; N]`, GEP directo |
| 25 | `slice` | ❌ | — | No existe |
| 26 | `Vec` / `list` | ✅ | `v := {1, 2, 3}` | Heap list `{T}`, métodos push/pop/len |
| 27 | `Map` / `dict` | ✅ | `d := {"k": 1}` | Heap dict `{K: V}` |
| 28 | `Set` | 🔶 | — | Existe `Type::Set(Box<Type>)` en types.rs pero sin runtime, sin sintaxis, sin documentación. **Variant muerto** |
| 29 | `Queue` | ❌ | — | No existe |
| 30 | `Stack` | ❌ | — | No existe |
| 31 | `Deque` | ❌ | — | No existe |
| 32 | `LinkedList` | ❌ | — | No existe |

---

## 3. Opcionales / Resultado / Fallibles

| # | Tipo | Estado | Cómo se usa | Notas |
|---|------|--------|-------------|-------|
| 33 | `Option<T>` / `T?` | ✅ | `name: str? = None` | Nullable type. `match name: None: ... _: ...` |
| 34 | `Result<T, E>` / `T!` | ✅ | `fn div(a,b): i32! = ...` | Fallible type. `ok(v)`/`error(e)` patterns |
| 35 | `Optional<T>` | ❌ | — | No existe. Es `T?` / `Option<T>` |
| 36 | `Fallible<T>` | ❌ | — | No existe. Es `T!` / `Result<T, E>` |

---

## 4. Referencias / Punteros / Ownership

| # | Tipo | Estado | Cómo se usa | Notas |
|---|------|--------|-------------|-------|
| 37 | `&T` (mutable ref) | ✅ | `count: &i32 = 0` | Marca como mutable, se resuelve al inner type |
| 38 | `^T` (move) | ✅ | `fn take(^data: str)` | Ownership transfer, se resuelve al inner type |
| 39 | `ptr` (raw pointer) | ✅ | `p := 0 as ptr` | Raw pointer, FFI |
| 40 | `*T` (typed pointer) | ❌ | — | No existe. Solo `ptr` genérico |
| 41 | `Box<T>` | ❌ | — | No existe. Heap implícito para str/list/dict |
| 42 | `Rc<T>` | ❌ | — | No existe |
| 43 | `Arc<T>` | ❌ | — | No existe (solo interno en runtime Rust) |
| 44 | `Weak<T>` | ❌ | — | No existe |
| 45 | `Cell<T>` | ❌ | — | No existe |
| 46 | `RefCell<T>` | ❌ | — | No existe |
| 47 | `Ref<T>` | ❌ | — | No existe |

---

## 5. Concurrencia / Async

| # | Tipo | Estado | Cómo se usa | Notas |
|---|------|--------|-------------|-------|
| 48 | `Future` | ❌ | — | No existe. Async usa thread pool + i64 handle |
| 49 | `Stream` | ❌ | — | No existe |
| 50 | `Generator` | ❌ | — | No existe |
| 51 | `Iterator` | ❌ | — | No existe. Sin trait Iterator |
| 52 | `Task` | 🔶 | `task = async: ...` | Opaque i64 handle, usable con `await` |
| 53 | `Channel` | ✅ | `ky_channel_new/send/recv` | C wrappers con valores i64 |
| 54 | `Atomic` | ❌ | — | No existe (solo interno en runtime Rust) |
| 55 | `AtomicI64` | ❌ | — | No existe (solo interno en runtime Rust) |
| 56 | `AtomicBool` | ❌ | — | No existe (solo interno en runtime Rust) |
| 57 | `Mutex` | ❌ | — | No existe |
| 58 | `RwLock` | ❌ | — | No existe |
| 59 | `Condvar` | ❌ | — | No existe |
| 60 | `Barrier` | ❌ | — | No existe |

---

## 6. Especializados (Runtime listo, no integrados como tipo Kyle)

Estos existen en `kyc_runtime/src/` como funciones `extern "C"` con FFI,
pero **no son tipos nativos de Kyle**. Se accede vía `extern fn` + `@link`.

| # | Tipo | Estado | Archivo runtime | Notas |
|---|------|--------|-----------------|-------|
| 61 | `datetime` | 🔜 | `datetime.rs` | `chrono` crate, `now()/parse()/format()` |
| 62 | `duration` | 🔜 | `datetime.rs` | `from_millis()/seconds()/minutes()` |
| 63 | `date` | 🔜 | `date.rs` | `today()/parse()` |
| 64 | `bytes` | 🔜 | `bytes.rs` | `new()/get()/set()/len()` |
| 65 | `decimal` | 🔜 | `decimal.rs` | Fixed precision i64*100 |
| 66 | `uuid` | 🔜 | `uuid.rs` | `v4()/parse()` |
| 67 | `url` | 🔜 | `url.rs` | `parse()/scheme()/host()/path()` |
| 68 | `regex` | 🔜 | `regex.rs` | `new()/is_match()/find()/replace()` |

---

## 7. Especializados (No existen en ninguna capa)

| # | Tipo | Estado | Notas |
|---|------|--------|-------|
| 69 | `Json` | ❌ | No hay tipo Json. Solo funciones: `json_parse()/stringify()/serialize()/deserialize()` |
| 70 | `Xml` | ❌ | No existe |
| 71 | `Regex` | ❌ | El runtime tiene `ky_regex_*` funciones (item 68) pero no hay tipo `Regex` nativo |
| 72 | `Path` | ❌ | File I/O usa strings directamente |
| 73 | `File` | ❌ | Solo funciones: `open()/read_str()/write_str()/close()` que retornan i32 fd |
| 74 | `Socket` | ❌ | Solo funciones: `ky_tcp_listen()/accept()/read()/write()/close()` con i32 fd |
| 75 | `DateTime` | ❌ | El runtime tiene `datetime` (item 61) pero no es tipo integrado |
| 76 | `Uuid` | ❌ | El runtime tiene `ky_uuid_*` (item 66) pero no es tipo integrado |
| 77 | `BigInt` | ❌ | No existe |
| 78 | `BigDecimal` | ❌ | El runtime tiene `decimal` (item 65) pero no es `BigDecimal` |
| 79 | `Tensor` | ❌ | No existe |
| 80 | `DataFrame` | ❌ | No existe |
| 81 | `Span` | ❌ | `Span` existe solo como struct interno en `kyc_core::span` (Rust), no en Kyle |
| 82 | `Memory` | ❌ | Gestión manual vía `ky_alloc()/ky_free()`, no hay tipo `Memory` |
| 83 | `HashMap` | ❌ | No existe. Usar `{K: V}` (es dict) |
| 84 | `HashSet` | ❌ | No existe. `Set<T>` (item 28) está muerto |
| 85 | `BTreeMap` | ❌ | No existe |
| 86 | `BTreeSet` | ❌ | No existe |
| 87 | `RingBuf` | ❌ | No existe |
| 88 | `PriorityQueue` | ❌ | No existe |

---

## 8. Funciones / Callables

| # | Tipo | Estado | Cómo se usa | Notas |
|---|------|--------|-------------|-------|
| 89 | `fn(...) T` | ✅ | `fn add(a: i32, b: i32) i32:` | Function pointer tipo |
| 90 | `async fn(...) T` | ✅ | `async fn fetch(url: str) str:` | Async function |
| 91 | Closure | 🔶 | `(x: i32): x * 2` | Mencionado en docs http, función flecha vía function pointers |
| 92 | `static fn` | ⚠️ | `static fn name` | Bug: parser espera LParen antes de Static |

---

## 9. Paquetes oficiales (vía `from X import Y`)

| # | Tipo | Estado | Package | Notas |
|---|------|--------|---------|-------|
| 93 | `Client` | ✅ | `http.client` | HTTP client: get/post/put/patch/delete |
| 94 | `Router` | ✅ | `http.server` | HTTP server router con middleware |
| 95 | `Request` | ✅ | `http` (implícito) | Server-side request |
| 96 | `Response` / `Res` | ✅ | `http` (implícito) | `json()/text()/status()` |
| 97 | `WebSocket` | ✅ | `http.websocket` | WebSocket handler |
| 98 | `HttpMethod` | ✅ | `http` (enum) | `GET | POST | PUT | DELETE | PATCH | HEAD | OPTIONS` |
| 99 | `HttpStatus` | ✅ | `http` (class) | `code: i32`, `text: str` |
| 100 | `Header` | ✅ | `http` (class) | `name: str`, `value: str` |
| 101 | `Database` | 🔶 | `sqlite` | Planeado pero no implementado aún |

---

## Resumen

| Estado | Cantidad |
|--------|----------|
| ✅ Completo | ~45 |
| ⚠️ Con bugs | 3 (`char`, `static fn`, patrones range `..=`) |
| 🔶 Parcial / Muerto | 4 (`Set`, `Task`, Closure, `Database`) |
| 🔜 Runtime listo sin integración | 8 (`datetime`, `duration`, `date`, `bytes`, `decimal`, `uuid`, `url`, `regex`) |
| ❌ No existe | ~35 |

## Prioridades sugeridas

1. **Bugs bloqueantes** — `char`, `static fn`
2. **Tipos runtime listos** — Integrar `datetime`, `duration`, `bytes`, `decimal`, `uuid`, `url`, `regex` como tipos Kyle nativos
3. **Estructuras de datos faltantes** — `slice`, `Set` funcional, `Queue`, `Stack`, `Deque`, `LinkedList`
4. **Concurrencia** — `Mutex`, `Atomic`, `Future` type
5. **Especializados** — `Json` type, `Path`, `File`, `Socket`, `BigInt`
