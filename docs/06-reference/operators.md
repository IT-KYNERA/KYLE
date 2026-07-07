# Operators

| Operator | Description | Category |
|----------|-------------|----------|
| `+` | Add | Arithmetic |
| `-` | Subtract | Arithmetic |
| `*` | Multiply | Arithmetic |
| `/` | Divide | Arithmetic |
| `%` | Remainder | Arithmetic |
| `**` | Power | Arithmetic |
| `==` | Equal | Comparison |
| `!=` | Not equal | Comparison |
| `<` | Less than | Comparison |
| `>` | Greater than | Comparison |
| `<=` | Less or equal | Comparison |
| `>=` | Greater or equal | Comparison |
| `and` | Logical AND | Logical |
| `or` | Logical OR | Logical |
| `not` | Logical NOT | Logical |
| `&` | Bitwise AND | Bitwise |
| `\|` | Bitwise OR | Bitwise |
| `^` | Bitwise XOR | Bitwise |
| `<<` | Shift left | Bitwise |
| `>>` | Shift right | Bitwise |
| `..` | Range exclusive | Range |
| `..=` | Range inclusive | Range |
| `..<` | Range exclusive (alias) | Range |
| `is` | Type test | Type |
| `as` | Type cast | Type |
| `=` | Assign | Assignment |
| `+=` | Add assign | Assignment |
| `-=` | Subtract assign | Assignment |

---

## Built-in methods

### String methods

| Method | Returns | Description |
|--------|---------|-------------|
| `s.len()` | i32 | Longitud |
| `s.upper()` | str | Mayúsculas |
| `s.lower()` | str | Minúsculas |
| `s.trim()` | str | Sin espacios |
| `s.contains(sub)` | i32 | Contiene subcadena |
| `s.replace(from, to)` | str | Reemplazar |
| `s.substr(start, len)` | str | Subcadena |
| `s.charAt(i)` | char | Carácter en índice |
| `s.isDigit()` | i32 | Primer char es dígito? |
| `s.isAlpha()` | i32 | Primer char es letra? |
| `s.isAlnum()` | i32 | Primer char es alfanumérico? |
| `s.isWhitespace()` | i32 | Primer char es espacio? |
| `s.isUpper()` | i32 | Primer char es mayúscula? |
| `s.isLower()` | i32 | Primer char es minúscula? |

### Char methods

| Method | Returns | Description |
|--------|---------|-------------|
| `c.ord()` | i32 | Código ASCII |
| `c.isDigit()` | i32 | Es dígito? |
| `c.isAlpha()` | i32 | Es letra? |
| `c.isAlnum()` | i32 | Es alfanumérico? |
| `c.isWhitespace()` | i32 | Es espacio? |
| `c.isUpper()` | i32 | Es mayúscula? |
| `c.isLower()` | i32 | Es minúscula? |

### Universal methods (disponibles en cualquier valor)

| Method | Returns | Description |
|--------|---------|-------------|
| `val.toStr()` | str | Convertir a string |
| `val.toI32()` | i32 | Convertir a entero |
| `val.toF64()` | f64 | Convertir a flotante |
| `val.toBool()` | bool | Convertir a booleano |
| `val.type()` | TypeInfo | Información del tipo (`.name`, `.kind`, `.size`) |

### Chaining

```ky
result = "  Hello World  ".trim().toUpper().substr(0, 5)
# result = "HELLO"
```
