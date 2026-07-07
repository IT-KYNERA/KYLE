# std.str — String Utilities

| Function | Description |
|----------|-------------|
| `starts_with(s, prefix)` | Check if string starts with prefix |
| `ends_with(s, suffix)` | Check if string ends with suffix |
| `capitalize(s)` | Capitalize first letter |
| `repeat_str(s, count)` | Repeat string n times |

## strBuilder — Efficient String Building

`strBuilder` is a growable string buffer for efficient concatenation, similar to Go's `strings.Builder` or Java's `StringBuilder`. It avoids the O(n²) cost of repeated `s + "x"` concatenation.

### Functions (extern fn)

| Function | Description |
|----------|-------------|
| `ky_str_builder_new(capacity: i64) ptr` | Create builder with initial capacity |
| `ky_str_builder_append(builder: ptr, data: ptr, len: i64)` | Append `len` bytes from `data` |
| `ky_str_builder_to_str(builder: ptr) ptr` | Extract heap-allocated string (caller frees with `ky_free`) |
| `ky_str_builder_free(builder: ptr)` | Free builder and its buffer |

### Usage

```ky
extern fn ky_str_builder_new(capacity: i64) ptr
extern fn ky_str_builder_append(builder: ptr, data: ptr, len: i64)
extern fn ky_str_builder_to_str(builder: ptr) ptr
extern fn ky_str_builder_free(builder: ptr)
extern fn ky_strlen(s: ptr) i32

fn main() i32:
    sb := ky_str_builder_new(50000)
    i: ^i32 = 0
    while i < 50000:
        ky_str_builder_append(sb, "x", 1)
        i = i + 1
    result := ky_str_builder_to_str(sb)
    println(result)
    ky_str_builder_free(sb)
    0
```

### Performance

`ky_str_builder_append` reallocates with doubling strategy (2× capacity) when the buffer is full, achieving amortized O(1) append. Compared to `s + "x"` (which allocates + copies on every concat), strBuilder is **~380× faster** for 50k concatenations.
