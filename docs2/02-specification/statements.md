# Statements

## Expression Statement

```ky
x + 1               # expression used as statement
println("hello")    # call as statement
```

## Variable Declaration

```ky
name = "Ana"         # immutable
age: &i32 = 25       # mutable
X := 10              # compile-time constant
```

## Block

```ky
:
    statement1
    statement2
```

## If / Elif / Else

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
for i in 0..10:             # range
    println(i)

for i in my_list:           # list iteration
    println(i)

for i in 0..=5:             # inclusive range
    println(i)

for i in 0..<5:             # exclusive (alias)
    println(i)

for i in 0..10:             # with else clause
    println(i)
else:
    println("completed")
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
    return a + b       # explicit return
    a + b              # implicit return (last expression)
```

## Break / Continue

```ky
while i < 10:
    if i == 5:
        break          # exit loop
    i = i + 1

while i < 10:
    i = i + 1
    if i == 3:
        continue       # skip to next iteration
    println(i)
```

## Defer

```ky
file = open("data.txt")
defer close(file)      # runs when scope exits
# ... use file ...
```

## Guard

```ky
fn process(x: i32?):
    guard value = x else:
        return "error"
    # value is non-none here
```

## Unsafe

```ky
result = unsafe:
    # raw pointer operations
    ptr[0] = 0xFF
```
