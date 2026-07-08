# random — Aleatoriedad

> Módulo de generación de números aleatorios.
> Import: `from random import random`

## random: números aleatorios

```ky
from random import random

# Enteros
n = random.int(100)           # 0..99
n = random.int_range(10, 20)  # 10..19

# Flotantes
x = random.float()            # 0.0..1.0
x = random.float_range(0, 10) # 0.0..10.0

# Booleanos
b = random.bool()             # true o false

# Utilidades
random.shuffle(list)          # mezclar lista in-place
elem = random.choice(list)    # elemento aleatorio de lista
```

### Funciones

| Función | Descripción |
|---------|-------------|
| `random.int(max)` | Entero aleatorio 0..max-1 |
| `random.int_range(min, max)` | Entero aleatorio min..max-1 |
| `random.float()` | Flotante aleatorio 0.0..1.0 |
| `random.float_range(min, max)` | Flotante aleatorio min..max |
| `random.bool()` | Booleano aleatorio |
| `random.shuffle(list)` | Mezclar lista in-place |
| `random.choice(list)` | Elemento aleatorio de lista |

### Ejemplo

```ky
from random import random

# Simular dado
dado = random.int_range(1, 7)
println("dado: " + dado.to_str())

# Barajar cartas
cartas = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10}
random.shuffle(cartas)
println("primera carta: " + cartas[0].to_str())
```
