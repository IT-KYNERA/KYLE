# Semantic Analysis

> Type checking, scope resolution, validación semántica.
> Crate: `kyc_semantic/src/` (~2200 líneas).

## Responsabilidad

El análisis semántico verifica que el programa sea **significativo** más allá de la sintaxis:
- Tipos correctos en cada expresión
- Variables y funciones existen en el scope actual
- Mutabilidad respetada (no reasignar variables inmutables)
- Ownership: move vs borrow consistente
- Visibility: `public` vs `private` vs `protected`

## Componentes

| Archivo | Propósito |
|---------|-----------|
| `type_checker.rs` (1408) | Inferencia y verificación de tipos |
| `scope.rs` (422) | Resolución de nombres y scopes |
| `symbol_table.rs` (163) | Registro de símbolos y builtins |

## Pipeline

```
HIR (desugared AST)
    │
    ▼
[1] Scope Resolution → Asigna cada identificador a su declaración
    │
    ▼
[2] Type Inference → Determina el tipo de cada expresión
    │
    ▼
[3] Type Checking → Verifica que los tipos coincidan
    │
    ▼
Typed AST (every node has a known type)
```

## Type Checker

El type checker infiere y verifica tipos mediante un algoritmo basado en Hindley-Milner
adaptado para un lenguaje imperativo con ownership.

### Inferencia básica

```rust
fn infer_expr(&mut self, expr: &Expr) -> Type {
    match expr {
        Expr::Literal { value: Literal::Integer(n) } => Type::I32,
        Expr::Literal { value: Literal::Float(n) } => Type::F64,
        Expr::Literal { value: Literal::String(s) } => Type::Str,
        Expr::Literal { value: Literal::Boolean(b) } => Type::Bool,
        Expr::Identifier { name } => self.lookup_type(name),
        Expr::Binary { left, operator: BinaryOp::Add, right } => {
            let l = self.infer_expr(left);
            let r = self.infer_expr(right);
            // Ambos deben ser numéricos y del mismo tipo
            self.unify(l, r, "addition")
        }
        // ...
    }
}
```

### Type checking de llamadas a función

```rust
fn check_function_call(&mut self, target: &Expr, args: &[Expr]) -> Type {
    let func_type = self.infer_expr(target);
    let param_types = self.get_param_types(func_type);
    for (arg, param) in args.iter().zip(param_types.iter()) {
        let arg_type = self.infer_expr(arg);
        self.check_type_match(&arg_type, param, "argument");
    }
    self.get_return_type(func_type)
}
```

## Scope Resolver

El scope resolver asigna cada nombre a su declaración:

```ky
# Scope global
x = 42              # x → global scope

fn main() i32:       # main → global scope
    y = 10           # y → function scope
    if true:
        z = 5        # z → block scope (inside if)
    # z no accesible aquí
```

### Reglas

- Los scopes se anidan con cada bloque indentado
- El scope interior puede acceder al exterior
- Variables de bloques superiores no se pueden reasignar desde el interior
- Funciones y clases se registran en el scope antes de procesar el cuerpo

## Symbol Table

```rust
 struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
     type_defs: HashMap<String, Type>,
}

 struct Symbol {
     name: String,
     kind: SymKind,
}

 enum SymKind {
    Variable { is_mutable: bool, is_auto: bool },
    Function(FunctionDecl),
    Constant(ConstantDecl),
    Type(Type),
    Module(Vec<String>),
}
```

### Builtins registrados

```rust
fn register_builtins(&mut self) {
    // Tipos builtin
    self.type_defs.insert("i32", Type::I32);
    self.type_defs.insert("str", Type::Str);
    // ... (todos los tipos primitivos)
    
    // Funciones builtin
    self.register("println", SymKind::Function(...));
    self.register("len", SymKind::Function(...));
    self.register("assert_eq", SymKind::Function(...));
    // ...
}
```

## Errores semánticos comunes

```rust
// Type mismatch
KL-E0001: Type mismatch: expected 'i32', found 'str'
  --> file.ky:10:5

// Undefined symbol
KL-E0009: Undefined symbol 'foo'
  --> file.ky:5:10

// Cannot modify immutable
KL-E0007: Cannot modify immutable variable 'x'
  --> file.ky:3:5
```

## Ver también

- `type_checker.rs` en código fuente
- `03-language/types/` — Especificación del sistema de tipos
- `03-language/memory/` — Reglas de ownership y mutabilidad
