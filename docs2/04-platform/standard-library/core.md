# std.core — Core Types

## Option

```ky
from std.core import Option

value: Option<i32> = none
value = 42

if val = value:
    println(val)       # 42

result = value.unwrap_or(0)
exists = value.is_some()
missing = value.is_none()
```

## Result

```ky
from std.core import Result

fn divide(a: i32, b: i32) Result<i32, str>:
    if b == 0:
        return error("division by zero")
    ok(a / b)
```
