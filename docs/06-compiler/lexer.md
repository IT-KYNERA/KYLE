# Lexer

> Transforma código fuente `.ky` en un stream de tokens.
> Crate: `kyc_frontend/src/lexer.rs` (914 líneas).

## Responsabilidad

El lexer (también llamado tokenizer) convierte el texto fuente en tokens estructurados
que el parser puede consumir. No realiza análisis sintáctico ni semántico.

## Funcionamiento

```rust
fn tokenize(source: &str) -> Vec<Token>
```

Toma el código fuente como string y devuelve un `Vec<Token>`.

## Tokens

Cada token tiene:

```rust
pub struct Token {
    pub kind: TokenKind,   // El tipo de token (identificador, keyword, símbolo, etc.)
    pub span: Span,         // Posición en el código fuente (línea, columna)
}
```

### TokenKind (principales)

| Categoría | Ejemplos |
|-----------|----------|
| Keywords | `fn`, `class`, `if`, `while`, `return`, `match`, `let`, `async`, `await` |
| Identifiers | `nombre`, `foo`, `mi_funcion` |
| Literals | Integer(`42`), Float(`3.14`), String(`"hola"`), Char(`'a'`), Boolean(`true`/`false`) |
| Operators | `+`, `-`, `*`, `/`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `\|\|` |
| Delimiters | `(`, `)`, `{`, `}`, `[`, `]`, `:`, `;`, `,` |
| Special | `:=` (walrus), `->` (arrow), `..` (range), `..=` (inclusive range) |
| Sigils | `^` (mutable/move), `&` (borrow), `@`, `#`, `?`, `!` |

### Indentación como sintaxis

Kyle usa **indentación** para delimitar bloques (como Python), no llaves:

```ky
fn main() i32:
    x = 42
    if x > 10:
        println("mayor")
    println("siempre")
```

El lexer genera tokens `Indent`/`Dedent` basados en cambios de indentación.
Esto simplifica el parser y hace el código más limpio.

## Reglas de indentación

- Primer indent define el tamaño base (se adapta a 2, 3, 4 espacios o tabs)
- `Indent` cuando la indentación aumenta
- `Dedent` cuando la indentación disminuye
- Líneas vacías y comentarios se ignoran para el cálculo de indentación
- `:` al final de una línea (ej. `fn f():`) indica que sigue un bloque indentado

## Ejemplo

```ky
# Código fuente
fn sum(a: i32, b: i32) i32:
    result = a + b
    result

# Tokens generados
[
    Fn, Ident("sum"), LParen, Ident("a"), Colon, Ident("i32"),
    Comma, Ident("b"), Colon, Ident("i32"), RParen, Ident("i32"),
    Colon, Newline,
    Indent,
        Ident("result"), Eq, Ident("a"), Plus, Ident("b"), Newline,
        Ident("result"), Newline,
    Dedent,
    Eof
]
```

## Manejo de errores

Si encuentra un carácter no válido, el lexer retorna un error con la ubicación exacta:

```rust
Err("unexpected character '#' at line 3, column 5")
```

## Ver también

- `parser.md` — Consume los tokens generados por el lexer
- `03-language/lexical/` — Especificación de tokens, keywords, literales
