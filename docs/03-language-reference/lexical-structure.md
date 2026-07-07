# Lexical Structure

## Source file format [x]

Kyle source files use **UTF-8** encoding. file extension: `.ky`.

## Indentation [x]

Indentation defines block structure. **4 spaces** per level.

```ky
fn main() i32:
    x = 1
    if x > 0:
        x + 1
```

## Comments [x]

```ky
# Line comment
## Doc comment (appears before declarations)
```
Note: no `/* */` block comments.

## Identifiers [x]

Identifiers start with a letter or underscore, followed by letters, digits, or underscores.

```ky
name        # public
_name       # protected
__name      # private
foo123      # digits allowed
```

## Keywords [x]

```
fn         class       final       abstract    enum
contract   struct      type        if          elif
else       while       for         in          match
return     break       continue    defer       guard
unsafe     async       await       const       static
true       false       None        and         or
not        is          as          this        super
```

**Bug:** docs dicen `none` pero el keyword real es `None` (mayuscula). Corregido arriba.

## Operators [x]

| Symbol | Description |
|---------|-------------|
| `+` `-` `*` `/` `%` | Arithmetic |
| `**` | Power |
| `==` `!=` `<` `>` `<=` `>=` | Comparison |
| `and` `or` `not` | Logical |
| `&` `\|` `^` `<<` `>>` | Bitwise |
| `..` `..=` `..<` | Range |
| `=` `+=` `-=` `*=` `/=` `%=` | Assignment |
| `is` `as` | Type test and cast |

## Literals [x]

```ky
42          # integer decimal
0xFF        # hexadecimal
0b1010      # binary
3.14        # float
"hello"     # string
true        # boolean
false
None        # null value (None con mayuscula)
```

## Integer literal types [x]

The default type for integer literals is `i32`. Values exceeding `i32` range are inferred as `i64`.

```ky
x = 42        # i32
y: i64 = 42   # explicit i64
```
