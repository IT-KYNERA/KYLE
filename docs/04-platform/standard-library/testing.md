# std.testing — Test Assertions

| Function | Description |
|----------|-------------|
| `assert.is_true(condition)` | Assert condition is true |
| `assert.eq(a, b)` | Assert a == b |
| `assert.ne(a, b)` | Assert a != b |
| `assert.str_eq(a, b)` | Assert string equality |

## Usage

```ky
from std.testing import assert

fn test_addition():
    result = 2 + 2
    assert.eq(result, 4)
```
