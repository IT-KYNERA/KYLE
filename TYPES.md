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

## Convenciones de ownership

| Símbolo | Contexto | Significado |
|---------|----------|-------------|
| `x = 1` | Declaración | Variable **inmutable** (por defecto) |
| `x: ^T = v` | Declaración | Variable **mutable** (`^` = mutable) |
| `y = x` | Asignación | **MOVE** (`x` inválido después) para tipos no-Copy |
| `y = x.clone()` | Asignación | **COPY** explícita (ambos vivos) |
| `f(x)` | Llamada | **MOVE** (`x` se transfiere a `f`) |
| `f(&x)` | Llamada | **BORROW** (`x` prestado, sigue vivo) |
| `f(^&x)` | Llamada | **MUT BORROW** (`x` prestado mutable) |
| `fn f(x: T)` | Parámetro | **OWNED** (el caller mueve) |
| `fn f(x: &T)` | Parámetro | **BORROW** (el caller presta) |
| `fn f(x: ^&T)` | Parámetro | **MUT BORROW** (el caller presta mutable) |

### Copy vs Move

| Semántica | Tipos |
|-----------|-------|
| **Copy** (automático) | `i8..u64`, `f32..f64`, `bool`, `char`, `ptr` |
| **Move** (por defecto) | `str`, `{T}`, `{K:V}`, `[T; N]`, classes, structs |

Tipos Copy se duplican solos en asignación. Tipos Move transfieren ownership.

---

## 1. Primitivos

| # | Tipo | Copy? | Estado | Cómo se usa | Notas |
|---|------|-------|--------|-------------|-------|
| 1 | `bool` | ✅ Copy | ✅ | `x = true` | 1 byte, `i1` en LLVM |
| 2 | `char` | ✅ Copy | ⚠️ | `c = 'a'` | `'a'` se infiere como i32. `char.to_i32()` = 97 |
| 3 | `byte` | ✅ Copy | ❌ | — | No existe. Usar `u8` |
| 4 | `str` | 🚫 Move | ✅ | `s = "hello"` | Heap-allocated, null-terminated |
| 5 | `i8` | ✅ Copy | ✅ | `x: i8 = 127` | Signed 8-bit |
| 6 | `i16` | ✅ Copy | ✅ | `x: i16 = 32767` | Signed 16-bit |
| 7 | `i32` | ✅ Copy | ✅ | `x = 42` (default) | Signed 32-bit, default para literales |
| 8 | `i64` | ✅ Copy | ✅ | `x: i64 = 42` | Signed 64-bit |
| 9 | `u8` | ✅ Copy | ✅ | `x: u8 = 255` | Unsigned 8-bit |
| 10 | `u16` | ✅ Copy | ✅ | `x: u16 = 65535` | Unsigned 16-bit |
| 11 | `u32` | ✅ Copy | ✅ | `x: u32 = 4294967295` | Unsigned 32-bit |
| 12 | `u64` | ✅ Copy | ✅ | `x: u64 = 18446744073709551615` | Unsigned 64-bit |
| 13 | `f32` | ✅ Copy | ✅ | `x: f32 = 3.14` | 32-bit float |
| 14 | `f64` | ✅ Copy | ✅ | `x = 3.14` (default) | 64-bit float, default para literales |
| 15 | `void` | — | ✅ | `fn foo() void:` | Tipo de retorno vacío |
| 16 | `never` | — | ❌ | — | `!` type para funciones que nunca retornan |
| 17 | `ptr` | ✅ Copy | ✅ | `p = 0 as ptr` | Raw pointer 8 bytes, FFI/unsafe |

---

## 2. Compuestos (Estructuras de datos)

| # | Tipo | Copy? | Estado | Cómo se usa | Notas |
|---|------|-------|--------|-------------|-------|
| 18 | `final class` | 🚫 Move | ✅ | `final class Point:` | Struct ligero, cero overhead |
| 19 | `class` | 🚫 Move | ✅ | `class Cat :: Animal:` | Con herencia vía `::` |
| 20 | `abstract class` | 🚫 Move | ✅ | `abstract class Shape:` | Base class, no instanciable |
| 21 | `contract` | 🚫 Move | ✅ | `contract Drawable:` | Trait/interface |
| 22 | `enum` | 🚫 Move | ✅ | `enum Color:` | Tagged union con variants + payload |
| 23 | `tuple` | 🚫 Move | ✅ | `p = (1, "a")` | Acceso `.0`, `.1` |
| 24 | `[T; N]` | 🚫 Move | ✅ | `arr = [1, 2, 3]` | Stack array, GEP directo. `arr = [0; 100]` repite |
| 25 | `{T}` | 🚫 Move | ✅ | `v = {1, 2, 3}` | Heap list, métodos push/pop/len |
| 26 | `{K: V}` | 🚫 Move | ✅ | `d = {"k": 1}` | Heap dict |
| 27 | `Set<T>` | 🚫 Move | 🔶 | — | Existe `Type::Set` en types.rs sin runtime |
| 28 | `Queue<T>` | 🚫 Move | ❌ | — | `ky_queue_new/push/pop` en runtime planeado |
| 29 | `Stack<T>` | 🚫 Move | ❌ | — | `ky_stack_new/push/pop` en runtime planeado |
| 30 | `Deque<T>` | 🚫 Move | ❌ | — | No existe |
| 31 | `LinkedList<T>` | 🚫 Move | ❌ | — | No existe |
| 32 | `slice` | 🚫 Move | ❌ | — | Vista de array existente (como Rust `&[T]`) |

---

## 3. Opcionales / Resultado / Fallibles

| # | Tipo | Copy? | Estado | Cómo se usa | Notas |
|---|------|-------|--------|-------------|-------|
| 33 | `T?` / `Option<T>` | 🚫 Move | ✅ | `name: str? = None` | Nullable. `match: None: ... _: ...` |
| 34 | `T!` / `Result<T, E>` | 🚫 Move | ✅ | `fn div(a,b): i32!` | Fallible. `ok(v)`/`error(e)` patterns |

---

## 4. Referencias / Punteros / Ownership

| # | Tipo | Copy? | Estado | Cómo se usa | Notas |
|---|------|-------|--------|-------------|-------|
| 35 | `^T` (mutable) | — | ✅ | `x: ^str = "hola"` | Marca variable como mutable. `^` en tipo |
| 36 | `&T` (borrow) | ✅ Copy | ✅ | `f(&x)` / `fn f(x: &str)` | Borrow inmutable. `&` en expr o param |
| 37 | `^&T` (mut borrow) | ✅ Copy | ✅ | `f(^&x)` / `fn f(x: ^&str)` | Borrow mutable. Compone `^` + `&` |
| 38 | `Box<T>` | 🚫 Move | ❌ | — | Heap allocation explícita |
| 39 | `Rc<T>` | ✅ Copy | ❌ | — | Reference counting (single-thread) |
| 40 | `Arc<T>` | ✅ Copy | ❌ | — | Atomic reference counting (multi-thread) |
| 41 | `Weak<T>` | ✅ Copy | ❌ | — | Weak reference (evita ciclos) |

### Sintaxis Kyle para ownership

```ky
# Variables
x = "hola"          # inmutable, OWNED
x: ^str = "hola"    # mutable, OWNED

# Parámetros (move por defecto)
fn f(s: str)        # MOVE: caller pierde ownership
fn f(s: &str)       # BORROW: caller presta
fn f(s: ^&str)      # MUT BORROW: caller presta mutable

# Expresiones
y = x               # MOVE: x inválido después
f(&x)               # BORROW: x sigue vivo
f(^&x)              # MUT BORROW: x mutable prestado

# Clone
y = x.clone()       # COPY explícita
```

---

## 5. Concurrencia / Async

| # | Tipo | Copy? | Estado | Cómo se usa | Notas |
|---|------|-------|--------|-------------|-------|
| 42 | `async fn` | — | ✅ | `async fn fetch(url: str) str:` | Thread pool |
| 43 | `await` | — | ✅ | `await task` | Espera resultado |
| 44 | `async:` block | — | ✅ | `t = async: ...` | Bloque async en línea |
| 45 | `Future<T>` | 🚫 Move | ❌ | `t: Future<str> = async: ...` | Tipo handle para async. Hoy es `i64` opaco |
| 46 | `Channel<T>` | 🚫 Move | 🔶 | `ch: Channel<i64> = ky_channel_new()` | Canales tipados. Hoy solo `i64` |
| 47 | `select` | — | ❌ | `select: ...` | Multiplexor de canales |
| 48 | `Mutex<T>` | 🚫 Move | ❌ | `m: Mutex<i32> = Mutex(0)` | Exclusión mutua |
| 49 | `RwLock<T>` | 🚫 Move | ❌ | `lock: RwLock<i32>` | Readers-writer lock |
| 50 | `AtomicI64` | ✅ Copy | ❌ | `counter: AtomicI64 = 0` | Operaciones atómicas |
| 51 | `AtomicBool` | ✅ Copy | ❌ | `flag: AtomicBool = false` | Flag atómico |
| 52 | `Barrier` | — | ❌ | `b = Barrier(4)` | Sincronización de threads |
| 53 | `Condvar` | — | ❌ | — | Condition variable |

### Sintaxis Kyle para concurrencia

```ky
ch = Channel<i64>(16)           # canal con buffer 16
ch.send(42)                     # enviar
val = ch.recv()                  # recibir

select:
    &msg -> ch1:                # ch1 tiene mensaje
        println(msg)
    &msg -> ch2:                # ch2 tiene mensaje
        println(msg)
    after 1s:                   # timeout
        println("timeout")

m = Mutex<i32>(0)
lock(m):
    *val += 1                   # dentro del lock

counter = AtomicI64(0)
counter.fetch_add(1)            # operación atómica
```

---

## 6. Especializados (Runtime listo, no integrados como tipo Kyle)

| # | Tipo | Estado | Notas | Sintaxis planeada |
|---|------|--------|-------|-------------------|
| 54 | `DateTime` | 🔜 | `chrono` crate | `dt = DateTime.parse("2024-01-01")` |
| 55 | `Duration` | 🔜 | Intervalos | `d = Duration.from_secs(60)` |
| 56 | `Date` | 🔜 | Solo fecha | `d = Date.today()` |
| 57 | `Bytes` | 🔜 | Buffer binario | `b = Bytes.new(1024)` |
| 58 | `Decimal` | 🔜 | Decimal fixed-point | `d = Decimal.from_str("3.14")` |
| 59 | `Uuid` | 🔜 | v4 UUID | `id = Uuid.v4()` |
| 60 | `Url` | 🔜 | URL parser | `u = Url.parse("https://...")` |
| 61 | `Regex` | 🔜 | Regex pattern | `re = Regex("[0-9]+")` |

---

## 7. No existen

| # | Tipo | Estado | Notas |
|---|------|--------|-------|
| 62 | `Json` type | ❌ | Solo funciones `parse/stringify/serialize`. Podría ser tipo |
| 63 | `Xml` | ❌ | No planeado |
| 64 | `Path` | ❌ | File I/O usa strings |
| 65 | `File` | ❌ | Funciones `open/read_str/write_str/close` retornan i32 fd |
| 66 | `Socket` | ❌ | `ky_tcp_listen/accept/read/write/close` con i32 fd |
| 67 | `BigInt` | ❌ | No planeado |
| 68 | `Tensor` | ❌ | No planeado |
| 69 | `DataFrame` | ❌ | No planeado |

---

## 8. Funciones / Callables

| # | Tipo | Estado | Cómo se usa |
|---|------|--------|-------------|
| 70 | `fn(...) T` | ✅ | `fn add(a: i32, b: i32) i32:` |
| 71 | `async fn(...) T` | ✅ | `async fn fetch(url: &str) str:` |
| 72 | Closure | 🔶 | `(x: i32): x * 2` |
| 73 | `static fn` | ⚠️ | `static fn name` — bug en parser |

---

## 9. Paquetes oficiales

| # | Tipo | Estado | Package | Notas |
|---|------|--------|---------|-------|
| 74 | `Client` | ✅ | `http.client` | HTTP client |
| 75 | `Router` | ✅ | `http.server` | Router con middleware |
| 76 | `Request` | ✅ | `http` | Server-side request |
| 77 | `Response` | ✅ | `http` | `json()/text()/status()` |
| 78 | `WebSocket` | ✅ | `http.websocket` | WebSocket handler |
| 79 | `Database` | 🔶 | `sqlite` | Planeado |

---

## Resumen

| Estado | Cantidad |
|--------|----------|
| ✅ Completo | ~45 |
| ⚠️ Con bugs | 3 |
| 🔶 Parcial / implementar | ~15 |
| 🔜 Runtime listo sin integrar | 8 |
| ❌ No existe | ~20 |

## Prioridades (Roadmap)

| Prioridad | Área | Items |
|-----------|------|-------|
| **P0** | Ownership nuevo | ✅ Completado en v0.6 — `^` = mutable, `&` = borrow, move por defecto, borrow checker |
| **P1** | Integrar runtime | `DateTime`, `Duration`, `Uuid`, `Url`, `Regex`, `Bytes`, `Decimal` |
| **P2** | Colecciones | `Set<T>`, `Queue<T>`, `Stack<T>` |
| **P3** | Concurrencia | `Channel<T>` tipado, `Mutex<T>`, `Atomic*`, `select` |
| **P4** | Smart pointers | `Box<T>`, `Rc<T>`, `Arc<T>` |
| **P5** | Avanzados | `File`, `Socket`, `Path`, `BigInt`, `Json`, `Tensor` |
