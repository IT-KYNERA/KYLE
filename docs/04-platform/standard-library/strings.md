# std.str — String Utilities

| Function | Description |
|----------|-------------|
| `str.starts_with(s, prefix)` | Check if string starts with prefix |
| `str.ends_with(s, suffix)` | Check if string ends with suffix |
| `str.capitalize(s)` | Capitalize first letter |
| `str.repeat(s, count)` | Repeat string n times |

## str_builder — Efficient String Building

`str_builder` is a growable string buffer for efficient concatenation, similar to Go's `strings.Builder` or Java's `StringBuilder`. It avoids the O(n²) cost of repeated `s + "x"` concatenation.

### Class

```ky
final class str_builder:
    data: ptr

    str_builder(capacity: i64 = 16):
        data = ky_str_builder_new(capacity)
    
    fn append(s: &str):
        ky_str_builder_append(data, s as ptr, len(s))
    
    fn to_str() str:
        ky_str_builder_to_str(data)
    
    fn free():
        ky_str_builder_free(data)
```

### Usage

```ky
sb = str_builder(50000)
i: ^i32 = 0
while i < 50000:
    sb.append("x")
    i = i + 1
result = sb.to_str()
println(result)
```

### Performance

`str_builder.append()` reallocates with doubling strategy (2× capacity) when the buffer is full, achieving amortized O(1) append. Compared to `s + "x"` (which allocates + copies on every concat), `str_builder` is **~380× faster** for 50k concatenations.
