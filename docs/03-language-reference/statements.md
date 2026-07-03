# Statements

## Expression statement

```ky
x + 1
println("hello")
```

## Block

```ky
:
    statement1
    statement2
```

## If / elif / else

```ky
if x > 0:
    println("positive")
elif x < 0:
    println("negative")
else:
    println("zero")
```

## While

```ky
i: &i32 = 0
while i < 10:
    println(i)
    i = i + 1
```

## For

```ky
for i in 0..10:         # range
    println(i)

for i in my_list:       # list iteration
    println(i)

for i in 0..=5:         # inclusive range
    println(i)

for i in 0..10:
    println(i)
else:                   # runs if loop completes without break
    println("done")
```

## Match

```ky
match x:
    1:
        println("one")
    2 | 3:
        println("two or three")
    n if n > 10:
        println("big")
    _:
        println("other")
```

## Return

```ky
fn add(a: i32, b: i32) i32:
    return a + b       # explicit
    a + b              # implicit (last expression)
```

## Break / continue

```ky
while true:
    if done:
        break
    i = i + 1
    if skip:
        continue
```

## Defer

```ky
file = open("data.txt")
defer close(file)      # runs when scope exits
```

## Guard

```ky
guard value = optional else:
    return
```

## Unsafe

```ky
unsafe:
    ptr = as_ptr(variable)
    ptr[0] = 0xFF
```
