# Kyle Language Syntax — Complete Feature Checklist

> **Legend:**
> - `[x]` = Confirmed working (test exists)
> - `[?]` = Documented but untested / uncertain
> - `[ ]` = Not implemented

**Total: 181 confirmed, 175 uncertain, 12 missing**

---

## 1. Variables & Mutability (11 features)

[x] 1.1 Immutable variable (default): `name = "Kyle"`
[x] 1.2 Mutable variable (`^T`): `count: ^i32 = 0`
[x] 1.3 Type annotation: `x: i32 = 42`
[x] 1.4 Type inference: `y = 10`
[x] 1.5 Compound assignment: `count += 1`
[x] 1.6 Destructuring tuple: `(x, y) = pair`
[?] 1.7 Destructuring list
[?] 1.8 Constants: `NAME := value`
[x] 1.9 Block scope
[x] 1.10 No `let`/`var`/`mut`
[x] 1.11 Use `this` not `self`

## 2. Types — Primitives (32 features)

[x] 2.1-2.16 i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool, char, str, ptr, void, never
[?] 2.17-2.32 bytes, decimal, uuid, url, regex, big_int, str_builder, env, json (native), file, socket, path, date_time, duration, date, time

## 3. Types — Compound (37 features)

[x] 3.1 Fixed array `[T, N]`
[x] 3.4 Array indexing
[x] 3.7 Dynamic list `{T}`
[x] 3.8 List `.push(val)`
[x] 3.9 List `.pop()`
[x] 3.13 List `.len()`
[x] 3.14 List `.contains(val)`
[x] 3.21 Dict `{K: V}`
[x] 3.22 Dict `.set(key, val)`
[x] 3.23 Dict `.get(key)`
[x] 3.24 Dict `.has(key)`
[x] 3.25 Dict `.remove(key)`
[x] 3.29 Tuple `(T1, T2, ...)`
[?] all other collection features

## 4. Functions (16 features)

[x] 4.1 Basic function: `fn add(a: i32) i32:`
[x] 4.2 Implicit return (last expression)
[x] 4.3 Explicit `return`
[x] 4.4 Void function (no return type)
[x] 4.13 Move parameter
[x] 4.14 Borrow param `&T`
[x] 4.15 Mutable borrow param `^&T`
[x] 4.16 Method with `this`
[?] 4.5-4.12 Default params, multi-return, fn pointers, closures, static methods

## 5. Control Flow (20 features)

[x] 5.1 `if`
[x] 5.2 `elif`
[x] 5.3 `else`
[x] 5.6 `while`
[x] 5.8 `for` over range
[x] 5.10 `for` over list
[x] 5.14 `for`-`else`
[x] 5.15 `break`
[x] 5.16 `continue`
[x] 5.17 `return`
[x] 5.18 `defer`
[x] 5.19 `guard`
[?] 5.4, 5.5, 5.7, 5.9, 5.11-5.13, 5.20

## 6. Match / Pattern Matching (11 features)

[x] 6.1 `match` statement
[x] 6.3 Literal pattern
[x] 6.4 Identifier pattern
[x] 6.5 Wildcard `_`
[x] 6.6 Or-pattern `|`
[x] 6.7 Guard `if`
[x] 6.9 Enum variant pattern
[?] 6.2 match as expression, 6.8 range pattern, 6.10 is type, 6.11-6.12 optional

## 7. Classes & Inheritance (13 features)

[x] 7.1 `final class`
[x] 7.2 Struct literal
[x] 7.3 Method on class
[x] 7.4 Mutable field `^T`
[x] 7.5 Mutable field assignment
[x] 7.6 Default field value
[x] 7.7 Class inheritance `::`
[x] 7.8 Method override
[?] 7.9 abstract class, 7.10 constructor, 7.11 properties, 7.12-7.13 contracts

## 8. Enums (5 features)

[x] 8.1 Enum (no payload)
[x] 8.2 Enum with payload
[x] 8.3 Enum with multiple payloads
[x] 8.4 Enum value construction
[x] 8.5 Enum match

## 9. Error Handling (16 features)

[x] 9.1 Fallible return `T!`
[x] 9.2 `error(msg)` built-in
[x] 9.3 `ok(val)` built-in
[x] 9.4 `!` error propagation
[x] 9.6 `match` on `T!`
[x] 9.7 Optional type `T?`
[x] 9.8 `none` literal
[x] 9.10 Safety check on `T?`
[x] 9.11 Null-coalescing `??`
[x] 9.16 Panic on division by zero
[x] 9.18 Custom error messages
[?] 9.5, 9.9, 9.12-9.15, 9.17

## 10. Borrowing & Ownership (12 features)

[x] 10.1 Move by default
[x] 10.2 Copy semantics (primitives)
[x] 10.3 Explicit `.clone()`
[x] 10.4 Immutable borrow `&T`
[x] 10.5 Mutable borrow `^&T`
[x] 10.6 Call site `&x`
[x] 10.7 Call site `^&x`
[x] 10.8 No lifetime annotations
[x] 10.9 No `&` return
[x] 10.10 Use-after-move error
[x] 10.11 Auto-free on scope exit
[x] 10.12 LIFO deallocation

## 11. Strings & Interpolation (24 features)

[x] 11.1 String literal
[x] 11.3 String concatenation `+`
[x] 11.4 String interpolation
[x] 11.8 `.len()`
[x] 11.9 `.upper()`
[x] 11.10 `.lower()`
[x] 11.12 `.contains(sub)`
[x] 11.15 `.char_at(i)`
[x] 11.21 `.to_str()`
[?] 11.2, 11.5-11.7, 11.11, 11.13-11.14, 11.16-11.20, 11.22-11.25

## 12. Generics (7 features)

[x] 12.1 Generic class `<T>`
[x] 12.2 Generic method
[x] 12.3 Generic function
[x] 12.4 Multiple type params
[x] 12.5 Type inference on generic
[x] 12.7 Monomorphization
[?] 12.6 Generic constraint `: copy`

## 13. Operators & Overloading (28 features)

[x] 13.1 Arithmetic `+ - * / %`
[x] 13.3 Comparison `== != < > <= >=`
[x] 13.4 Logical `and or not`
[x] 13.6 Range `..`
[x] 13.8 Type cast `as`
[x] 13.9 Assignment `=`
[x] 13.10 Compound `+= -= *= /= %=`
[x] 13.11 Null-coalescing `??`
[x] 13.14 `op_add` overloading
[x] 13.15 `op_sub` overloading
[x] 13.20 `op_eq` overloading
[?] 13.2, 13.5, 13.7, 13.12-13.13, 13.16-13.19, 13.21-13.28

## 14. FFI (11 features)

[x] 14.1 `extern fn`
[x] 14.2 `@link` directive
[x] 14.6 `ptr` type
[x] 14.9 Type mapping C ↔ Kyle
[?] 14.3-14.5, 14.7-14.8, 14.10-14.11

## 15. Modules & Imports (11 features)

[x] 15.1 Named import `from X import Y`
[?] 15.2-15.11

## 16. Attributes & Metaprogramming (6 features)

[ ] 16.1-16.6: #[test], #[bench], macros: NOT IMPLEMENTED

## 17. Async/Await & Concurrency (21 features)

[ ] 17.1-17.21: NOT IMPLEMENTED (docs exist, no working tests)

## 18-25. Remaining: See full docs/03-language/

---

## How to Verify

```bash
cd /tmp/prueba/sintaxis
ky build src/main.ky    # debe compilar sin errores
ky run src/main.ky       # debe ejecutar sin crashes
```

Para features marcadas `[?]`, crear un archivo `.ky` de prueba y compilarlo.
