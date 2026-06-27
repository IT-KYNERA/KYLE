# Kyle Language Reference

> Complete syntax, semantics, and formal grammar. Every construct is marked with
> its implementation status (✅ working, 🔶 partial, ❌ planned).

---

## Legend

| Mark | Meaning |
|------|---------|
| ✅ | Implemented and generates working code |
| 🔶 | Parsed/type-checked but no codegen (or partial) |
| ❌ | Not implemented (planned for future phase) |

---

## 1. Program Structure

### Entry Point ✅

```kl
fn main(args: [str]) -> i32:
    println("Hello, Kyle!")
    return 0
```

The entry point is `fn main` in `src/main.kl`. The `args` parameter receives
command-line arguments. Return type `i32` is the exit code.

### Comments ✅

```kl
# Line comment

#[#
  Block comment
  (multi-line)
#]
```

---

## 2. Variables & Mutability

### Immutable by Default ✅

```kl
x = 42             # immutable
name = "Anna"      # immutable
```

### Mutable ✅

```kl
mut count = 0
count = count + 1
```

### Constants ✅

```kl
PI = 3.14159       # UPPERCASE = compile-time constant
MAX_RETRIES = 3
```

Constants are always immutable. No `mut` keyword on constants.

### Explicit Types ✅

```kl
mut count: i32 = 0
items: [str] = ["a", "b"]
ratio: f64 = 3.14
```

### Type Inference ✅

```kl
x = 42           # i32
y = 3.14         # f64
s = "hello"      # str
b = true         # bool
nums = [1, 2, 3] # [i32]
```

### Auto-Declare ✅

```kl
result = some_function()    # type inferred from return type
```

Variable type is inferred from the expression and registered in scope.

---

## 3. Primitive Types ✅

| Type | Size | Example |
|------|------|---------|
| `i32` | 4 bytes | `42` |
| `i64` | 8 bytes | `9999999999` |
| `f32` | 4 bytes | `3.14` |
| `f64` | 8 bytes | `3.141592653589793` |
| `bool` | 1 byte | `true`, `false` |
| `str` | ptr+8 | `"hello"` |
| `char` | 1 byte | `'A'` |
| `void` | 0 | unit type |

### Integer Literals ✅

```kl
42              # decimal
0xFF            # hexadecimal
0b1010          # binary
1_000_000       # underscores for readability
```

### Float Literals ✅

```kl
3.14
1.0
0.001
```

### String Literals ✅

```kl
"hello"
"with\nnewlines\tand\ttabs"
"escaped \"quotes\""
"unicode: \u0041"
```

Escape sequences: `\n`, `\t`, `\r`, `\0`, `\"`, `\'`, `\\`, `\xHH`, `\uXXXX`.

### Char Literals ✅

```kl
'A'
'9'
'\n'
```

### Boolean Literals ✅

```kl
true
false
```

---

## 4. Operators

### Arithmetic ✅

```kl
a + b          # addition (or string concat)
a - b          # subtraction
a * b          # multiplication
a / b          # division
a % b          # modulo
```

### Comparison ✅

```kl
a == b         # equal
a != b         # not equal
a < b          # less than
a > b          # greater than
a <= b         # less or equal
a >= b         # greater or equal
```

### Logical ✅

```kl
a and b        # logical and
a or b         # logical or
not a          # logical not
```

### Assignment ✅

```kl
x = 42
mut y = 0
y = y + 1
```

### Bitwise 🔶

```kl
a & b          # bitwise and (parsed, needs verification)
a | b          # bitwise or
a ^ b          # bitwise xor
a << b         # shift left
a >> b         # shift right
```

### Range ✅

```kl
0..10          # range from 0 to 9 (exclusive end)
list[0..3]     # slice from index 0 to 2
```

### Spread ✅

```kl
[...a, 4, 5]   # spread list a, then 4, 5
```

### Ternary ✅

```kl
result = cond ? "yes" : "no"
```

### Optional Chaining ✅

```kl
name = user?.name       # None if user is None
age = user?.age ?: 0    # default 0 if None
```

### Error Propagation ✅

```kl
val = might_fail()?     # propagate error to caller
```

---

## 5. Functions

### Declaration ✅

```kl
fn add(a: i32, b: i32) -> i32:
    return a + b
```

### Type Inference in Body ✅

```kl
fn compute():
    x = 42         # type inferred as i32
    result = x * 2 # type inferred as i32
```

### Generic Functions ✅

```kl
fn first<T>(items: [T]) -> Option<T>:
    if len(items) > 0:
        return Some(items[0])
    return None
```

Generics are monomorphized at compile time.

### Error Return Type ✅

```kl
fn parse_int(s: str) -> i32!:
    if is_digit(s):
        return 42
    return error("not a number")
```

### Async Functions ✅

```kl
fn fetch_data() -> async i32:
    return 42

# Usage:
task = fetch_data()
result = await task
```

### Const Functions ✅ (type-check only, compile-time eval ❌ Phase 10)

```kl
const fn factorial(n: i32) -> i32:
    if n <= 1:
        return 1
    return n * factorial(n - 1)
```

### Contracts ✅

```kl
contract Greeter:
    fn greet() -> str

class Person(name: str) : Greeter:
    fn greet() -> str:
        return "Hello, " + name
```

Contracts with generic constraints (`T: Comparable`) — ❌ Phase 10.

---

## 6. Control Flow

### If / Elif / Else ✅

```kl
if x > 0:
    println("positive")
elif x < 0:
    println("negative")
else:
    println("zero")
```

### While ✅

```kl
mut i = 0
while i < 10:
    println(i)
    i = i + 1
```

### While-Else ✅

```kl
while condition():
    if found():
        break
else:
    println("loop ended without break")
```

### For (List) ✅

```kl
for item in items:
    println(item)
```

### For (Range) ✅

```kl
for i in 0..10:
    println(i)
```

### For-Else ✅

```kl
for item in items:
    if item == target:
        println("found")
        break
else:
    println("not found")
```

### Loop ✅

```kl
loop:
    if done():
        break
    do_work()
```

### Break ✅

```kl
for item in items:
    if item == target:
        break
```

### Continue ✅

```kl
for item in items:
    if item == skip:
        continue
    process(item)
```

### Match ✅

```kl
match value:
    0 => println("zero")
    1 | 2 => println("one or two")
    n if n > 10 => println("big")
    _ => println("other")
```

### Match-Expression ✅

```kl
result = match x:
    0 => "zero"
    1 => "one"
    _ => "many"
```

### Defer ✅

```kl
fn process_file(path: str):
    file = open(path)
    defer close(file)
    # ... use file ...
    # close(file) called automatically on return (LIFO order)
```

### Guard ✅

```kl
fn process(data: [i32]):
    guard len(data) > 0:
        return    # early return if guard fails
    # ... rest of function ...
```

### Unsafe ✅ (parsed, FFI lowering ❌ Phase 9)

```kl
unsafe:
    # operations that bypass safety checks
    # (FFI calls will go here in Phase 9)
```

---

## 7. Data Structures

### Struct ✅

```kl
struct Point:
    x: i32
    y: i32

p = Point { x: 10, y: 20 }
println(p.x)
```

Structs are pass-by-reference (pointer ABI). Passed to functions as pointers.

### Generic Struct ✅

```kl
struct Pair<A, B>:
    first: A
    second: B

p1 = Pair { first: 1, second: 2 }
p2 = Pair { first: "a", second: "b" }
```

### Enum ✅

```kl
enum Option:
    Some(i32)
    None

match option:
    Some(v) => println(v)
    None => println("nothing")
```

Enums are tagged unions: `{disc: i32, payload: i64}`. Payload binds in match.

### Class ✅

```kl
class Counter(start: i32):
    count: i32

    Counter(start: i32):
        this.count = start

    fn increment() -> i32:
        this.count = this.count + 1
        return this.count

c = Counter(10)
println(c.increment())   # 11
```

Classes have constructors, methods, and use `this` for instance reference.

### Inheritance ✅

```kl
class Animal(name: str):
    fn speak() -> str:
        return "..."

class Dog(name: str) : Animal:
    fn speak() -> str:
        return "Woof!"
```

### Contract ✅

```kl
contract Serializable:
    fn serialize() -> str

class User(name: str, age: i32) : Serializable:
    fn serialize() -> str:
        return name + "," + str(age)
```

### Type Alias ✅

```kl
type UserId = i32
type UserMap = {str: UserId}
```

---

## 8. Collections

### Lists ✅

```kl
nums = [1, 2, 3]
nums.add(4)          # append
first = nums[0]      # index
len_s = len(nums)    # length
popped = nums.pop()  # remove last, returns it
sliced = nums[0..2]  # range slice → [1, 2]
```

### List of Strings ✅

```kl
names = ["Alice", "Bob"]
```

### Dict / Map ✅

```kl
ages = {"Alice": 30, "Bob": 25}
ages["Charlie"] = 35
println(ages["Alice"])   # 30
println(ages.len())      # 3 (via method)
```

### Spread in List ✅

```kl
a = [1, 2, 3]
b = [...a, 4, 5]        # [1, 2, 3, 4, 5]
```

---

## 9. Closures ✅

```kl
double = (x: i32) => x * 2
result = double(21)     # 42

add = (a: i32, b: i32) => a + b
result = add(100, 1)   # 101
```

Closures create unique `_closure_N` functions and use `FnAddr` + `CallIndirect`.

---

## 10. Async / Await ✅

```kl
task = async 42
result = await task    # 42
```

Thread-based: `async` spawns an OS thread (`kl_spawn_thread`), `await` joins it
(`kl_join_thread`).

---

## 11. Error Handling

### Error Type ✅

```kl
fn might_fail(x: i32) -> i32!:
    if x < 0:
        return error("negative input")
    return x
```

### Error Propagation ✅

```kl
fn process() -> i32!:
    val = might_fail(-1)?
    return val
```

### Optional Type ✅

```kl
fn find(id: i32) -> Option<str>:
    if id == 1:
        return Some("found")
    return None

match find(1):
    Some(msg) => println(msg)
    None => println("not found")
```

### Optional Chaining ✅

```kl
name = user?.address?.street   # chains safely through None
default = user?.name ?: "anonymous"
```

---

## 12. Imports ✅

```kl
import io
import math from "std/math"
from io import println
from io import println as pln
```

---

## 13. String Operations ✅

```kl
s = "Hello, World!"

len(s)               # 13
contains(s, "World") # true
to_upper(s)          # "HELLO, WORLD!"
to_lower(s)          # "hello, world!"
trim("  hi  ")       # "hi"
replace(s, "World", "Kyle")  # "Hello, Kyle!"
substr(s, 0, 5)      # "Hello"
char_at(s, 0)        # 'H'
ord('A')             # 65
is_digit('9')        # true
is_alpha('A')        # true
is_alnum('A')        # true
```

### String Concatenation ✅

```kl
greeting = "Hello, " + name + "!"
```

### String Conversion ✅

```kl
s = str(42)          # "42"
i = len("hello")     # 5
```

---

## 14. Built-in Functions ✅

| Function | Signature | Description |
|----------|-----------|-------------|
| `print(x)` | `print(str)` | Print without newline |
| `println(x)` | `println(str)` | Print with newline |
| `len(x)` | `len(str_or_list) -> i32` | Length |
| `str(x)` | `str(i32_or_i64) -> str` | Convert to string |
| `range(n)` | `range(i32) -> [i32]` | Create range list [0, n-1] |
| `open(path)` | `open(str) -> file` | Open file for reading |
| `read_str(f)` | `read_str(file) -> str` | Read file content |
| `write_str(f, s)` | `write_str(file, str)` | Write string to file |
| `close(f)` | `close(file)` | Close file |
| `sleep(ms)` | `sleep(i32)` | Sleep milliseconds |
| `now()` | `now() -> i64` | Current timestamp |
| `input()` | `input() -> str` | Read from stdin |
| `ord(c)` | `ord(char) -> i32` | Character to ASCII code |
| `char_at(s, i)` | `char_at(str, i32) -> char` | Character at index |
| `is_digit(c)` | `is_digit(char) -> bool` | Check if digit |
| `is_alpha(c)` | `is_alpha(char) -> bool` | Check if alphabetic |
| `is_alnum(c)` | `is_alnum(char) -> bool` | Check if alphanumeric |
| `is_whitespace(c)` | `is_whitespace(char) -> bool` | Check if whitespace |
| `is_upper(c)` | `is_upper(char) -> bool` | Check if uppercase |
| `is_lower(c)` | `is_lower(char) -> bool` | Check if lowercase |
| `json_parse(s)` | `json_parse(str) -> value` | Parse JSON string |
| `json_stringify(v)` | `json_stringify(value) -> str` | Serialize to JSON |

---

## 15. Formatted Grammar (EBNF)

```
program         = { declaration } .

declaration     = function_decl
                | struct_decl
                | class_decl
                | enum_decl
                | contract_decl
                | type_alias
                | import_decl
                | const_decl .

function_decl   = [ "const" ] "fn" identifier
                  [ generic_params ] "(" [ params ] ")"
                  [ return_type ] ":" block .

return_type     = "->" type .

generic_params  = "<" type_param { "," type_param } ">" .

type_param      = identifier .

params          = param { "," param } .

param           = identifier [ ":" type ] .

struct_decl     = "struct" identifier [ generic_params ] ":" NEWLINE
                  INDENT { field_decl } DEDENT .

field_decl      = identifier ":" type NEWLINE .

class_decl      = "class" identifier "(" [ class_params ] ")"
                  [ ":" inherited ] ":" NEWLINE
                  INDENT { class_member } DEDENT .

class_params    = param { "," param } .

inherited       = identifier { "," identifier } .

class_member    = field_decl | constructor | method_decl .

constructor    = identifier "(" [ params ] ")" ":" block .

method_decl     = [ "override" ] "fn" identifier "(" [ params ] ")"
                  [ return_type ] ":" block .

enum_decl       = "enum" identifier ":" NEWLINE
                  INDENT { enum_variant } DEDENT .

enum_variant    = identifier [ "(" type ")" ] NEWLINE .

contract_decl   = "contract" identifier ":" NEWLINE
                  INDENT { contract_method } DEDENT .

contract_method = "fn" identifier "(" [ params ] ")" [ return_type ] NEWLINE .

type_alias      = "type" identifier "=" type .

import_decl     = "import" identifier
                | "import" identifier "from" string
                | "from" identifier "import" identifier
                | "from" identifier "import" identifier "as" identifier .

const_decl      = identifier "=" expression .

type            = primitive_type
                | "Option" "<" type ">"
                | type "!"
                | "[" type "]"           (* list *)
                | "{" type ":" type "}"  (* dict *)
                | identifier             (* user type *)
                | identifier "<" type { "," type } ">"  (* generic *)
                .

primitive_type  = "i32" | "i64" | "f32" | "f64"
                | "bool" | "str" | "char" | "void" .

block           = NEWLINE INDENT { statement } DEDENT .

statement       = simple_stmt | compound_stmt .

simple_stmt     = assignment
                | expression_stmt
                | return_stmt
                | break_stmt
                | continue_stmt
                | defer_stmt
                .

assignment      = target "=" expression .

return_stmt     = "return" [ expression ] .

break_stmt      = "break" .
continue_stmt   = "continue" .

defer_stmt      = "defer" expression .

compound_stmt   = if_stmt
                | while_stmt
                | for_stmt
                | loop_stmt
                | match_stmt
                | guard_stmt
                | unsafe_block .

if_stmt         = "if" expression ":" block
                  { "elif" expression ":" block }
                  [ "else" ":" block ] .

while_stmt      = "while" expression ":" block
                  [ "else" ":" block ] .

for_stmt        = "for" identifier "in" expression ":" block
                  [ "else" ":" block ] .

loop_stmt       = "loop" ":" block .

match_stmt      = "match" expression ":" block
                  ( where block contains match arms ) .

guard_stmt      = "guard" expression ":" block .

unsafe_block    = "unsafe" ":" block .

expression      = ternary_expr .

ternary_expr    = or_expr [ "?" expression ":" expression ] .

or_expr         = and_expr { "or" and_expr } .
and_expr        = not_expr { "and" not_expr } .
not_expr        = "not" not_expr | comparison .
comparison     = additive { comp_op additive } .
comp_op         = "==" | "!=" | "<" | ">" | "<=" | ">=" .
additive        = multiplicative { ("+" | "-") multiplicative } .
multiplicative  = unary { ("*" | "/" | "%") unary } .
unary           = ("-" | "!") unary | postfix .
postfix         = primary { postfix_op } .

postfix_op      = "." identifier              (* property access *)
                | "(" [ args ] ")"            (* function call *)
                | "[" expression "]"           (* index *)
                | "[" range_expr "]"           (* slice *)
                | "?"                          (* error propagation *)
                | "?."                         (* optional chain *)
                | "?:" expression              (* default *)
                .

range_expr      = [ expression ] ".." [ expression ] .

primary         = literal
                | identifier
                | "(" expression ")"           (* grouping *)
                | "(" closure ")"              (* closure *)
                | "[" list_elements "]"        (* list literal *)
                | "{" dict_elements "}"        (* dict literal *)
                | "async" expression           (* async spawn *)
                | "await" expression          (* await task *)
                | match_expr                   (* match as expression *)
                | struct_literal
                | "..." expression             (* spread *) .

closure         = [ "(" [ params ] ")" ] "=>" expression .

struct_literal  = identifier "{" field_inits "}" .

field_inits     = field_init { "," field_init } .
field_init      = identifier ":" expression .

match_expr      = "match" expression ":" block
                  ( used in expression context, produces a value ) .

literal         = integer_literal
                | float_literal
                | string_literal
                | char_literal
                | boolean_literal
                | "None" .

boolean_literal = "true" | "false" .

integer_literal = digit { digit | "_" }
                | "0x" hex_digit { hex_digit }
                | "0b" bit { bit } .

float_literal   = digit { digit } "." { digit } .

string_literal  = '"' { char | escape } '"' .

escape          = "\" ( "n" | "t" | "r" | "0" | "\"" | "'" | "\\" | "x" hex hex | "u" hex hex hex hex ) .

char_literal    = "'" ( char | escape ) "'" .

identifier     = alpha { alpha | digit | "_" } .

alpha           = "a".."z" | "A".."Z" | "_" .
digit           = "0".."9" .
hex_digit       = digit | "a".."f" | "A".."F" .
bit             = "0" | "1" .
```

---

## 16. Status Summary

| Category | Status |
|----------|--------|
| Variables & mutability | ✅ Complete |
| Primitive types | ✅ Complete |
| Functions (incl. generics, errors, async, const) | ✅ Complete |
| Control flow (if/while/for/loop/match/defer/guard) | ✅ Complete |
| Structs (incl. generics) | ✅ Complete |
| Enums (tagged unions) | ✅ Complete |
| Classes (incl. inheritance) | ✅ Complete |
| Contracts | ✅ Complete (constraints ❌ Phase 10) |
| Closures | ✅ Complete |
| Async/await | ✅ Complete (thread-based) |
| Error handling (! / ?) | ✅ Complete |
| Optional types (Option<T>, ?.) | ✅ Complete |
| Dict/Map | ✅ Complete |
| List operations | ✅ Complete |
| Spread operator | ✅ Complete (list only) |
| Range slicing | ✅ Complete |
| Ternary operator | ✅ Complete |
| Match-expression | ✅ Complete |
| Type aliases | ✅ Complete |
| Imports | ✅ Complete |
| String operations | ✅ Complete |
| Unsafe blocks | ✅ Parsed (FFI lowering ❌ Phase 9) |
| FFI (extern "C") | ❌ Phase 9 |
| Iterators (.iter()) | ❌ Phase 10 |
| Functional ops (map/filter/reduce) | ❌ Phase 10 |
| Advanced collections (HashMap/HashSet) | ❌ Phase 10 |
| Const evaluation (compile-time) | ❌ Phase 10 |
| Operator overloading | ❌ Future |
| Block comments (#[# ... #]) | ✅ Lexed |

---

## Version

```
Kyle Language Reference v1.0
Last updated: 2026-06-26
```