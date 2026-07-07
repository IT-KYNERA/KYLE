# Collections

> **Lista:** `{T}` [x] — Dinámica, heap.
> **Dict:** `{K: V}` [x] — Hash map.
> **Set:** `Set<T>` [ ] — Hash set.
> **Queue:** `Queue<T>` [ ] — FIFO.
> **Stack:** `Stack<T>` [ ] — LIFO.

---

## List: `{T}` [x]

```ky
v = {1, 2, 3}
v.push(4)
v.reserve(100)
x = v[0]
v[0] = 99
v.pop()
v.len()
v.contains(10)
v.insert(1, 50)
v.remove_at(2)
v.clear()
```

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
    println(val.to_str())
```

## Queue: `Queue<T>` [ ]

```ky
q: Queue<i32> = Queue{}
q.push(10)       # enqueue
q.push(20)
val = q.pop()    # dequeue → 10 (FIFO)
q.len()
```

## Stack: `Stack<T>` [ ]

```ky
st: Stack<i32> = Stack{}
st.push(10)
st.push(20)
val = st.pop()   # → 20 (LIFO)
st.len()
```
