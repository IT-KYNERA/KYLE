# Collections

> Tipos de colecciones integradas en Kyle.

## Comparación

| Tipo | Mutabilidad | Acceso | Uso |
|------|-------------|--------|-----|
| `{T}` | `push`/`pop`/`set` | Por índice | Lista dinámica |
| `{K: V}` | `set`/`remove` | Por key | Diccionario |
| `set<T>` | `add`/`remove` | Por valor | Set único |
| `[T; N]` | `arr[i] = val` | Por índice | Array fijo |

## Copy vs Move

Todos los tipos de colecciones son **Move** (no se copian implícitamente):

```ky
a: {i32} = {1, 2, 3}
b: {i32} = a           # MOVE: a inválido
b = a.clone()           # COPY explícita
```

## Iteración

```ky
for val in lista:
    println(val.to_str())

for key in dict:
    println("key: " + key)

for val in set:
    println(val.to_str())
```

## Ver también

- `compound-types.md` — Array, List, Tuple, Dict
- `04-standard-library/collections.md` — API completa
