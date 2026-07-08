# Grammar

> Gramática de Kyle en formato EBNF simplificado.
> **Nota:** Esta es una gramática de alto nivel. La gramática formal completa
> está en el código fuente del parser (`kyc_frontend/src/parser.rs`, 2628 líneas).

## Programa

```
program         = { declaration | statement }
declaration     = function_decl | class_decl | enum_decl | contract_decl
                | variable_decl | constant_decl | type_alias | import
```

## Funciones

```
function_decl   = "fn" identifier params [ ":" type ] ":" block
params          = "(" [ param { "," param } ] ")"
param           = identifier ":" type [ "=" expression ]
block           = NEWLINE INDENT { statement } DEDENT
```

## Tipos

```
type            = primitive_type | user_type | generic_type
                | "?" type    -- optional (T?)
                | "!" type    -- fallible (T!)
                | "&" type    -- borrow
                | "^" type    -- mutable
                | "[" type ";" NUMBER "]"  -- array
                | "{" type "}"             -- list
                | "{" type ":" type "}"    -- dict
                | "(" type { "," type } ")"  -- tuple
                | "fn" "(" [ type { "," type } ] ")" type  -- fn ptr
```

## Statements

```
statement       = variable_decl | typed_decl | assignment
                | if_stmt | while_stmt | for_stmt | match_stmt
                | return_stmt | break_stmt | continue_stmt
                | defer_stmt | guard_stmt | unsafe_block
                | expression

variable_decl   = identifier "=" expression
typed_decl      = identifier ":" type "=" expression
constant_decl   = identifier ":=" expression
if_stmt         = "if" expression ":" block
                  { "elif" expression ":" block }
                  [ "else" ":" block ]
while_stmt      = "while" expression ":" block
for_stmt        = "for" identifier "in" expression ":" block
match_stmt      = "match" expression ":" NEWLINE INDENT
                  { pattern ":" block } DEDENT
return_stmt     = "return" [ expression ]
```

## Expresiones

```
expression      = literal | identifier | binary_op | unary_op
                | function_call | index | property_access
                | array_literal | list_literal | dict_literal
                | tuple_literal | struct_literal
                | "await" expression
                | "async" expression
                | "(" expression ")"

literal         = INTEGER | FLOAT | STRING | BOOLEAN | CHAR | "none"
binary_op       = expression operator expression
operator        = "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | ">"
                | "<=" | ">=" | "and" | "or" | "??"
function_call   = expression "(" [ expression { "," expression } ] ")"
index           = expression "[" expression "]"
array_literal   = "[" [ expression { "," expression } ] "]"
list_literal    = "{" [ expression { "," expression } ] "}"
```

## Ver también

- `06-compiler/parser.md` — Implementación del parser recursive descent
- `06-compiler/lexer.md` — Tokenización
- `03-language/syntax/` — Explicación detallada de cada construcción
