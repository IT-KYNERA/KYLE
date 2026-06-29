# Standard Library Reference

> Built-in modules that ship with every Kyle installation. No package manager
> required — just `import module` and use.

---

## 1. `core`

Core language primitives always available in every program.

| Name | Signature | Description |
| :--- | :--- | :--- |
| `T?` | type sugar | Optional type (compiles to `Option<T>` internally) |
| `Some` | `(T) T?` | Wrap a value in an optional (automatic via assignment) |
| `None` | `None` | The absent value (uninhabited type, coerced to any `T?`) |
| `unwrap_or` | `(T?, T) T` | Unwrap optional with default |
| `is_some` | `(T?) bool` | True if optional contains a value |
| `is_none` | `(T?) bool` | True if optional is absent |

```kl
name: str? = None
display = unwrap_or(name, "anonymous")
if is_some(name):
    println("got {unwrap_or(name, "")}")
```

---

## 2. `math`

Mathematical operations beyond basic arithmetic.

| Name | Signature | Description |
| :--- | :--- | :--- |
| `absolute` | `(i32) i32` | Absolute value (i32) |
| `pow` | `(i32, i32) i32` | Integer exponentiation |
| `sqrt` | `(i32) i32` | Integer square root (floor) |
| `gcd` | `(i32, i32) i32` | Greatest common divisor |
| `min` | `(i32, i32) i32` | Minimum of two values |
| `max` | `(i32, i32) i32` | Maximum of two values |
| `clamp` | `(i32, i32, i32) i32` | Clamp value between min and max |

```kl
import math
x = math.absolute(-5)       # 5
g = math.gcd(12, 18)        # 6
c = math.clamp(42, 0, 10)   # 10
```

---

## 3. `io`

File I/O operations.

| Name | Signature | Description |
| :--- | :--- | :--- |
| `read_file` | `(str) str!` | Read entire file as string |
| `write_file` | `(str, str) i32!` | Write string to file (overwrites) |

```kl
import io
content = io.read_file("data.txt")?
io.write_file("out.txt", "Hello, Kyle!")?
```

---

## 4. `str`

String utility functions (standalone versions of string methods).

| Name | Signature | Description |
| :--- | :--- | :--- |
| `starts_with_str` | `(str, str) bool` | Check if string starts with prefix |
| `ends_with_str` | `(str, str) bool` | Check if string ends with suffix |
| `capitalize` | `(str) str` | First character uppercase, rest lowercase |
| `repeat_str` | `(str, i32) str` | Repeat string n times |

```kl
import str
s = "hello"
b = str.starts_with_str(s, "he")   # true
c = str.capitalize(s)               # "Hello"
r = str.repeat_str("ha", 3)         # "hahaha"
```

---

## 5. `testing`

Assertion functions for tests and validation.

| Name | Signature | Description |
| :--- | :--- | :--- |
| `assert` | `(bool) void` | Panic if condition is false |
| `assert_eq` | `(any, any) void` | Panic if values differ |
| `assert_ne` | `(any, any) void` | Panic if values are equal |
| `assert_str` | `(str, str) void` | Panic if strings differ (detailed diff) |

```kl
import testing
testing.assert(1 + 1 == 2)
testing.assert_eq(2 + 2, 4)
testing.assert_str("hello", "hello")
```

---

## 6. `collections`

List utility functions.

| Name | Signature | Description |
| :--- | :--- | :--- |
| `list_sum` | `([i32]) i32` | Sum of all elements |
| `list_product` | `([i32]) i32` | Product of all elements |
| `list_max` | `([i32]) i32` | Maximum element |
| `list_min` | `([i32]) i32` | Minimum element |
| `list_range` | `(i32, i32) [i32]` | Create range list `[start, end)` |

```kl
import collections
nums = [1, 2, 3, 4, 5]
s = collections.list_sum(nums)         # 15
r = collections.list_range(0, 10)      # [0, 1, ..., 9]
```

---

## 7. `json`

JSON serialization and deserialization.

| Name | Signature | Description |
| :--- | :--- | :--- |
| `parse` | `(str) dict<str, i64>` | Parse JSON string to dict (objects only) |
| `stringify` | `(dict<str, i64>) str` | Serialize dict to JSON string |

```kl
import json
data = json.parse('{"a": 1, "b": 2}')
json.stringifyn = json.stringify(data)    # '{"a": 1, "b": 2}'
```

---

## 8. `time`

Time and sleep utilities.

| Name | Signature | Description |
| :--- | :--- | :--- |
| `timestamp` | `() i32` | Current Unix timestamp (seconds) |
| `sleep_ms` | `(i32) void` | Sleep for milliseconds |
| `seconds_since` | `(i32) i32` | Seconds elapsed since given timestamp |

```kl
import time
start = time.timestamp()
time.sleep_ms(1000)
elapsed = time.seconds_since(start)    # 1
```
