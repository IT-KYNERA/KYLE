# Variables

> Declaración y uso de variables en Kyle.

## Declaración

Las variables se declaran con `nombre = valor`. No hay `let`, `var` ni `const`.

```ky
x: i32 = 42              # tipo explícito + valor
y = 10                   # tipo inferido (i32)
nombre: str = "Ana"      # string
sueldo: f64 = 3500.50    # float
activo: bool = true      # bool
```

## Inmutabilidad por defecto

Por defecto, las variables son **inmutables**. No se pueden reasignar.

```ky
x: i32 = 10
x = 20   # ERROR: cannot modify immutable variable 'x'
```

### Mutabilidad con `^T`

```ky
x: ^i32 = 10     # mutable
x = x + 1        # ✅ permitido

nombre: ^str = "Ana"
nombre = "Pepe"  # ✅ permitido
```

## Tipos Copy vs Move

### Copy (numéricos, bool, char, ptr)

```ky
x: i32 = 42
y: i32 = x       # COPY: ambos vivos
println(x)        # ✅ 42

a: f64 = 3.14
b: f64 = a       # COPY
```

### Move (str, {T}, {K:V}, [T; N], clases)

```ky
s: str = "hola"
t: str = s        # MOVE: s inválido después
println(s)        # ❌ ERROR: use-after-move

# Copia explícita
t = s.clone()     # ambos vivos
println(s)        # ✅ "hola"
```

## Shorthands globales

`print()`, `println()`, `input()` están disponibles globalmente:

```ky
println("hello")
name: str = input("¿nombre? ")
```

## Tipado estricto

Kyle es **fuertemente tipado**. No hay coerción implícita entre tipos incompatibles.

```ky
x: i32 = 42
y: f64 = x as f64   # ✅ cast explícito
# y = x             # ❌ type mismatch
```

## Scope

Las variables pertenecen al bloque donde se declaran:

```ky
x: i32 = 1
if true:
    y: i32 = 2
    x = x + y       # ✅ acceso a variable exterior
# y no accesible aquí
```

## Destructuring

```ky
punto: (i32, str) = (10, "hello")
(x, y) = punto       # x=10, y="hello"

lista: {i32} = {1, 2, 3}
(primero, segundo) = lista
```
