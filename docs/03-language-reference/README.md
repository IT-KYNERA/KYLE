# Language Reference

Formal specification of the Kyle programming language.

Status: 12/15 completos [x], 3 con bugs [ ] (static fn, patterns range, error-handling). Cada doc tiene su status al inicio.

| Document | Description |
|-----------|-------------|
| [lexical-structure.md](lexical-structure.md) | file format, indentation, comments, identifiers |
| [types.md](types.md) | Type system: primitive, compound, user-defined |
| [variables.md](variables.md) | Variable declaration, constants, mutability |
| [expressions.md](expressions.md) | All expression forms |
| [statements.md](statements.md) | All statement forms |
| [functions.md](functions.md) | Functions, methods, closures |
| [pattern-matching.md](pattern-matching.md) | Pattern matching |
| [classes.md](classes.md) | Classes, final class, inheritance |
| [enums.md](enums.md) | Enumerations with payload |
| [generics.md](generics.md) | Generics in classes and functions |
| [modules.md](modules.md) | Modules, imports, visibility |
| [ownership.md](ownership.md) | Ownership, borrowing, move semantics |
| [error-handling.md](error-handling.md) | Fallible types, `T!`, `?` operator |
| [concurrency.md](concurrency.md) | Async, await, threads |
| [ffi.md](ffi.md) | FFI: extern fn, @link, ptr |
