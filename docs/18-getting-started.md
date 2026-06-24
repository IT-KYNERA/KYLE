# Kyle — Guía de Inicio Rápido

## 1. Instalación y Compilación del Compilador

```bash
# Clonar el repo
git clone <repo-url>
cd kl

# Compilar el compilador (Rust toolchain requerida)
cargo build --workspace

# El binario se llama `klc` (también disponible como `kl`)
cargo run --bin klc -- help
```

Alias recomendado:
```bash
alias kl='cargo run --bin klc --'
```

---

## 2. Crear un Proyecto Nuevo

```bash
kl new mi-proyecto
```

Esto crea:

```
mi-proyecto/
├── kl.toml           # Manifiesto del proyecto
├── src/
│   └── main.kl       # Punto de entrada
└── tests/            # Carpeta para tests
```

### kl.toml (generado automáticamente)

```toml
name = "mi-proyecto"
version = "0.1.0"
edition = "1"
authors = []
license = "MIT"

[compiler]
optimization = "O2"
target = "native"

[dependencies]
```

---

## 3. Punto de Entrada

**Archivo:** `src/main.kl`
**Función:** `fn main`

Todo programa ejecutable DEBE tener una función `main`. Formas posibles:

```kl
# Mínima (sin args, sin retorno)
fn main():
    print("Hola Mundo")

# Con argumentos de línea de comandos
fn main(args: [str]):
    for arg in args:
        println(arg)

# Con retorno (código de salida: 0 = éxito)
fn main() -> i32:
    println("Todo bien")
    return 0
```

---

## 4. Compilar y Ejecutar

### Modo proyecto (src/main.kl + kl.toml)

```bash
cd mi-proyecto
kl run          # Compila y ejecuta
kl build        # Solo compila (genera target/main)
```

### Modo archivo suelto

```bash
kl run examples/hola.kl      # Compila y ejecuta
kl build examples/hola.kl    # Solo compila → genera ./hola
```

### Otros comandos

```bash
kl check archivo.kl     # Solo type-check (sin codegen)
kl parse archivo.kl     # Mostrar el AST
kl mir archivo.kl       # Mostrar la representación intermedia MIR
kl fmt archivo.kl       # Formatear código
kl fmt archivo.kl --check   # Verificar formato (sin modificar)
```

---

## 5. Sintaxis Básica — Lo que ya funciona

### Variables

```kl
x = 5                    # Inferido como i32
name = "Kyle"            # str (inmutable por defecto)
mut total = 0            # mutable con `mut`
total = total + 1        # ok
```

### Tipos explícitos

```kl
edad: i32 = 25
mut nombres: [str]       # lista vacía de strings
```

### Tipos disponibles

| Tipo | Descripción | Literal |
|------|-------------|---------|
| `i32` | Entero 32 bits | `42` |
| `i64` | Entero 64 bits | (vía cast) |
| `f32`, `f64` | Flotante | `3.14` |
| `bool` | Booleano | `true`, `false` |
| `str` | Cadena UTF-8 | `"hola"` |
| `char` | Carácter | `'A'` |
| `[T]` o `list<T>` | Lista | `[1, 2, 3]` |
| `Dict<K, V>` | Diccionario | `{"key": value}` |
| `struct` | Estructura | ver abajo |

### Strings

```kl
s = "Hola, Mundo!"
len(s)              # → i32 (longitud)
contains(s, "Hola") # → bool (contiene substring)
to_upper(s)         # → str (MAYÚSCULAS)
to_lower(s)         # → str (minúsculas)
trim(s)             # → str (sin espacios extremos)
replace(s, "a", "b") # → str (reemplazar)
substr(s, 0, 4)     # → str (subcadena)
char_at(s, 0)       # → char (carácter en posición)
```

### If / Elif / Else

```kl
if edad >= 18:
    println("Adulto")
elif edad >= 13:
    println("Adolescente")
else:
    println("Niño")
```

### For loops

```kl
items = [10, 20, 30]
for item in items:
    print(item)

# for con rango: 🔶 parsea pero no genera código aún
```

### While

```kl
mut i = 0
while i < 5:
    println(i)
    i = i + 1
```

### Match

```kl
match x:
    0 => println("cero")
    1 => println("uno")
    _ => println("otro")
```

### Funciones

```kl
fn suma(a: i32, b: i32) -> i32:
    return a + b

# Con valor por defecto
fn saludar(nombre: str = "Mundo"):
    println("Hola " + nombre)

# Sin retorno explícito (void)
fn solo_imprimir():
    println("ok")
```

### Listas

```kl
mut nums = [1, 2, 3]
nums.add(4)          # agregar al final
nums.pop()           # sacar el último
print(nums[0])       # indexar
print(len(nums))     # longitud
for n in nums:
    println(n)
```

### Diccionarios

```kl
d = {"alice": 25, "bob": 30}
print(d["alice"])     # get → 25
d["bob"] = 31         # set
print(d.len())        # len → 2
```

### Structs

```kl
struct Punto:
    x: i32
    y: i32

fn main():
    mut p: Punto
    p.x = 10
    p.y = 20
    print(str(p.x) + ", " + str(p.y))
```

### Enums + Match

```kl
enum Option:
    Some(val: i32)
    None

fn main():
    v = Option.Some(42)
    match v:
        Option.Some(n) => println(str(n))
        Option.None => println("nada")
```

### Closures

```kl
doble = (x) => x * 2
resultado = doble(21)  # → 42
```

### Genéricos

```kl
struct Par<T, U>:
    primero: T
    segundo: U

fn main():
    p = Par { primero: 1, segundo: "hola" }
    println(str(p.primero))
```

### Async/Await

```kl
task = async 42
resultado = await task
println(resultado)
```

### Operador ternario

```kl
status = edad >= 18 ? "Adulto" : "Menor"
```

### Defer (LIFO)

```kl
fn ejemplo():
    defer println("segundo")
    defer println("primero")  # se ejecuta al salir de la función
    println("dentro")
```

### Type aliases

```kl
type Edad = i32
type Años = Edad
```

### Operador `?` (propagación de error)

```kl
valor = funcion_riesgosa()?
```

### Spread en listas

```kl
a = [1, 2, 3]
b = [...a, 4, 5]  # → [1, 2, 3, 4, 5]
```

### Range slicing

```kl
items = [0, 1, 2, 3, 4, 5]
slice = items[1..4]  # → [1, 2, 3]
```

---

## 6. Funciones Built-in (Runtime)

Sin importar nada:

| Función | Retorno | Descripción |
|---------|---------|-------------|
| `print(val)` | void | Imprime sin salto de línea |
| `println(val)` | void | Imprime con salto de línea |
| `print_err(val)` | void | Imprime a stderr |
| `len(s)` | i32 | Longitud de string o lista |
| `str(val)` | string | Convierte a string |
| `contains(s, sub)` | bool | Verifica substring |
| `to_upper(s)` | string | A mayúsculas |
| `to_lower(s)` | string | A minúsculas |
| `trim(s)` | string | Sin espacios extremos |
| `replace(s, a, b)` | string | Reemplazar texto |
| `substr(s, i, n)` | string | Subcadena |
| `char_at(s, i)` | char | Carácter en posición |
| `ord(c)` | i32 | Código Unicode |
| `is_digit(c)` | bool | Es dígito |
| `is_alpha(c)` | bool | Es letra |
| `is_alnum(c)` | bool | Es alfanumérico |
| `is_upper(c)` | bool | Es mayúscula |
| `is_lower(c)` | bool | Es minúscula |
| `input()` | string | Leer línea de stdin |
| `input("prompt: ")` | string | Leer línea con mensaje |
| `open(path, mode)` | i64 | Abrir archivo (0=read,1=write) |
| `read_str(h)` | string | Leer archivo completo |
| `write_str(h, txt)` | void | Escribir a archivo |
| `close(h)` | void | Cerrar archivo |
| `sleep(ms)` | void | Dormir milisegundos |
| `now()` | i64 | Timestamp Unix en ms |

---

## 7. Importar Módulos

Puedes separar código en múltiples archivos usando `import`:

```kl
# archivo: mis_utilidades.kl (en la misma carpeta)
fn suma(a: i32, b: i32) -> i32:
    return a + b

fn saludo(nombre: str):
    println("Hola " + nombre + "!")
```

```kl
# archivo: main.kl
import mis_utilidades

fn main():
    # Llamada directa (recomendada)
    r = suma(10, 20)
    println("10 + 20 = " + str(r))

    # Llamada con calificador de módulo
    saludo("Mundo")
```

```bash
kl run main.kl    # Busca mis_utilidades.kl automáticamente
```

---

## 8. Tests

```kl
# tests/mis_tests.kl
fn test_suma():
    assert(1 + 1 == 2)
    assert_eq(2 + 2, 4)
    assert_str("hola", "hola")

fn main():
    test_suma()
```

Ejecutar:
```bash
kl test
```

---

## 9. Formateo de Código

```bash
kl fmt archivo.kl           # Formatea en el lugar
kl fmt archivo.kl --check   # Verifica sin modificar
```

---

## 10. Lo que NO funciona aún (Phase 6 gaps)

| Feature | Estado |
|---------|--------|
| Optional chaining (`?.`) | ❌ Parse + type-check, **no codegen** |
| Error types (`!`) | ❌ Parse + type-check, **no codegen** |
| Contracts (`contract` / `implements`) | ❌ Parse + type-check, **no lowering** |
| For Range (`for i in 0..10`) | 🔶 Parse, **no codegen** |
| For-Else | ❌ Parse, **no codegen** |
| `const fn` | 📄 Solo especificado en docs |
| LSP completion / hover / goto-def | ❌ No implementado |
| Debug info (DWARF) | ❌ No implementado |

---

## 11. Resumen Rápido

```bash
kl new mi-app          # Crear proyecto
cd mi-app
kl run                 # Compilar y ejecutar
kl build               # Solo compilar
kl check src/main.kl   # Solo type-check
kl test                # Ejecutar tests
```

Archivo mínimo funcional (`src/main.kl`):

```kl
fn main():
    println("Hola Kyle!")
```

Todo lo demás en esta guía **funciona hoy** — puedes crear archivos `.kl`, usar el CLI, y probar todas las features marcadas con ✅.
