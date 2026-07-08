# AST (Abstract Syntax Tree)

> Representación estructurada del programa después del parsing.
> Crate: `kyc_core/src/ast.rs` (~1372 líneas).

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
    Primitive { name: String, span: Span },
    User { name: String, span: Span },
    Generic { name: String, args: Vec<AstType>, span: Span },
    Optional { inner: Box<AstType>, span: Span },    // T?
    Error { inner: Box<AstType>, span: Span },        // T!
    Dict { key: Box<AstType>, value: Box<AstType>, span: Span },  // {K: V}
    FnPtr { params: Vec<AstType>, return_: Box<AstType>, span: Span },
    Mutable { inner: Box<AstType>, span: Span },      // ^T
    Borrow { inner: Box<AstType>, span: Span },       // &T
    Array { inner: Box<AstType>, size: usize, span: Span },  // [T; N]
    Ptr { span: Span },
}
```

> **Nota:** `{T}` (list) y `{K:V}` (dict) NO son `AstType` variants propios.
> Se representan como `Generic { name: "list" }` / `Generic { name: "dict" }` en el AST.
> Los structs literales tampoco tienen variant propia — se resuelven como tipos usuario.

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
