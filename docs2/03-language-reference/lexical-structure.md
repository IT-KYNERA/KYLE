# Lexical Structure

## Source file format

Kyle source files use **UTF-8** encoding. File extension: `.ky`.

## Indentation

Indentation defines block structure. **4 spaces** per level.

```ky
fn main() i32:
    x = 1
    if x > 0:
        x + 1
```

## Comments

```ky
# Line comment
## Doc comment (appears before declarations)
```

## Identifiers

Identifiers start with a letter or underscore, followed by letters, digits, or underscores.

```ky
name        # public
_name       # protected
__name      # private
foo123      # digits allowed
```

## Keywords

```
fn         class       final       abstract    enum
contract   struct      type        if          elif
else       while       for         in          match
return     break       continue    defer       guard
unsafe     async       await       const       static
true       false       none        and         or
not        is          as          this        super
```

## Operators

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

## Literals

```ky
42          # integer decimal
0xFF        # hexadecimal
0b1010      # binary
3.14        # float
"hello"     # string
true        # boolean
false
none        # null value
```

## Integer literal types

The default type for integer literals is `i32`. Values exceeding `i32` range are inferred as `i64`.

```ky
x = 42        # i32
y: i64 = 42   # explicit i64
```
