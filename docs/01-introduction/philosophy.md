# Philosophy

Kyle se construye sobre cuatro pilares fundamentales:

## Python Readability

El código Kyle se lee como prosa. Indentación para bloques, sin `{}`, sin `;`.
Variables sin keywords. Funciones con sintaxis minimalista.

```ky
fn procesar(archivo: &str) str!:
    datos: str = file.open(archivo, "r").read()
    if datos.is_empty():
        return error("archivo vacío")
    datos.trim()
```

**40% menos líneas que C++**, **30% menos que Rust**, comparable a Python.

## Rust Type Safety

Sistema de tipos fuerte, estático, con inferencia. Todo se conoce en compile-time:
- **No `null`** — `T?` para opcionales
- **No undefined behavior** — borrow checker, bounds check
- **No data races** — move por defecto, one mut XOR many immut
- **No dangling pointers** — ownership tracking

```ky
nombre: str? = obtener_nombre()
if nombre.is_some():
    println(nombre.unwrap())   # seguro: ya verificamos
```

## Go Simplicity

Pocas features, cada una bien diseñada. Una forma de hacer cada cosa.
- Sin `let`, `var`, `const`, `mut` — solo `^` para mutable
- Sin `class` vs `struct` — solo `final class`
- Sin `try/catch` — `T!` para fallos
- Sin `null` — `T?` para opcionales
- Sin `interface` — `contract` para traits
- Sin `async/await` complejo — `async fn` simple

## C Performance

Compila a código nativo vía LLVM 18 con pipeline completo de optimización.

```
Source → Lexer → Parser → HIR → Semantic → MIR → SSA → LLVM IR → O3 → Binary
```

| Medición | Kyle | C | Rust | Python |
|----------|:----:|:-:|:----:|:------:|
| Fib (500M) | 180ms | 115ms | 116ms | ❌ |
| Concat (500k) | 9ms | 8ms | 1ms | 22ms |
| Primes (3M) | 19ms | 8ms | 8ms | 240ms |
| Matmul (100×100×30) | 47ms | 6ms | 6ms | 1154ms |

**Sin GC, sin runtime overhead, sin conteo de referencias implícito.**

## Ver también

- `principles.md` — Principios de diseño
- `03-language/` — Referencia del lenguaje
- `04-standard-library/` — Librería estándar
