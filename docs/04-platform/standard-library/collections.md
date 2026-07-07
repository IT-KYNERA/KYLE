# Collections

> **Regla general:** `{T}` (list) cubre Stack y Queue vía métodos.
> Tipos dedicados solo para `Set<T>` (hash set) y `{K:V}` (dict).

---

## List: `{T}` [x]

```ky
v = {1, 2, 3}
v.push(4)
v.reserve(100)
x = v[0]
v[0] = 99
v.pop()          # LIFO: saca el último
v.popFirst()     # FIFO: saca el primero
v.len()
v.contains(10)
v.insert(1, 50)
v.removeAt(2)
v.clear()
```

**Como Stack:** `push()` + `pop()` (LIFO — ya funciona).
**Como Queue:** `push()` + `popFirst()` (FIFO).

## Dict: `{K: V}` [x]

```ky
d = {"name": "Kyle", "age": 30}
d["city"] = "NYC"
name = d["name"]
d.len()
```

## Set: `Set<T>` [ ]

```ky
s: Set<i32> = Set{1, 2, 3}
s.add(4)
s.contains(1)    # → true
s.remove(1)
s.len()
for val in s:
    println(val.toStr())
```
