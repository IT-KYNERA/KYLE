# Parser

> Transforma un stream de tokens en un AST (Abstract Syntax Tree).
> Crate: `kyc_frontend/src/parser.rs` (2628 líneas).

## Responsabilidad

El parser toma los tokens del lexer y construye el AST, verificando la estructura
sintáctica del programa. Detecta errores como paréntesis no cerrados, keywords mal
ubicados, indentación incorrecta, etc.

## Invocación

```rust
fn parse(tokens: Vec<Token>) -> Result<Module, Vec<Diagnostic>>
```

Devuelve un `Module` (AST completo) o una lista de errores de sintaxis.

## AST (Abstract Syntax Tree)

El árbol de sintaxis abstracta representa el programa como una estructura de datos
que el compilador puede analizar. Definido en `kyc_core/src/ast.rs`.

### Module

```rust
pub struct Module {
    pub name: String,
    pub declarations: Vec<Decl>,
    pub statements: Vec<Stmt>,
    pub links: Vec<String>,      // @link directives
}
```

### Decl

```rust
pub enum Decl {
    Function(FunctionDecl),       // fn declarations
    Class(ClassDecl),             // class / final class
    Enum(EnumDecl),               // enum
    Contract(ContractDecl),       // contract (trait)
    Variable(VariableDecl),       // global variable
    Constant(ConstantDecl),       // NAME := expr
    TypeAlias(TypeAliasDecl),     // type alias
    Import(ImportDecl),           // from/import
}
```

### Stmt

```rust
pub enum Stmt {
    Variable(VariableDecl),        // x = value / x := value
    TypedVariable(VariableDecl),   // x: Type = value
    Expression(Expr),              // expression statement
    Return(Option<Box<Expr>>),
    Break(Option<Box<Expr>>),
    Continue,
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Match(MatchStmt),
    Defer(DeferStmt),
    Guard(GuardStmt),
    Unsafe(UnsafeBlock),
}
```

### Expr

```ky
pub enum Expr {
    Literal { value: Literal },
    Identifier { name: String },
    Binary { left, operator, right },
    Unary { operator, operand },
    Assignment { target, value },
    FunctionCall { target, arguments, type_args },
    PropertyAccess { object, property },
    Index { target, index },
    Array { elements },
    ArrayRepeat { value, count },
    List { elements },
    Dictionary { entries },
    Tuple { elements },
    StructLiteral { struct_name, fields },
    BorrowRef { expression, mutable },     // &x, ^&x
    NullCoalesce { left, right },          // left ?? right
    Ternary { cond, then, else },
    MatchExpr { expression, arms },
    Await { expression },                  // await expr
    Async { expression },                  // async expr
    AsyncBlock { body },                   // async: ...
}
```

## Parsing strategy (Recursive Descent)

Kyle usa un parser **recursive descent** con un token de lookahead.

```rust
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn parse_module(&mut self) -> Result<Module, String> { ... }
    fn parse_decl(&mut self) -> Result<Decl, String> { ... }
    fn parse_stmt(&mut self) -> Result<Stmt, String> { ... }
    fn parse_expr(&mut self) -> Result<Expr, String> { ... }
    fn parse_type(&mut self) -> Result<AstType, String> { ... }
    fn parse_block(&mut self) -> Result<Block, String> { ... }
}
```

### Ejemplo de parsing: `fn add(a: i32, b: i32) i32:`

```rust
fn parse_function(&mut self) -> Result<FunctionDecl, String> {
    let start = self.pos;
    self.expect(TokenKind::Fn)?;                    // consume 'fn'
    let name = self.eat_identifier();               // 'add'
    self.expect(TokenKind::LParen)?;                // '('
    let params = self.parse_params()?;              // 'a: i32, b: i32'
    self.expect(TokenKind::RParen)?;                // ')'
    let return_type = if self.at(TokenKind::Colon) { // ':'
        self.advance();
        Some(self.parse_type()?)                     // 'i32'
    } else { None };
    self.expect(TokenKind::Colon)?;                  // ':' después del return type
    let body = self.parse_block()?;                  // bloque indentado
    Ok(FunctionDecl { name, params, return_type, body, ... })
}
```

## Indentación sensible

El parser usa tokens `Indent`/`Dedent` del lexer para estructurar bloques:

```ky
fn main() i32:             # Colon → espera bloque indentado
    x = 42                 # Indent
    println(x.to_str())     # mismo nivel
                            # Dedent (fin de bloque)
```

## Manejo de errores

El parser reporta errores con ubicación precisa:

```rust
Err("expected ':' after function declaration at line 5, column 20")
```

Errores comunes:
- `Indent`/`Dedent` inconsistente
- `:` faltante después de declaraciones
- Paréntesis no cerrados
- Keywords en posición incorrecta
- Tipo inválido

## Ver también

- `lexer.md` — Genera los tokens que consume el parser
- `ast.md` — Definiciones detalladas del AST
- `hir.md` — Transformaciones post-parsing
