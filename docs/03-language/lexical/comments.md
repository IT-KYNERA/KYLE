# Comments

> Los comentarios en Kyle usan `#` hasta el final de la línea.
> No hay comentarios multilínea (`/* */`).

## Comentarios de línea

```ky
# Esto es un comentario
x: i32 = 42  # comentario inline

fn suma(a: i32, b: i32) i32:
    # Los comentarios pueden estar en bloques indentados
    a + b
```

## No hay `/* */`

Kyle no soporta comentarios multilínea estilo C (`/* ... */`).
Usar `#` en cada línea:

```ky
# Esta es una
# explicación
# multilínea
```

## Doc comments

```ky
# Esta función suma dos números
# @param a: primer número
# @param b: segundo número
# @returns: suma
fn add(a: i32, b: i32) i32:
    a + b
```

## Ver también

- `literals.md` — Literales del lenguaje
- `identifiers.md` — Reglas de identificadores
