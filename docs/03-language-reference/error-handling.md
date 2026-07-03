# Error Handling

## Fallible type: `T!`

`T!` is sugar for `Result<T, Error>`. Functions that can fail return `T!`.

```ky
fn divide(a: i32, b: i32) i32!:
    if b == 0:
        return error("division by zero")
    a / b
```

## Error propagation: `?`

The `?` operator propagates errors automatically.

```ky
fn process() str!:
    data = read_file("config.txt") ?
    parse_config(data) ?
```

If `read_file` returns an error, `process` returns that error immediately.

## Handling errors

```ky
result = divide(10, 0) !

if result:
    println("result: {result}")
else:
    println("error: {result.error()}")
```

## Custom errors

```ky
fn validate(age: i32) i32!:
    if age < 0:
        return error("age cannot be negative")
    if age > 150:
        return error("age seems unrealistic")
    age
```
