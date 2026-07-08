# Primer Programa

> Crear y ejecutar tu primer programa Kyle.

## Hola Mundo

Crear un archivo `hola.ky`:

```ky
fn main() i32:
    println("Hola, Kyle!")
    0
```

Ejecutar:

```bash
ky run hola.ky
```

Salida:

```
Hola, Kyle!
```

## Explicación

| Parte | Significado |
|-------|-------------|
| `fn main() i32:` | Punto de entrada del programa. Retorna `i32` (código de salida) |
| `println(...)` | Función global para imprimir texto con salto de línea |
| `0` | La última expresión es el retorno (código 0 = éxito) |

## Compilar sin ejecutar

```bash
ky build hola.ky
./hola               # ejecutar el binario
```

## Variables

```ky
fn main() i32:
    nombre: str = "Mundo"
    println("Hola, " + nombre + "!")
    0
```

## Entrada del usuario

```ky
fn main() i32:
    nombre: str = input("¿Cómo te llamas? ")
    println("Hola, " + nombre + "!")
    0
```

## Ver también

- `build.md` — Compilar proyectos
- `project-layout.md` — Estructura de proyectos
- `03-language/syntax/variables.md` — Variables en Kyle
