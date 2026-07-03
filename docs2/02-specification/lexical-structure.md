# Lexical Structure

## Source File Format

Kyle source files use **UTF-8** encoding. Each file has extension `.ky`.

## Indentation

Indentation defines block structure. **4 spaces** per level.
Tabs are not allowed.

```ky
fn main() i32:
    x = 1          # indented 4 spaces
    if x > 0:
        x + 1      # indented 8 spaces
```

## Comments

```ky
# This is a comment (line comment)
## This is a doc comment (appears before declarations)
```

## Identifiers

Identifiers start with a letter or underscore, followed by letters, digits, or underscores.

```ky
name        # public
_name       # protected
__name      # private
foo123      # digits allowed
```

## Reserved Keywords

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

See [Operators Reference](../05-reference/README.md#operators).

## Literals

### Integer
```ky
42        # decimal
-17       # negative
0xFF      # hexadecimal
0b1010    # binary
```

### Float
```ky
3.14      # float
-2.5      # negative
1e10      # scientific
```

### String
```ky
"hello"                     # basic string
"line1\nline2"             # escape sequences
"hello {name}"             # interpolation
```

### Boolean
```ky
true
false
```

### None
```ky
none
```
