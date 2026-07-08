# FAQ

> Preguntas frecuentes sobre Kyle.

## Generales

### ¿Kyle es un lenguaje de alto o bajo nivel?

**Bajo nivel.** Compila a nativo vía LLVM, tiene control de memoria manual
(ownership, borrow checker), punteros raw, FFI directo con C. Pero con sintaxis
simple y legible como Python.

### ¿Kyle compite con Rust?

Sí, en el sentido de ser un lenguaje de sistemas seguro y rápido. Pero Kyle
prioriza la simplicidad sintáctica sobre la exhaustividad del type system.
Menos features, menos complejidad.

### ¿Kyle tiene garbage collector?

**No.** La memoria se gestiona mediante ownership (move por defecto) y el
borrow checker inserta `ky_free` automáticamente.

## Sintaxis

### ¿Por qué snake_case y no camelCase?

snake_case es más legible para código con nombres largos y es consistente
con la filosofía de "sin ruido sintáctico".

### ¿Por qué no hay `let`/`var`/`mut`?

Para reducir ruido. La declaración es `nombre = valor`. La mutabilidad se
marca con `^`: `x: ^i32 = 0`.

### ¿Por qué `^` para mutable y `&` para borrow?

`^` es un sigilo minimalista que no compite con operadores existentes.
`&` para borrow es familiar para programadores Rust.

## Rendimiento

| Benchmark | Kyle vs C | Gap | Causa principal |
|-----------|:---------:|:---:|-----------------|
| Fib | 1.6× | Register alloc | Optimizar `^i32` en codegen |
| Concat | 1.1× | FFI call overhead | strBuilder inline hints |
| Primes | 2.7× | List push overhead | `reserve()` + batch push |
| Matmul | 7.8× | List get/set calls | Arrays nativos pass-by-ref |

## Paquetes

### ¿Por qué DateTime, Regex, UUID son nativos y no packages?

Porque son tipos base que cualquier aplicación necesita. Solo HTTP, SQLite y
PostgreSQL son packages porque son protocolos/bases de datos específicos.

### ¿Dónde están los archivos del runtime?

En `crates/kyc_runtime/src/`. 3350 líneas de Rust, 88 funciones `extern "C"`.

## Ver también

- `philosophy.md` — Filosofía del lenguaje
- `architecture.md` — Arquitectura del ecosistema
