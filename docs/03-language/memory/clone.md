# Clone

> Copia explícita para tipos Move mediante `.clone()`.

## Propósito

Los tipos Move (`str`, `{T}`, `{K:V}`, `[T; N]`, clases) no se pueden copiar
implícitamente con `y = x`. Para crear una copia independiente se usa `.clone()`.

## Uso

```ky
a: str = "hello"
b: str = a.clone()    # COPY explícita: ambos vivos
println(a)            # ✅ "hello"
println(b)            # ✅ "hello"
```

## Clone en listas

```ky
original: {i32} = {1, 2, 3}
copia: {i32} = original.clone()
original.push(4)
println(copia.len().to_str())    # 3 (independiente)
println(original.len().to_str()) # 4
```

## Clone en arrays

```ky
arr: [i32; 3] = [1, 2, 3]
copia: [i32; 3] = arr.clone()
```

## Clone en clases

```ky
class Point:
    x: i32
    y: i32

    fn clone() Point:
        Point(x, y)

p: Point = Point { x: 10, y: 20 }
q: Point = p.clone()
```

## Rendimiento

`.clone()` hace una copia **profunda** (deep copy). Para strings grandes o listas
largas, puede ser costoso. Usar borrow (`&`) cuando sea posible.

## Ver también

- `move.md` — Move semantics (contraste con clone)
- `copy.md` — Copy semantics (automático, sin clone)
