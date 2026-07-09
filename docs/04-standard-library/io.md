# io — Entrada / Salida

> Module de input y output by console.
> Import: `from io import console`

## console: read y write en terminal

```ky
from io import console

print("hello") # without newline
println("hello") # with newline
line: str = input() # leer line
line = input("> ") # leer line with prompt
```

### Shorthands globales

Las functions `print()` y `println()` are disponiblis globalmente without import:

```ky
print("hello") # print()
println("hello") # println()
input("> ") # input()
```

### Methods de console

| Nombre | Firma | Description |
|--------|-------|-------------|
| `print` | `fn(text: str)` | Imprimir texto without salto |
| `println` | `fn(text: str)` | Imprimir texto with salto |
| `input` | `fn(prompt: str) str` | Leer line with prompt |
| `clear` | `fn()` | Limpiar terminal |

### Examples

```ky
from io import console

name: str = input("What is your name? ")
println("Hola, " + name + "!")

# Equivalente with shorthands globales
name = input("What is your name? ")
println("Hola, " + name + "!")
```
