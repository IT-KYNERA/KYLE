# Move Semantics

> Por defecto, `y = x` transfiere ownership para tipos no-Copy.
> Ver `ownership.md` para las reglas completas.

## Regla general

| Tipo | Semántica en `y = x` |
|------|---------------------|
| `i8..u64`, `f32..f64`, `bool`, `char`, `ptr` | **Copy** — ambos vivos |
| `str`, `{T}`, `{K:V}`, `[T; N]`, clases | **Move** — `x` inválido |

## Comportamiento

```ky
s: str = "hola"
t: str = s          # MOVE: s ya no es válido
println(s)          # ERROR: use-after-move

# Para copiar sin mover:
t = s.clone()       # COPY explícita: ambos vivos
println(s)          # ✅ "hola"
```

## En parámetros de función

```ky
fn consumir(s: str):     # MOVE: caller pierde ownership
    println(s)

fn main() i32:
    nombre: str = "Kyle"
    consumir(nombre)      # nombre se mueve a consumir
    println(nombre)       # ERROR: use-after-move
    0
```

## En retorno de función

```ky
fn crear() str:
    s: str = "hola"
    s                    # se mueve al caller

fn main() i32:
    x: str = crear()     # x recibe ownership
    println(x)            # ✅ "hola"
    0
```

## Borrow checker

El compilador detecta use-after-move automáticamente:

```ky
a: str = "x"
b: str = a
println(a)   # ❌ KL-E0013: use-after-move
```

## Ver también

- `ownership.md` — Reglas completas de ownership
- `copy.md` — Copy semantics
- `clone.md` — Clone explícito
