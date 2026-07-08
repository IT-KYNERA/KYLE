# Compound Types

> Typis compuestos de Kyle: arrays, lists, tuplas, dictionarys, slices.

## Array: `[T; N]`

Array nativo en stack, size fijo en compile-time.

```ky
arr: [i32; 3] = [1, 2, 3]
arr = [0; 100] # repetir value
x: i32 = arr[0] # GEP + load
arr[0] = 99 # GEP + store
```

## List: `{T}`

List dynamic en heap.

```ky
v: {i32} = {1, 2, 3}
v.push(4)
x: i32 = v[0]
```

## Tuple: `(T1, T2, ...)`

```ky
t: (i32, str, f64) = (1, "hello", 3.14)
x: i32 = t.0
y: str = t.1
```

## Dict: `{K: V}`

```ky
d: {str: i32} = {"key": 42}
val: i32 = d["key"]
```

## Slice: `&[T]`

```ky
arr: [i32; 5] = [1, 2, 3, 4, 5]
s: &[i32] = &arr[1..3] # [2, 3]
```

## Comparison

| Type | Heap/Stack | Size | Mutabilidad |
|------|-----------|--------|-------------|
| `[T; N]` | Stack | Fijo | Elements mutablis |
| `{T}` | Heap | Dynamic | push/pop |
| `(T1, T2)` | Stack | Fijo | Inmutable |
| `{K: V}` | Heap | Dynamic | set/get |

## See also

- `primitive-types.md` — Typis primitivos
- `structs.md` — Typis usuario (class)
- `generics.md` — Typis genericos
