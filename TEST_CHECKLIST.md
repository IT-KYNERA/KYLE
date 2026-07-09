# Kyle Test Checklist

> Systematic verification of ALL documented syntax features.
> Each item must compile and produce correct output.
> Mark `[x]` when tested and working, `[b]` if bug found, `[-]` if not applicable.

---

## P0: Ownership v0.6

### Variables

| # | Test | Expected | Result |
|---|------|----------|--------|
| 1 | `x = 42` | Immutable i32, cannot reassign | [x] |
| 2 | `x: ^i32 = 0; x = x + 1` | Mutable i32, can reassign | [x] |
| 3 | `x: ^str = "hola"; x = "mundo"` | Mutable str, can reassign | [x] |

### Move by default

| # | Test | Expected | Result |
|---|------|----------|--------|
| 4 | `s = "hola"; t = s; println(s)` | Error: use-after-move | [x] |
| 5 | `x = 42; y = x; println(x)` | Works (Copy type, i32) | [x] |
| 6 | `s = "hola"; t = s.clone(); println(s)` | Works (explicit clone) | [x] |

### Borrow `&T`

| # | Test | Expected | Result |
|---|------|----------|--------|
| 7 | `fn read(s: &str): println(s)` | Function takes borrow | [x] |
| 8 | `read(&name); println(name)` | Caller retains ownership | [x] |
| 9 | `read(&name); read(&name)` | Multiple immutable borrows OK | [x] |

### Mutable borrow `^&T`

| # | Test | Expected | Result |
|---|------|----------|--------|
| 10 | `fn append(s: ^&str): s = s + "!"` | Function takes mut borrow | [x] |
| 11 | `append(^&buf); println(buf)` | Caller sees mutation | [x] |
| 12 | `read(&x); append(^&x)` | Error: immut + mut borrow | [x] |
| 13 | `append(^&x); read(&x)` | Error: mut + immut borrow | [x] |
| 14 | `append(^&x); append(^&x)` | Error: mut + mut borrow | [x] |

### Move in function params

| # | Test | Expected | Result |
|---|------|----------|--------|
| 15 | `fn consume(s: str): println(s)` | Move param (default) | [x] |
| 16 | `consume(s); println("done")` | OK (s moved, no further use) | [x] |
| 17 | `consume(s); println(s)` | Error: use-after-move | [x] |

---

## P1: Primitives

### Integer types

| # | Test | Expected | Result |
|---|------|----------|--------|
| 18 | `x: i8 = 127` | Signed 8-bit | [x] |
| 19 | `x: i16 = 32767` | Signed 16-bit | [x] |
| 20 | `x: i32 = 2147483647` | Signed 32-bit | [x] |
| 21 | `x: i64 = 9223372036854775807` | Signed 64-bit | [x] |
| 22 | `x: u8 = 255` | Unsigned 8-bit | [x] |
| 23 | `x: u16 = 65535` | Unsigned 16-bit | [x] |
| 24 | `x: u32 = 4294967295` | Unsigned 32-bit | [x] |
| 25 | `x: u64 = 18446744073709551615` | Unsigned 64-bit | [b] |

### Float types

| # | Test | Expected | Result |
|---|------|----------|--------|
| 26 | `x: f32 = 3.14` | 32-bit float | [x] |
| 27 | `x = 3.14` | Default f64 | [x] |

### Char

| # | Test | Expected | Result |
|---|------|----------|--------|
| 28 | `c = 'a'; println(c.to_str())` | Should print "a" | [x] |

### Bool

| # | Test | Expected | Result |
|---|------|----------|--------|
| 29 | `b = true; println(b.to_str())` | Prints "true" | [x] |
| 30 | `if true: println("ok")` | Executes block | [x] |

### Ptr

| # | Test | Expected | Result |
|---|------|----------|--------|
| 31 | `p = 0 as ptr` | Null pointer | [b] |
| 32 | `p = variable as ptr` | Address of variable | [b] |

---

## P2: Collections

### List `{T}`

| # | Test | Expected | Result |
|---|------|----------|--------|
| 33 | `v = {1, 2, 3}` | List literal | [x] |
| 34 | `v.push(4); println(v.len())` | 4 | [x] |
| 35 | `x = v[0]` | List get | [x] |
| 36 | `v[0] = 99` | List set | [x] |
| 37 | `x = v.pop()` | List pop | [x] |
| 38 | `v.reserve(100)` | Pre-allocate | [x] |

### Dict `{K: V}`

| # | Test | Expected | Result |
|---|------|----------|--------|
| 39 | `d = {"key": 42}` | Dict literal | [x] |
| 40 | `x = d["key"]` | Dict get | [x] |
| 41 | `d["key"] = 99` | Dict set | [x] |

### Array `[T; N]`

| # | Test | Expected | Result |
|---|------|----------|--------|
| 42 | `a = [1, 2, 3]` | Array literal `[i32; 3]` | [x] |
| 43 | `a[0]` | GEP + load | [x] |
| 44 | `a[0] = 99` | GEP + store | [x] |
| 45 | `a = [0; 100]` | Array repeat | [x] |

### Tuple

| # | Test | Expected | Result |
|---|------|----------|--------|
| 46 | `t = (1, "hello")` | Tuple literal | [x] |

---

## P3: Concurrency

| # | Test | Expected | Result |
|---|------|----------|--------|
| 47 | `async fn f(n: i64) i64: n * 2; task = f(21); await task` | Async function | [x] |
| 48 | `task = async: 42; await task` | Async block | [x] |
| 49 | `ky_parallel_for(fn, 0, 8)` | Parallel for | [x] |
| 50 | `h = ky_spawn_thread(fn, arg); r = ky_join_thread(h)` | Threads | [x] |

---

## P4: Classes

| # | Test | Expected | Result |
|---|------|----------|--------|
| 51 | `class Animal: ...` | Class declaration | [x] |
| 52 | `final class Point: ...` | Non-inheritable struct | [x] |
| 53 | `class Dog :: Animal:` | Simple inheritance | [x] |
| 54 | `contract Drawable: fn draw()` | Contract declaration | [x] |
| 55 | `class Circle :: Drawable: fn draw(): ...` | Implement contract | [x] |
| 56 | `enum Color: RED GREEN BLUE` | Enum with variants | [x] |
| 57 | `class Box<T>: ...` | Generic class | [x] |
| 58 | `fn identity<T>(x: T) T: x` | Generic function | [x] |
| 59 | `identity<i32>(42)` | Generic function call | [x] |

### Field access

| # | Test | Expected | Result |
|---|------|----------|--------|
| 60 | `this.x = nx` | Field access via this | [x] |
| 61 | `Point { x: 10, y: 20 }` | Struct literal | [x] |

---

## P5: Options / Results

| # | Test | Expected | Result |
|---|------|----------|--------|
| 62 | `name: str? = none` | Optional declaration | [x] |
| 63 | `match name: none: ... some(v): ...` | Pattern match option | [x] |
| 64 | `fn div(a,b) i32!:` | Fallible return | [x] |
| 65 | `return error("msg")` | Error return | [x] |
| 66 | `ok(val)` | Success return | [x] |
| 67 | `match res: ok(v): ... error(e): ...` | Pattern match result | [ ] |

---

## P6: Borrow Checker

| # | Test | Expected | Result |
|---|------|----------|--------|
| 68 | `y = x; println(x)` for str | Error: use-after-move | [x] |
| 69 | `f(x); println(x)` for str | Error: use-after-move via fn | [x] |
| 70 | `append(^&x); read(&x)` | Error: mut + immut borrow | [x] |
| 71 | `read(&x); read(&x)` | OK: multiple immut borrows | [x] |
| 72 | `f(&x); consume(x)` | OK: borrow then move | [x] |

---

## P7: String Operations

| # | Test | Expected | Result |
|---|------|----------|--------|
| 73 | `s.trim()` | Trim whitespace | [x] |
| 74 | `s.to_upper()` | Uppercase | [x] |
| 75 | `s.to_lower()` | Lowercase | [x] |
| 76 | `s.contains(sub)` | Contains substring | [x] |
| 77 | `s.replace(a, b)` | Replace substring | [x] |
| 78 | `s.substr(start, len)` | Substring | [x] |
| 79 | `len(s)` | String length | [x] |

---

## P8: str_builder

| # | Test | Expected | Result |
|---|------|----------|--------|
| 80 | `sb = str_builder(100)` | Create builder | [x] |
| 81 | `sb.append("x")` | Append string | [x] |
| 82 | `sb.to_str()` | Extract string | [x] |
| 83 | `sb.free()` | Free memory | [x] |

---

## P9: Functions

| # | Test | Expected | Result |
|---|------|----------|--------|
| 84 | `fn add(a, b) i32: a + b` | Function declaration | [x] |
| 85 | `fn greet(name: str = "world")` | Default parameter | [x] |
| 86 | `fn_ptr = add as ptr; fn_ptr(1, 2)` | Function pointer | [x] |
| 87 | `(x: i32): x * 2` | Closure | [x] |

---

## P10: Statements

| # | Test | Expected | Result |
|---|------|----------|--------|
| 88 | `if x > 0: println("pos") elif x == 0: ... else: ...` | If/elif/else | [x] |
| 89 | `while i < 10: i = i + 1` | While loop | [x] |
| 90 | `for i in 0..10: println(i)` | For-in-range | [x] |
| 91 | `for val in list: println(val)` | For-in-list | [x] |
| 92 | `match x: 1: println("one") _: println("other")` | Match statement | [x] |
| 93 | `return value` | Return | [x] |
| 94 | `defer: close(f)` | Defer | [x] |
| 95 | `guard cond else: return` | Guard | [x] |

---

## Summary

| Priority | Tests | Pass | Fail | Bug |
|----------|:-----:|:----:|:----:|:---:|
| P0 Ownership | 17 | 0 | 0 | 0 |
| P1 Primitives | 15 | 0 | 0 | 1 |
| P2 Collections | 14 | 0 | 0 | 1 |
| P3 Concurrency | 4 | 0 | 0 | 0 |
| P4 Classes | 11 | 0 | 0 | 0 |
| P5 Options | 6 | 0 | 0 | 0 |
| P6 Borrow Checker | 5 | 0 | 0 | 0 |
| P7 Strings | 7 | 0 | 0 | 0 |
| P8 str_builder | 4 | 0 | 0 | 0 |
| P9 Functions | 4 | 0 | 0 | 0 |
| P10 Statements | 8 | 0 | 0 | 0 |
| **Total** | **95** | **0** | **0** | **2** |
