# Changelog

> Registro de cambios del lenguaje Kyle.

## v0.5.3 (2026-07-06)

| Área | Cambio |
|------|---------|
| Ownership | v0.6: `^` = mutable, `&` = borrow, move por defecto |
| Borrow checker | Use-after-move, one-mut-XOR-many-immut |
| Arrays | `[val; N]` syntax, load elimination en Index |
| str_builder | Clase nativa `str_builder(50000).append("x")` |
| Listas | `list.reserve(n)` pre-asignación |
| Naming | snake_case global en docs y diseño |
| Docs | Reestructuración SSOT completa (165 archivos) |

## v0.5.2

| Área | Cambio |
|------|---------|
| HTTP | Server con router y middleware |
| JSON | `serialize`/`deserialize` para structs |
| SQLite | Package sqlite implemented |
| FFI | `extern fn` + `@link` completado |

## v0.5.1

| Área | Cambio |
|------|---------|
| LSP | Completado, hover, diagnostics |
| Formatter | `ky fmt` implementado |
| Packages | `ky add/remove/install/publish` |

## v0.5.0

| Área | Cambio |
|------|---------|
| Compiler | Fases 1-17 completadas |
| Syntax | Generics, ranges, match, operator overloading |
| Async | `async fn`, `await`, thread pool |
| LLVM | O3 pipeline, TBAA, nsw, LTO |

## v0.4.0

| Área | Cambio |
|------|---------|
| Ownership | Borrow por defecto, `^T` para move |
| Runtime | `libkyc_runtime.a` con 88 funciones |
| CLI | `ky build`, `ky run`, `ky check` |

## v0.3.0

| Área | Cambio |
|------|---------|
| Language | Indentación, tipos, funciones, clases |
| Compiler | LLVM codegen funcional |
| Runtime | Memoria, strings, listas, dicts |
