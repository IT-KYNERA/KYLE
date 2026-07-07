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
| `s.char_at(i)` | char | Carácter en índice |
| `s.is_digit()` | i32 | Primer char es dígito? |
| `s.is_alpha()` | i32 | Primer char es letra? |
| `s.is_alnum()` | i32 | Primer char es alfanumérico? |
| `s.is_whitespace()` | i32 | Primer char es espacio? |
| `s.is_upper()` | i32 | Primer char es mayúscula? |
| `s.is_lower()` | i32 | Primer char es minúscula? |

### Char methods

| Method | Returns | Description |
|--------|---------|-------------|
| `c.ord()` | i32 | Código ASCII |
| `c.is_digit()` | i32 | Es dígito? |
| `c.is_alpha()` | i32 | Es letra? |
| `c.is_alnum()` | i32 | Es alfanumérico? |
| `c.is_whitespace()` | i32 | Es espacio? |
| `c.is_upper()` | i32 | Es mayúscula? |
| `c.is_lower()` | i32 | Es minúscula? |

### Universal methods (disponibles en cualquier valor)

| Method | Returns | Description |
|--------|---------|-------------|
| `val.to_str()` | str | Convertir a string |
| `val.to_i32()` | i32 | Convertir a entero |
| `val.to_f64()` | f64 | Convertir a flotante |
| `val.to_bool()` | bool | Convertir a booleano |
| `val.type()` | TypeInfo | Información del tipo (`.name`, `.kind`, `.size`) |

### Chaining

```ky
result = "  Hello World  ".trim().to_upper().substr(0, 5)
# result = "HELLO"
```
