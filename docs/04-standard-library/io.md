# io — Entrada / Salida

> Módulo de entrada y salida por consola.
> Import: `from io import console`

## console: lectura y escritura en terminal

```ky
from io import console

print("hello")         # sin newline
println("hello")       # con newline
line: str = input()    # leer línea
line = input("> ")     # leer línea con prompt
```

### Shorthands globales

Las funciones `print()` y `println()` están disponibles globalmente sin import:

```ky
print("hello")          # print()
println("hello")        # println()
input("> ")             # input()
```

### Métodos de console

| Nombre | Firma | Descripción |
|--------|-------|-------------|
| `print` | `fn(text: str)` | Imprimir texto sin salto |
| `println` | `fn(text: str)` | Imprimir texto con salto |
| `input` | `fn(prompt: str) str` | Leer línea con prompt |
| `clear` | `fn()` | Limpiar terminal |

### Ejemplos

```ky
from io import console

name: str = input("¿Cómo te llamas? ")
println("Hola, " + name + "!")

# Equivalente con shorthands globales
name = input("¿Cómo te llamas? ")
println("Hola, " + name + "!")
```
