# Tokens

> Los tokens son las unidades léxicas que el lexer produce y el parser consume.

## Categorías

### Keywords

```ky
fn, class, final, abstract, enum, contract, type
if, elif, else, while, for, in, match, return
break, continue, defer, guard, unsafe
async, await, static, true, false, none, error, ok
import, from, as, is, and, or, not
let, var, const, pub, use, type
```

### Literales

```ky
42              # Integer (i32)
3.14            # Float (f64)
"hello"         # String
true / false    # Boolean
'a'             # Char
none            # Option::None
```

### Operadores

```ky
+   -   *   /   %       # Aritméticos
==  !=  <   >   <=  >=  # Comparación
&&  ||  !               # Lógicos
&   |   ^   <<  >>      # Bitwise
=   :=                  # Asignación y declaración
+=  -=  *=  /=  %=      # Compuestos
++  --                  # Incremento/decremento
..  ..=                 # Range
??                      # Null-coalescing
->                      # Arrow
as  is                  # Cast y type check
```

### Delimitadores

```ky
( ) { } [ ] : ; , .  # Paréntesis, llaves, corchetes
```

### Indentación

```ky
:           # Indent (inicio de bloque)
# Indent es un token especial generado por el lexer
# basado en cambios de indentación
```

## Ver también

- `06-compiler/lexer.md` — Implementación del lexer
- `06-compiler/parser.md` — Cómo el parser consume tokens
