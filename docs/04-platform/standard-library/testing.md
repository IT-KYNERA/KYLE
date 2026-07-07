# std.testing — Test Assertions

| Function | Description |
|----------|-------------|
| `assert(condition)` | Assert condition is true |
| `assertEq(a, b)` | Assert a == b |
| `assertNe(a, b)` | Assert a != b |
| `assertStr(a, b)` | Assert string equality |

## Usage

```ky
from std.testing import assert, assertEq

fn testAddition():
    result = 2 + 2
    assertEq(result, 4)
```
