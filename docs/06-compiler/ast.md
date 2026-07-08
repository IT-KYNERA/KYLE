# AST (Abstract Syntax Tree)

> Representación estructurada del programa después del parsing.
> Crate: `kyc_core/src/ast.rs` (~800 líneas).

## Responsabilidad

El AST es la representación intermedia entre el parser y el resto del compilador.
Es un árbol que refleja directamente la sintaxis del programa. A diferencia del HIR,
el AST retiene la estructura sintáctica exacta (incluyendo sugar sintáctico).

## Estructura principal

```
Module
├── Declarations
│   ├── Function     → fn declarations
│   ├── Class        → class / final class
│   ├── Enum         → enum variants
│   ├── Contract     → contract (traits)
│   ├── Variable     → global variables
│   ├── Constant     → NAME := expr
│   ├── TypeAlias    → type alias
│   └── Import       → from/import
└── Statements (module-level)
    ├── Variable
    ├── Expression
    ├── If / While / For / Match
    └── Return / Break / Continue
```

## Types (AstType)

```rust
pub enum AstType {
    Primitive { name: String },       // i32, str, bool, etc.
    User { name: String },             // user-defined type
    Mutable { inner: Box<AstType> },   // ^T
    Borrow { inner: Box<AstType> },    // &T
    Generic { name: String, args: Vec<AstType> },  // Option<i32>
    Array { inner: Box<AstType>, size: usize },     // [T; N]
    List { inner: Box<AstType> },       // {T}
    Dict { key, value },                // {K: V}
    Optional { inner },                  // T?
    Error { inner },                     // T!
    FnPtr { params, return_ },           // fn(T) U
    Ptr,                                 // ptr
    Struct { fields },                   // struct literal type
}
```

## Span

Cada nodo del AST tiene un `Span` que indica su posición exacta en el código fuente:

```rust
pub struct Span {
    pub start: usize,   // byte offset
    pub end: usize,     // byte offset
    pub line: u32,
    pub column: u32,
}
```

Los spans son fundamentales para reportar errores con ubicación precisa.

## Ejemplo

```ky
# Código fuente
fn add(a: i32, b: i32) i32:
    a + b
```

Produce un AST como:

```
Module
└── Function "add"
    ├── Params
    │   ├── "a": Primitive("i32")
    │   └── "b": Primitive("i32")
    ├── Return: Primitive("i32")
    └── Body
        └── BinaryOp(Add)
            ├── Identifier("a")
            └── Identifier("b")
```

## Ver también

- `parser.md` — Construye el AST
- `hir.md` — Desugaring del AST a HIR
