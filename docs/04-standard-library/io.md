# io — Entrada / Salida

> Módulo de entrada y salida por consola.
> Import: `from io import console`

## console: lectura y escritura en terminal

```ky
from io import console

console.print("hello")         # sin newline
console.println("hello")       # con newline
line = console.input()         # leer línea
line = console.input("> ")     # leer línea con prompt
```

### Funciones

| Función | Descripción |
|---------|-------------|
| `console.print(text)` | Imprimir texto sin salto de línea |
| `console.println(text)` | Imprimir texto con salto de línea |
| `console.input()` | Leer línea desde stdin |
| `console.input(prompt)` | Leer línea con prompt |
| `console.clear()` | Limpiar terminal |

### Ejemplo

```ky
from io import console

name = console.input("¿Cómo te llamas? ")
console.println("Hola, " + name + "!")
```
