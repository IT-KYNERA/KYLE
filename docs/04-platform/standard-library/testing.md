# std.testing — Test Assertions

| Function | Description |
|----------|-------------|
| `assert(condition)` | Assert condition is true |
| `assert_eq(a, b)` | Assert a == b |
| `assert_ne(a, b)` | Assert a != b |
| `assert_str(a, b)` | Assert string equality |

## Usage

```ky
from std.testing import assert, assert_eq

fn testAddition():
    result = 2 + 2
    assert_eq(result, 4)
```
