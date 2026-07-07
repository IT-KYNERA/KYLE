# Ownership

**Status:** [x] `^` = mutable, `&` = borrow, `^&` = mutable borrow, move por defecto. Borrow checker implementado.

## Reglas

1. **Move por defecto**: `y = x` transfiere ownership de `x` a `y` (para tipos no-Copy).
2. **Borrow con `&`**: `f(&x)` presta `x` sin transferir ownership.
3. **Mutable con `^`**: `x: ^str` declara variable mutable.
4. **Mutable borrow con `^&`**: `f(^&x)` presta `x` con permiso de modificar.
5. **Copy automático**: Tipos numéricos, `bool`, `char`, `ptr` se copian en `y = x`.
6. **Clone explícito**: `y = x.clone()` para copiar tipos Move.
7. **One mutable XOR many immutable** en un mismo scope.
8. **No dangling pointers**: las referencias no pueden outlive al valor original.
9. **No `&` return**: prohibido devolver referencias (evita problemas de lifetime).

## Copy vs Move

| Copia (automático) | Mueve (por defecto) |
|--------------------|---------------------|
| `i8`, `i16`, `i32`, `i64` | `str` |
| `u8`, `u16`, `u32`, `u64` | `{T}` (list) |
| `f32`, `f64` | `{K:V}` (dict) |
| `bool`, `char` | `[T; N]` (array) |
| `ptr` | classes, structs, enums |

## Variables

```ky
x = 42              # inmutable (default), COPY (i32)
s = "hola"          # inmutable (default), OWNED (str)
x: ^i32 = 0         # mutable, COPY
buf: ^str = ""      # mutable, OWNED
```

## Parámetros

```ky
fn f(s: str)        # MOVE: el caller pierde ownership
fn f(s: &str)       # BORROW: el caller presta
fn f(s: ^&str)      # MUT BORROW: el caller presta mutable
```

## Expresiones

```ky
# Move (default)
a = "hola"
b = a               # MOVE: a inválido después
println(a)          # ERROR: a fue movido

# Borrow
a = "hola"
println(&a)         # BORROW: a sigue vivo

# Mutable borrow
buf: ^str = ""
fill(^&buf)         # MUT BORROW: buf mutable prestado
println(buf)        # ✅ buf sigue vivo, modificado

# Clone (copia explícita)
a = "hola"
b = a.clone()       # COPY: ambos vivos
println(a)          # ✅ "hola"
println(b)          # ✅ "hola"

# Copy types (automático)
x = 42
y = x               # COPY: ambos vivos
println(x)          # ✅ 42
```

## Clases (sin `this` obligatorio)

```ky
class Point:
    x: ^i32 = 0
    y: ^i32 = 0

    fn moveTo(nx: &i32, ny: &i32):
        x = nx      # campo directo, sin this.x
        y = ny

    fn clone():
        Point(x, y)

    fn register():
        poll_events(&this)   # autoreferencia con this
```

## Borrow checker (implementado)

El borrow checker valida:

1. ✅ **Use-after-move**: `y = x; println(x)` → error.
2. ✅ **Aliasing mutability**: `r1 = ^&x; r2 = &x` → error (uno mutable + otro inmutable).
3. 🔶 **Dangling references**: `x = &y; drop(y); println(x)` — pendiente.

## Comparación con Rust

| Concepto | Rust | Kyle |
|----------|------|------|
| Variable inmutable | `let x` | `x = v` |
| Variable mutable | `let mut x` | `x: ^T = v` |
| Move | `y = x` (default) | `y = x` (default) |
| Borrow | `&x` | `&x` |
| Mutable borrow | `&mut x` | `^&x` |
| Clone | `.clone()` | `.clone()` |
| Copy types | `#[derive(Copy)]` | Built-in para numéricos |
| Lifetime params | `'a` | **No existen** (prohibido return `&`) |
| `self` | `self.foo` | `foo` (directo) |
