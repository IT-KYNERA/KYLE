# Copy Semantics

> Los tipos numéricos y primitivos se copian automáticamente en `y = x`.

## Tipos Copy

| Tipo | Tamaño | Descripción |
|------|--------|-------------|
| `i8` | 1 byte | Signed 8-bit |
| `i16` | 2 bytes | Signed 16-bit |
| `i32` | 4 bytes | Signed 32-bit |
| `i64` | 8 bytes | Signed 64-bit |
| `u8` | 1 byte | Unsigned 8-bit |
| `u16` | 2 bytes | Unsigned 16-bit |
| `u32` | 4 bytes | Unsigned 32-bit |
| `u64` | 8 bytes | Unsigned 64-bit |
| `f32` | 4 bytes | Float 32-bit |
| `f64` | 8 bytes | Float 64-bit |
| `bool` | 1 byte | `true` / `false` |
| `char` | 4 bytes | Unicode |
| `ptr` | 8 bytes | Raw pointer |

## Comportamiento

```ky
x: i32 = 42
y: i32 = x          # COPY: ambos vivos
println(x)           # ✅ 42
println(y)           # ✅ 42

a: f64 = 3.14
b: f64 = a           # COPY
b = 2.71
println(a)           # ✅ 3.14 (a no se modifica)
```

## Dato curioso

Los tipos Copy son los que caben en **≤ 8 bytes** y no tienen memoria heap asociada.
Se copian con una instrucción de CPU (`mov`), no hay diferencia de rendimiento
entre pasar por valor o por referencia para estos tipos.

## Ver también

- `move.md` — Move semantics
- `clone.md` — Clone explícito para tipos Move
