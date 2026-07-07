# std.core — Core Types

## Option

```ky
from std.core import Option

value: Option<i32> = none
value = 42

if val = value:
    println(val)       # 42

result = value.unwrapOr(0)
exists = value.isSome()
missing = value.isNone()
```

## Result

```ky
from std.core import Result

fn divide(a: i32, b: i32) Result<i32, str>:
    if b == 0:
        return error("division by zero")
    ok(a / b)
```
