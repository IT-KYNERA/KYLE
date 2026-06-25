# Kyle — Referencia Completa de Sintaxis v1.0

> **Idioma:** Español  
> **Propósito:** Documentar absolutamente toda la sintaxis del lenguaje Kyle, con marcas de estado de implementación.  
> **Versión del lenguaje:** v2.0  
> **Última actualización:** 2026-06-22

---

## Leyenda de Estado

| Marca | Significado |
|-------|-------------|
| ✅ | **Completamente implementado** — parsea, type-checkea, genera código, funciona en runtime |
| ⚠️ | **Implementado parcialmente** — parsea y/o type-checkea, pero algo falla en codegen o runtime |
| 🔶 | **Parseado pero sin codegen** — el parser lo entiende, el type checker lo valida, pero no genera código máquina |
| ❌ | **No implementado** — no parsea, no type-checkea, no genera código |
| 📄 | **Solo en especificación** — está en los docs pero el compilador aún no lo soporta |

---

## 1. Variables y Mutabilidad ✅

Kyle no usa `let`, `var` ni `const`. Las variables se declaran con el nombre directamente.

### Inmutables por defecto

```kl
nombre = "Ana"        # ✅ inmutable — no se puede reasignar
edad = 25             # ✅
```

### Mutables con `mut`

```kl
mut total = 0          # ✅ se puede reasignar
total = total + 1      # ✅
```

### Tipos explícitos

```kl
nombre: str = "Ana"    # ✅
edad: i32 = 25         # ✅
altura: f64 = 1.75     # ✅
```

### Variables globales

```kl
VERSION = "1.0"        # ✅ constante (ver sección 3)
mut contador = 0       # ✅ mutable global (con mut)
```

### Reglas

- Sin `let`, `var`, `const` como keywords
- UPPERCASE = constante inmutable
- lowercase / camelCase = variable inmutable por defecto
- `mut` delante = variable mutable
- Tipado estático con inferencia

---

## 2. Tipos Primitivos ✅

```kl
i8, i16, i32, i64       # enteros con signo ✅
u8, u16, u32, u64       # enteros sin signo ✅
f32, f64                # punto flotante ✅
bool                    # booleano (true / false) ✅
char                    # carácter Unicode (1 byte) ✅
str                     # cadena UTF-8 (inmutable) ✅
```

### Literales

```kl
edad = 42               # i32 ✅
hex = 0xFF              # i32 hexadecimal ✅
bin = 0b1010            # i32 binario ✅
grande = 1000000i64     # i64 explícito ✅
flotante = 3.14         # f64 ✅
flotante32 = 3.14f32    # f32 explícito ✅
letra = 'a'             # char ✅
texto = "Hola"          # str ✅
activo = true           # bool ✅
falso = false           # bool ✅
nada = None             # Option<T> vacío ✅
```

---

## 3. Constantes ✅

Toda variable en UPPERCASE es una constante de compilación.

```kl
PI = 3.141592           # ✅
MAX_USERS = 1000        # ✅
API_URL = "https://api.com"  # ✅
```

### Reglas

- Se evalúan en tiempo de compilación
- No se pueden reasignar
- Solo aceptan literales y expresiones constantes
- Sin keyword `const` — la convención UPPERCASE es suficiente

---

## 4. Operadores

### Aritméticos ✅

```kl
+   suma               # ✅
-   resta              # ✅
*   multiplicación     # ✅
/   división           # ✅
%   módulo             # ✅
**  potencia           # ✅
```

### Bitwise ✅

```kl
&   AND bitwise        # ✅
|   OR bitwise         # ✅
^   XOR bitwise        # ✅
~   NOT bitwise        # ✅
<<  shift izquierda    # ✅
>>  shift derecha      # ✅
```

### Comparación ✅

```kl
==  igual              # ✅
!=  distinto           # ✅
>   mayor que          # ✅
<   menor que          # ✅
>=  mayor o igual      # ✅
<=  menor o igual      # ✅
```

### Lógicos ✅

```kl
&&  AND lógico         # ✅
||  OR lógico          # ✅
!   NOT lógico         # ✅
```

### Asignación ✅

```kl
=   asignación         # ✅
+=  suma y asigna      # ✅
-=  resta y asigna     # ✅
*=  multiplica y asigna # ✅
/=  divide y asigna    # ✅
%=  módulo y asigna    # ✅
&=  AND bitwise y asigna  # ✅
|=  OR bitwise y asigna   # ✅
^=  XOR bitwise y asigna  # ✅
<<= shift izq y asigna    # ✅
>>= shift der y asigna    # ✅
```

### Type Check (is) ✅

```kl
if value is Admin:      # ✅ verifica tipo en runtime
    print("Admin")
```

```kl
if value is not Admin:  # ✅ negado
    print("No es Admin")
```

### Rango ✅

```kl
rango = 0..10           # ✅ genera código (for i in 0..10)
```

---

## 5. Funciones ✅

### Básica

```kl
fn saludar(nombre: str) -> str:      # ✅
    return "Hola " + nombre
```

### Sin parámetros

```kl
fn version() -> str:                 # ✅
    return "1.0"
```

### Sin retorno (void)

```kl
fn log(mensaje: str):                # ✅
    print(mensaje)
```

### Parámetros por defecto ✅

```kl
fn saludar(nombre: str, saludo: str = "Hola"):  # ✅
    print(saludo + " " + nombre)
```

### Argumentos nombrados ✅

```kl
fn configurar(host: str, puerto: i32, ssl: bool):  # ✅
    ...

configurar(                                      # ✅
    host: "localhost",
    ssl: true,
    puerto: 443
)
```

Los argumentos posicionales deben ir primero, los nombrados después.

### Funciones variádicas ✅

```kl
fn log(nivel: str, ...mensajes: str):   # ✅
    for msg in mensajes:
        print(nivel + ": " + msg)

log("INFO", "inicio", "procesando", "fin")  # ✅
```

El parámetro variádico es siempre el último y es de tipo `[T]` (lista).

### Retorno múltiple (tupla implícita)

```kl
fn dividir(a: i32, b: i32) -> (i32, i32):     # ✅
    return (a / b, a % b)

cociente, resto = dividir(10, 3)              # ✅ destructuring
```

### Tipos de retorno complejos

```kl
fn encontrar(id: i32) -> Option<User>:   # ✅
    ...
```

### Función como valor

```kl
mi_fn = mi_funcion                # ✅ (ver closures)
callback = (x) => x * 2           # ✅
```

---

## 6. Closures ✅

```kl
doblar = (x) => x * 2            # ✅
sumar = (a, b) => a + b          # ✅
pares = numeros.filter((x) => x % 2 == 0)  # ✅
```

### Con tipo explícito

```kl
doblar = (x: i32) => x * 2       # ✅
```

### Multi-línea (con bloque)

```kl
procesar = (x: i32) =>:          # ✅
    resultado = x * 2
    return resultado
```

---

## 7. Condicionales

### If / Elif / Else ✅

```kl
if edad >= 18:                   # ✅
    print("Adulto")
elif edad >= 13:                 # ✅
    print("Adolescente")
else:                            # ✅
    print("Niño")
```

### Binding If ✅

Crea una variable y chequea su verdad en un solo paso:

```kl
if usuario = obtener_usuario():  # ✅
    print(usuario.nombre)

# Equivale a:
if obtener_usuario():
    usuario = obtener_usuario()
    print(usuario.nombre)
```

La variable `usuario` solo existe dentro del bloque `if`.

### Ternary / operador ternario ✅

```kl
categoria = edad >= 18 ? "Adulto" : "Menor"
```

---

## 8. Bucles

### While ✅

```kl
while edad < 18:                 # ✅
    edad += 1
```

### While infinito ✅

```kl
loop:                            # ✅
    trabajo()
    if terminado:
        break
```

### While-Bind ✅

```kl
while valor = iterador.next():   # ✅
    procesar(valor)
```

Sale del bucle cuando `valor` es `None` o falsy.

### While-Else ✅

```kl
while condicion:                 # ✅
    trabajo()
else:                            # ✅
    print("Nunca se cumplió la condición")
```

El bloque `else` se ejecuta si la condición nunca fue verdadera.

### For ✅ (listas)

```kl
for item in items:               # ✅ genera código, itera sobre listas
    print(item)
```

### For con rango 🔶

```kl
for i in 0..10:                  # 🔶 parsea, sin codegen
    print(i)
```

### For-Else 🔶 (implementación pendiente)

```kl
for item in items:               # ✅ listas, ❌ else branch pendiente
    print("No hay elementos")
```

### Reglas de bucles

- No existe `continue` — la ejecución sigue naturalmente a la siguiente iteración
- `break` termina el bucle más interno
- `break valor` solo funciona en `loop:` expressions

---

## 9. Match ✅

```kl
match status:                    # ✅
    activo:
        print("Activo")
    inactivo:
        print("Inactivo")
    suspendido:
        print("Suspendido")
```

### Match con enum variants ✅

```kl
match resultado:                 # ✅
    Option.Some(valor):
        print(valor)
    Option.None:
        print("Sin valor")
```

### Match Guards ✅

```kl
match x:                         # ✅
    n if n > 0:
        print("positivo")
    n if n < 0:
        print("negativo")
    0:
        print("cero")
```

### Match con tipos

```kl
match valor:                     # ✅
    is Admin:
        print("Admin")
    is User:
        print("User")
```

### Match como expresión ✅

```kl
descripcion = match x:           # ✅
    0:
        "cero"
    n if n > 0:
        "positivo"
    else:
        "negativo"
```

---

## 10. Break / Defer / Guard

### Break ✅

```kl
while ejecutando:                # ✅
    if terminado:
        break                    # ✅ termina el bucle

resultado = loop:                # ✅ break con valor
    if terminado:
        break 42                 # ✅
```

`break` termina el bucle más interno. `break valor` solo en `loop:` expressions.

### Defer ✅

```kl
fn procesar_archivo():           # ✅
    archivo = open("datos.txt")
    defer archivo.close()        # ✅ LIFO, se ejecuta al retornar
    ...
```

Los defers se ejecutan en orden inverso (LIFO) al salir del scope.

### Guard ✅

```kl
fn procesar(usuario: Option<User>):  # ✅
    guard usuario != None else:      # ✅ CondBr lowering
        return                       # ✅
    print(usuario.nombre)            # ✅
```

Equivalente a:

```kl
if usuario == None:
    return
```

---

## 11. Strings y Chars ✅

### Strings

```kl
texto = "Hola Mundo"           # ✅ doble comilla
```

### Escape sequences ✅

| Secuencia | Significado |
|-----------|-------------|
| `\n` | Newline |
| `\t` | Tab |
| `\r` | Carriage return |
| `\\` | Backslash |
| `\"` | Double quote |
| `\{` | Abrir llave escapada |
| `\0` | Carácter nulo |

```kl
texto = "Línea1\nLínea2\tindentado"   # ✅
ruta = "C:\\Usuarios\\nombre"         # ✅
```

### String Interpolation ✅

```kl
nombre = "Ana"
edad = 30
print("Hola {nombre}, tienes {edad} años")  # ✅ genera código
print("Suma: {a + b}")                      # ✅ expresión arbitraria
```
```

Solo en strings de doble comilla. No en chars (comilla simple).

### Chars

```kl
c = 'x'              # ✅
newline = '\n'       # ✅
tab = '\t'           # ✅
backslash = '\\'     # ✅
```

---

## 12. Listas (Arrays) ✅

```kl
nombres = [                       # ✅
    "Juan",
    "Ana",
    "Mike"
]

edades: [i32] = [10, 20, 30]     # ✅ con tipo explícito
```

### Indexación ✅

```kl
primero = nombres[0]              # ✅
```

### Lista de strings

```kl
tokens: [str] = ["x", "=", "1"]  # ✅
primer_token = tokens[0]          # ✅ retorna str
```

### Métodos de lista ✅

```kl
lista.add(elemento)               # ✅ agrega al final
lista.pop()                       # ✅ saca y retorna el último
len(lista)                        # ✅ retorna i32
```

---

## 13. Tuplas y Destructuring

### Tuplas ✅

```kl
posicion = (10, 20)               # ✅
coordenadas: (f64, f64) = (1.5, 3.2)  # ✅
```

### Destructuring ✅

```kl
x, y = posicion                   # ✅
nombre, edad = ("Ana", 30)        # ✅
cociente, resto = dividir(10, 3)  # ✅
```

---

## 14. Structs ✅

```kl
struct Punto:                     # ✅
    x: f64
    y: f64

origen = Punto(x: 0.0, y: 0.0)   # ✅ named fields
punto = Punto(x: 10, y: 20)      # ✅
```

### Struct con genéricos ✅

```kl
struct Par<T, U>:                 # ✅ parsea, type-checkea, genera código
    primero: T
    segundo: U
```

---

## 15. Enums ✅

```kl
enum Status:                     # ✅
    activo
    inactivo
    suspendido

estado = Status.activo           # ✅
```

### Enum con payload ✅

```kl
enum Option<T>:                  # ✅
    Some(valor: T)
    None

resultado = Option.Some(42)      # ✅
nada = Option.None               # ✅
```

### Match sobre enum ✅

```kl
match resultado:                 # ✅
    Option.Some(v):
        print(v)
    Option.None:
        print("nada")
```

---

## 16. Clases ✅

```kl
class Usuario:                   # ✅
    nombre: str
    edad: i32
```

### Constructor ✅

```kl
class Usuario:                   # ✅
    nombre: str
    edad: i32

    Usuario(nombre: str, edad: i32):  # ✅
        this.nombre = nombre
        this.edad = edad
```

### Métodos ✅

```kl
class Usuario:                   # ✅
    nombre: str
    edad: i32

    fn saludar() -> str:         # ✅
        return "Hola, soy " + this.nombre
```

El primer parámetro `this` se pasa implícitamente — no se declara.

### Propiedades (get/set) ✅

```kl
class Usuario:                   # ✅
    _nombre: str

    nombre: str:                 # ✅ property
        get:
            return _nombre
        set(valor):
            _nombre = valor
```

---

## 17. Herencia y Polimorfismo ✅

```kl
class Animal:                    # ✅
    fn hablar()                  # ✅ método sin cuerpo (abstracto virtual)

class Perro : Animal:            # ✅ herencia
    fn hablar():                 # ✅ override
        print("Guau")

animal: Animal = Perro()         # ✅ polimorfismo
animal.hablar()                  # ✅ → "Guau"
```

---

## 18. Clases Abstractas ✅

```kl
abs class Animal:                # ✅ clase abstracta
    abs fn hablar()              # ✅ método abstracto
```

No se puede instanciar una clase abstracta directamente.

```kl
animal = Animal()                # ❌ error de compilación
```

---

## 19. Contracts (Interfaces) ✅

```kl
contract Serializable:           # ✅
    fn serializar() -> str

class Usuario : Serializable:    # ✅ implementación de contract
    fn serializar() -> str:      # ✅
        return nombre
```

### Múltiples contracts

```kl
class Usuario : Serializable, Comparable:  # ✅
    ...
```

---

## 20. Visibilidad (por convención de nombres) ✅

| Convención | Visibilidad |
|------------|-------------|
| `nombre` | ✅ Público |
| `_nombre` | ✅ Protegido (subclases) |
| `__nombre` | ✅ Privado (solo la clase) |

```kl
class Usuario:                   # ✅
    nombre: str                  # ✅ público
    _rol: str                    # ✅ protegido
    __password: str              # ✅ privado

    fn _validar():               # ✅ método protegido
        ...

    fn __encriptar():            # ✅ método privado
        ...
```

---

## 21. Genéricos ✅

### Funciones genéricas ✅

```kl
fn primero<T>(items: [T]) -> T:     # ✅ parsea, type-checkea, genera código
    return items[0]
```

### Clases genéricas 🔶

```kl
class Repositorio<T>:               # 🔶 parsea y type-checkea, sin codegen
    fn agregar(item: T)
```

### Structs genéricos ✅

```kl
struct Par<T, U>:                   # ✅ parsea, type-checkea, genera código
    primero: T
    segundo: U
```

### Uso ✅

```kl
numeros = Pair<i32, i32> { first: 1, second: 2 }  # ✅ monomorfización funcionando
```

---

## 22. Optionals y Optional Chaining

### Option<T> ✅

```kl
nombre: Option<str> = None          # ✅
nombre = Option.Some("Ana")         # ✅
```

### Optional Chaining ?. 🔶

```kl
nombre = usuario?.nombre            # 🔶 parsea y type-checkea, sin codegen
ciudad = usuario?.direccion?.ciudad # 🔶
```

Equivale a:

```kl
nombre = usuario ? usuario.nombre : None
```

### Pattern matching con Option ✅

```kl
match nombre:                       # ✅
    Option.Some(valor):
        print(valor)
    Option.None:
        print("sin nombre")
```

---

## 23. Error Handling

### Tipos de error ! 🔶

```kl
fn encontrar_usuario(id: i32) -> Usuario!  # 🔶 parsea y type-checkea, sin codegen
```

`!` indica que la función puede retornar un error.

### Operador ? ✅

```kl
fn procesar():
    usuario = encontrar_usuario(1)? # ✅ propaga el error (Option<T>)
    ...
```

### Match con error ✅

```kl
resultado = encontrar_usuario(1)    # ✅
match resultado:
    ok(usuario):
        print(usuario.nombre)
    error(err):
        print(err)
```

Kyle NO usa excepciones. Los errores son valores explícitos.

---

## 24. Async / Await ✅

### Función asíncrona ✅

```kl
async fn cargar_usuarios() -> [Usuario]:  # ✅
    ...
```

### Ejecución directa ✅

```kl
usuarios = await cargar_usuarios()    # ✅
```

### Tarea en segundo plano ✅

```kl
tarea = async cargar_usuarios()       # ✅
# ... otro trabajo ...
usuarios = await tarea                # ✅
```

El runtime actual usa threads (`kl_spawn_thread` / `kl_join_thread`).

---

## 25. Imports y Módulos ✅

### Import de módulo ✅

```kl
import math                          # ✅
import io                            # ✅
```

### Import específico ✅

```kl
from math import sqrt                # ✅
```

### Alias ✅

```kl
import database as db                # ✅
```

### Estructura de proyecto

```kl
src/
    main.kl                          # entry point
    models/
        usuario.kl
    services/
        auth.kl
```

---

## 26. Object Literals (Structura Tipada) ✅

```kl
usuario = {                          # ✅
    nombre: "Juan",
    edad: 25
}

print(usuario.nombre)                # ✅
print(usuario.edad)                  # ✅
```

### Anidados ✅

```kl
config = {                           # ✅
    database: {
        host: "localhost",
        puerto: 5432
    }
}

print(config.database.host)          # ✅
```

Son structuralmente tipados — no necesitan declaración de clase.

---

## 27. Dicts y Maps 📄

```kl
puntajes: Dict<str, i32>             # 📄 especificado, no implementado
puntajes["jugador1"] = 100           # 📄
```

Los object literals usan notación de punto.  
Los Dict usan notación de corchete `[]`.

---

## 28. Spread Operator 🔶

### En listas 🔶

```kl
combinada = [...lista_a, ...lista_b]  # 🔶 parsea, sin codegen
actualizada = [...original, nuevo]    # 🔶
```

### En object literals 🔶

```kl
defaults = { theme: "dark", lang: "en" }  # ✅
config = { ...defaults, lang: "es" }       # 🔶 sin codegen
```

---

## 29. Range y Slicing

### Range 🔶

```kl
for i in 0..10:                     # 🔶 parsea, sin codegen
rango = 0..10                       # 🔶
```

### Slicing de listas 🔶

```kl
items = [0, 1, 2, 3, 4, 5]         # ✅
primeros = items[0..3]              # 🔶 parsea, sin codegen
resto = items[3..]                  # 🔶
todos = items[..]                   # 🔶
```

---

## 30. Operator Overloading ✅

```kl
struct Vector:                       # ✅
    x: f64
    y: f64

    fn add(other: Vector) -> Vector: # ✅
        return Vector(x: x + other.x, y: y + other.y)

    fn mul(scalar: f64) -> Vector:   # ✅
        return Vector(x: x * scalar, y: y * scalar)
```

### Mapeo de operadores a métodos

| Operador | Método |
|----------|--------|
| `+` | `add` |
| `-` | `sub` |
| `*` | `mul` |
| `/` | `div` |
| `%` | `rem` |
| `**` | `pow` |
| `==` | `eq` |
| `!=` | `neq` |
| `<<` | `shl` |
| `>>` | `shr` |
| `&` | `bitand` |
| `|` | `bitor` |
| `^` | `bitxor` |

---

## 31. Compile-Time Evaluation (const fn) 📄

```kl
const fn factorial(n: i32) -> i32:   # 📄 especificado, no implementado
    if n <= 1:
        return 1
    return n * factorial(n - 1)

RESULTADO = factorial(10)            # 📄 evaluado en compilación
```

Restricciones de `const fn`:
- Sin FFI
- Sin I/O
- Sin heap allocations
- Sin polimorfismo runtime
- Solo cómputos puros con tipos primitivos

---

## 32. Type Aliases 🔶

```kl
type Entero = i32                    # 🔶 parsea y type-checkea, sin codegen
type Pares = (i32, i32)             # 🔶
type Callback = (i32) -> bool       # 🔶
```

---

## 33. Attributes / Annotations ✅

```kl
#[deprecated("usar nueva_api en su lugar")]  # ✅
fn vieja_api():
    ...

#[allow(dead_code)]                          # ✅
fn helper():
    ...

#[test]                                      # ✅
fn test_math():
    assert_eq(2 + 2, 4)

#[inline]                                    # ✅
fn path_rapido():
    ...
```

Los atributos van en línea separada antes de la declaración, empezando con `#[`.

---

## 34. Built-in Functions ✅

### String & Character

```kl
len(s)                    # i32 — longitud de s ✅
str(valor)                # str — convertir a string ✅
char_at(s, i)             # char — carácter en índice i ✅
ord(c)                    # i32 — código Unicode del char ✅
is_digit(c)               # i32 — 1 si es dígito ✅
is_alpha(c)               # i32 — 1 si es letra ✅
is_alnum(c)               # i32 — 1 si es alfanumérico ✅
is_whitespace(c)          # i32 — 1 si es espacio/tab/newline ✅
is_upper(c)               # i32 — 1 si es mayúscula ✅
is_lower(c)               # i32 — 1 si es minúscula ✅
contains(s, sub)          # i32 — 1 si s contiene sub ✅
to_upper(s)               # str — copia en mayúsculas ✅
to_lower(s)               # str — copia en minúsculas ✅
trim(s)                   # str — recorta espacios ✅
replace(s, from, to)      # str — reemplaza ocurrencias ✅
input(prompt)             # str — leer línea de stdin ✅
```

### I/O

```kl
print(valor)              # imprime sin newline ✅
println(valor)            # imprime con newline ✅
open(path, mode)          # i64 — file handle (0=read, 1=write) ✅
read_str(handle)          # str — leer archivo completo ✅
write_str(handle, text)   # escribir string a archivo ✅
close(handle)             # cerrar file handle ✅
```

### Time

```kl
sleep(ms)                 # dormir milisegundos ✅
now()                     # i64 — timestamp Unix en ms ✅
```

### Math

```kl
range(end)                # rango 0..end 🔶
range(start, end)         # rango start..end 🔶
abs(x)                    # valor absoluto (std/math.kl) ✅
pow(base, exp)            # potencia (std/math.kl) ✅
sqrt(x)                   # raíz cuadrada (std/math.kl) ✅
gcd(a, b)                 # máximo común divisor (std/math.kl) ✅
```

---

## 35. Comentarios ✅

### Línea

```kl
# esto es un comentario     # ✅
```

### Bloque

```kl
#[                           # ✅
    esto es un
    comentario de
    múltiples líneas
#]
```

---

## 36. Package Manager ✅

```bash
kl new proyecto             # crear nuevo proyecto ✅
kl init                     # alias de new ✅
kl add paquete@1.0          # agregar dependencia ✅
kl remove paquete           # remover dependencia ✅
kl info paquete             # mostrar información ✅
kl build                    # compilar proyecto ✅
kl run                      # compilar y ejecutar ✅
kl test                     # ejecutar tests ✅
```

### kl.toml (Manifest) ✅

```toml
[package]
name = "mi-proyecto"
version = "0.1.0"
```

---

## 37. Project Structure

```text
mi-proyecto/
├── kl.toml              # manifest ✅
├── kl.lock              # lock file ✅
├── src/
│   └── main.kl          # entry point ✅
├── tests/               # test directory ✅
└── lib/                 # library code
```

---

## 38. Características Removidas (NO existen en Kyle) ✅

```text
self          ❌ — se usa `this` para la instancia actual
let / var     ❌ — las variables se declaran directamente
public/private/protected  ❌ — visibilidad por naming (_ y __)
virtual       ❌ — no existe, los métodos son virtuales por defecto
override      ❌ — no existe, se overriden implícitamente
try/catch/finally  ❌ — no hay excepciones
export        ❌ — visibilidad por naming
pass          ❌ — método sin cuerpo = sin implementación
continue      ❌ — el flujo sigue naturalmente
spawn         ❌ — se usa `async expr`
{} (bloques)  ❌ — se usa indentación
;             ❌ — newline termina statements
usuario["nombre"]  ❌ — los object literals usan punto
```

---

## Resumen de Estado por Categoría

| Categoría | Estado |
|-----------|--------|
| Variables y mutabilidad | ✅ Completo |
| Tipos primitivos | ✅ Completo |
| Constantes | ✅ Completo |
| Operadores (todos) | ✅ Completo |
| Funciones (incl. params por defecto, nombrados, variádicas) | ✅ Completo |
| Closures | ✅ Completo |
| If/Elif/Else | ✅ Completo |
| Binding If | ✅ Completo |
| While / Loop / While-Bind / While-Else | ✅ Completo |
| **For / For-Else / For Range** | **✅ Listas completo** / **🔶 Range/Else** |
| Match + guards + enum variants | ✅ Completo |
| Break (con y sin valor) | ✅ Completo |
| **Defer** | **✅ Completo** |
| **Guard** | **✅ Completo** |
| Strings + chars + escapes | ✅ Completo |
| **String Interpolation** | **✅ Completo** |
| Listas + indexación + métodos | ✅ Completo |
| Tuplas + destructuring | ✅ Completo |
| Structs | ✅ Completo |
| Enums (con payload) | ✅ Completo |
| Clases + constructores + métodos | ✅ Completo |
| Herencia + polimorfismo | ✅ Completo |
| Clases abstractas | ✅ Completo |
| Contracts (interfaces) | ✅ Completo |
| Properties (get/set) | ✅ Completo |
| Visibilidad por naming | ✅ Completo |
| **Genéricos** | **✅ Completo** |
| Option<T> + Option.Some/None | ✅ Completo |
| **Optional Chaining ?.** | **🔶 Sin codegen** |
| Error types ! | ✅ Parse + type-check, **🔶 sin codegen** |
| **Operador ?** | **✅ Completo** |
| Async/Await | ✅ Completo |
| Imports (import, from, alias) | ✅ Completo |
| Object literals { } | ✅ Completo |
| **Dict< K, V>** | **📄 Solo especificado** |
| **Spread operator (...)** | **✅ Completo** |
| **Range + Slicing** | **✅ Completo** |
| Operator overloading | ✅ Completo |
| **const fn** | **📄 Solo especificado** |
| **Type aliases** | **✅ Completo** |
| Attributes (#[ ]) | ✅ Completo |
| Built-in functions | ✅ Completo |
| Comentarios (# y #[#]) | ✅ Completo |
| Package manager | ✅ Completo |

---

## Prioridades Actuales (Phase 6 — Language Completion)

### 🟥 P0 — End-to-end language features (bloquean el MVP)

1. **For loops** — ✅ **COMPLETED**
2. **Genéricos** — ✅ **COMPLETED**
3. **Error handling** — `!` y `?` ✅ **COMPLETED**
4. **Optional chaining** — `?.` lowering + codegen (⚠️ PARCIAL)
5. **String interpolation** — ✅ **COMPLETED**

### 🟧 P1 — Secondary features

6. Defer — ✅ **COMPLETED** (Sesión 25, LIFO lowering)
7. Type aliases — ✅ **COMPLETED** (Sesión 25)
8. Dict/Map literals — parse + type-check + codegen
9. Spread operator — ✅ **COMPLETED** (Sesión 25)
10. Range slicing — ✅ **COMPLETED** (Sesión 25)
11. const fn — compile-time evaluation
12. If como expresión (bloques) — codegen (ternary cubre 90% casos)
13. Match como expresión — ✅ **COMPLETED** (Sesión 24)
14. Guard — ✅ **COMPLETED** (Sesión 24, CondBr lowering)
15. Standard library — collections, json, str, time

### 🟪 P4 — Tooling polish

15. LSP autocompletion, go-to-definition, hover
16. Debug info (DWARF)

### 🟩 P5 — Robustness & testing

17. Tests de integración (100+)
18. LLVM verification errors
19. CI pipeline

---

## Versión

```text
Kyle Syntax Reference v1.0
Idioma: Español
Basado en: Kyle Language Specification v2.0
Última actualización: 2026-06-22
Prioridades: Phase 6 (Language Completion) → P0-P5
```
