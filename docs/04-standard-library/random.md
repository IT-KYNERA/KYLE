# random — Aleatoriedad

> Módulo de generación de números aleatorios.
> Import: `from random import random`

## random: números aleatorios

```ky
from random import random

n: i32 = random.int(100)              # 0..99
n = random.int_range(10, 20)          # 10..19
x: f64 = random.float()               # 0.0..1.0
x = random.float_range(0.0, 10.0)     # 0.0..10.0
b: bool = random.bool()               # true o false
random.shuffle(lista)                 # mezclar in-place
elem: i32 = random.choice(lista)      # elemento aleatorio
```

### Funciones

| Función | Firma | Descripción |
|---------|-------|-------------|
| `random.int(max)` | `fn(max: i32) i32` | Entero 0..max-1 |
| `random.int_range(min, max)` | `fn(min: i32, max: i32) i32` | Entero min..max-1 |
| `random.float()` | `fn() f64` | Flotante 0.0..1.0 |
| `random.float_range(min, max)` | `fn(min: f64, max: f64) f64` | Flotante min..max |
| `random.bool()` | `fn() bool` | Booleano aleatorio |
| `random.shuffle(list)` | `fn(self: &{T})` | Mezclar in-place |
| `random.choice(list)` | `fn(self: &{T}) T` | Elemento aleatorio |

### Ejemplo

```ky
from random import random

dado: i32 = random.int_range(1, 7)
println("dado: " + dado.to_str())

cartas: {i32} = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10}
random.shuffle(cartas)
println("primera carta: " + cartas[0].to_str())
```
