# math — Matemáticas

> Módulo de funciones matemáticas.
> Import: `from math import math`

## math: funciones matemáticas

```ky
from math import math

x = math.max(10, 20)         # → 20
x = math.min(10, 20)         # → 10
x = math.abs(-5)             # → 5
x = math.pow(2, 10)          # → 1024
x = math.sqrt(144)           # → 12 (f64)
x = math.floor(3.7)          # → 3 (i64)
x = math.ceil(3.2)           # → 4 (i64)
x = math.round(3.5)          # → 4 (i64)
x = math.clamp(15, 0, 10)   # → 10 (limita a rango)
```

### Aritméticas

| Función | Descripción | Retorno |
|---------|-------------|---------|
| `math.max(a, b)` | Mayor de dos valores | i32/i64/f64 |
| `math.min(a, b)` | Menor de dos valores | i32/i64/f64 |
| `math.abs(x)` | Valor absoluto | i32/i64/f64 |
| `math.pow(base, exp)` | Potencia | i64 |
| `math.sqrt(x)` | Raíz cuadrada | f64 |

### Redondeo

| Función | Descripción |
|---------|-------------|
| `math.floor(x)` | Redondear hacia abajo |
| `math.ceil(x)` | Redondear hacia arriba |
| `math.round(x)` | Redondear (0.5 hacia arriba) |

### Utilidades

| Función | Descripción |
|---------|-------------|
| `math.clamp(val, min, max)` | Limitar valor a rango |
| `math.lerp(a, b, t)` | Interpolación lineal (`a + (b-a) * t`) |

### Constantes

```ky
println(math.pi)    # 3.141592653589793
println(math.e)     # 2.718281828459045
```

### Ejemplo

```ky
from math import math

fn solve_quadratic(a: f64, b: f64, c: f64) (f64, f64)!:
    disc = b * b - 4 * a * c
    if disc < 0:
        return error("no real roots")
    sqrt_disc = math.sqrt(disc)
    x1 = (-b + sqrt_disc) / (2 * a)
    x2 = (-b - sqrt_disc) / (2 * a)
    ok((x1, x2))

match solve_quadratic(1, -3, 2):
    ok((x1, x2)):
        println("x1: " + x1.to_str() + ", x2: " + x2.to_str())
    error(e): println(e)
```
