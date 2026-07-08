# io — Entrada / Salida

> Módulo de entrada y salida por consola.
> Import: `from io import console`

## console: lectura y escritura en terminal

```ky
from io import console

console.print("hello")         # sin newline
console.println("hello")       # con newline
line: str = console.input()    # leer línea
line = console.input("> ")     # leer línea con prompt
```

### Shorthands globales

Las funciones `print()` y `println()` están disponibles globalmente sin import:

```ky
print("hello")          # console.print()
println("hello")        # console.println()
input("> ")             # console.input()
```

### Métodos de console

| Nombre | Firma | Descripción |
|--------|-------|-------------|
| `console.print` | `fn(text: str)` | Imprimir texto sin salto |
| `console.println` | `fn(text: str)` | Imprimir texto con salto |
| `console.input` | `fn(prompt: str) str` | Leer línea con prompt |
| `console.clear` | `fn()` | Limpiar terminal |

### Ejemplos

```ky
from io import console

name: str = console.input("¿Cómo te llamas? ")
console.println("Hola, " + name + "!")

# Equivalente con shorthands globales
name = input("¿Cómo te llamas? ")
println("Hola, " + name + "!")
```
