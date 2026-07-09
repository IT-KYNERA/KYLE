# Variables

> Declaration y uso de variablis en Kyle.

## Declaration

Las variablis se declaran with `name = value`. No there is `let`, `var` ni `const`.

```ky
x: i32 = 42 # type explicito + value
y = 10 # type inferido (i32)
name: str = "Ana" # string
sueldo: f64 = 3500.50 # float
activo: bool = true # bool
```

## Inmutabilidad by defecto

Por defecto, variablis are **inmutables**. No se can reallocate.

```ky
x: i32 = 10
x = 20 # ERROR: cannot modify immutable variable 'x'
```

### Mutabilidad with `^T`

```ky
x: ^i32 = 10 # mutable
x = x + 1 # ✅ permitido

name: ^str = "Ana"
name = "Pepe" # ✅ permitido
```

## Typis Copy vs Move

### Copy (numericos, bool, char, ptr)

```ky
x: i32 = 42
y: i32 = x # COPY: ambos vivos
println(x) # ✅ 42

a: f64 = 3.14
b: f64 = a # COPY
```

### Move (str, {T}, {K:V}, [T; N], clases)

```ky
s: str = "hola"
t: str = s # MOVE: s invalido after
println(s) # ❌ ERROR: use-after-move

# Copia explicita
t = s.clone() # ambos vivos
println(s) # ✅ "hola"
```

## Shorthands globales

`print()`, `println()`, `input()` are disponiblis globalmente:

```ky
println("hello")
name: str = input("name? ")
```

## Tipado estricto

Kyle is **fuertemente tipado**. No there is coercion implicita between typis incompatibles.

```ky
x: i32 = 42
y: f64 = x as f64 # ✅ cast explicito
# y = x # ❌ type mismatch
```

## Borrow con listas

```ky
# &T para borrow inmutable (solo lectura)
fn contar_libros(biblioteca: &{str}) i64:
    biblioteca.len()

# ^&T para borrow mutable (lectura + escritura)
fn agregar_libro(biblioteca: ^&{str}, libro: str):
    biblioteca.push(libro)

# Move por defecto (transferencia de ownership)
fn tomar_lista(biblioteca: {str}):
    # biblioteca es dueña de los datos ahora
    println(biblioteca.len())

# Uso:
libros: ^{str} = {}
agregar_libro(^&libros, "El Quijote")
n = contar_libros(&libros)  # solo prestamos
println(n.to_str())
tomar_lista(libros)  # movemos ownership
# libros ya no es accesible aqui (use-after-move)
```

## Scope

Las variablis pertenecen al bloque where se declaran:

```ky
x: i32 = 1
if true:
 y: i32 = 2
 x = x + y # ✅ acceso a variable exterior
# y no accesible aqui
```

## Destructuring

```ky
punto: (i32, str) = (10, "hello")
(x, y) = punto # x=10, y="hello"

list: {i32} = {1, 2, 3}
(primero, segundo) = list
```
