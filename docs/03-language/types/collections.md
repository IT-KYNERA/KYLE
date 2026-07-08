# Collections

> Typis de collections integradas en Kyle.

## Comparison

| Type | Mutabilidad | Acceso | Uso |
|------|-------------|--------|-----|
| `{T}` | `push`/`pop`/`set` | Por index | List dynamic |
| `{K: V}` | `set`/`remove` | Por key | Dictionary |
| `set<T>` | `add`/`remove` | Por value | Set unico |
| `[T; N]` | `arr[i] = val` | Por index | Array fijo |

## Copy vs Move

Todos typis de collections are **Move** (no se copian implicitamente):

```ky
a: {i32} = {1, 2, 3}
b: {i32} = a # MOVE: a invalido
b = a.clone() # COPY explicita
```

## Iteration

```ky
for val in list:
 println(val.to_str())

for key in dict:
 println("key: " + key)

for val in set:
 println(val.to_str())
```

## See also

- `compound-types.md` — Array, List, Tuple, Dict
- `04-standard-library/collections.md` — API completa
