# Testing

Kyle has a built-in test framework using the `#[test]` attribute.

## Writing tests

```ky
#[test]
fn testAddition():
    result = 2 + 2
    assert(result == 4)

#[test]
fn testString():
    assert_eq("hello", "hello")
    assert_ne("hello", "world")
```

## Assertions

| Function | Description |
|----------|-------------|
| `assert(condition)` | Assert condition is true |
| `assert_eq(a, b)` | Assert a == b |
| `assert_ne(a, b)` | Assert a != b |
| `assert_str(a, b)` | Assert string equality |

## Running tests

```bash
ky test
```

This finds all `#[test]` functions in `tests/` directory, compiles and runs each one.

## Test project structure

```
my-project/
├── ky.toml
├── src/
│   └── main.ky
└── tests/
    ├── testUnit.ky
    └── testIntegration.ky
```

Each test function must:
- Take no parameters
- Return `i32` (0 for pass, non-zero for fail)
- Use `assert` builtins for validation
