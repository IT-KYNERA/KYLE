# Kyle Language — Reference

## Keywords

| Keyword | Context | Description |
|---------|---------|-------------|
| `fn` | declaration | Define function |
| `final class` | declaration | Lightweight struct (no inheritance) |
| `class` | declaration | Class with inheritance |
| `abstract class` | declaration | Cannot be instantiated |
| `enum` | declaration | Tagged union |
| `contract` | declaration | Interface (trait) |
| `struct` | declaration | Temporary alias for `final class` |
| `type` | declaration | Type alias |
| `if` / `elif` / `else` | statement | Conditional |
| `while` | statement | Loop |
| `for` | statement | Iteration |
| `match` | statement | Pattern matching |
| `return` | statement | Return value |
| `break` | statement | Exit loop |
| `continue` | statement | Skip to next iteration |
| `defer` | statement | Deferred execution |
| `guard` | statement | Early exit on condition |
| `unsafe` | statement | Unsafe block |
| `async` / `await` | expression | Async operations |
| `const` | modifier | Compile-time evaluation |
| `static` | modifier | Static method |
| `true` / `false` / `none` | literal | Boolean and null |
| `and` / `or` / `not` | operator | Logical operators |
| `is` | operator | Type test |
| `as` | operator | Type cast |
| `this` / `super` | keyword | Instance reference |
| `_` | pattern | Wildcard in match |

## Operators

| Operator | Description | Associativity | Precedence |
|----------|-------------|---------------|------------|
| `.` | Property access, method call | Left | 15 |
| `()` | Function call | Left | 14 |
| `[]` | Index, slice | Left | 13 |
| `as` | Type cast | Left | 12 |
| `**` | Power | Right | 11 |
| `*` / `/` / `%` | Multiply, divide, remainder | Left | 10 |
| `+` / `-` | Add, subtract | Left | 9 |
| `<<` / `>>` | Shift | Left | 8 |
| `&` | Bitwise AND | Left | 7 |
| `^` | Bitwise XOR | Left | 6 |
| `\|` | Bitwise OR | Left | 5 |
| `<` / `>` / `<=` / `>=` | Comparison | Left | 4 |
| `==` / `!=` | Equality | Left | 3 |
| `is` | Type test | Left | 3 |
| `..` / `..=` / `..<` | Range | Left | 2 |
| `and` | Logical AND | Left | 1 |
| `or` | Logical OR | Left | 0 |

## Literals

| Type | Examples |
|------|----------|
| Integer | `42`, `-17`, `0xFF`, `0b1010` |
| Float | `3.14`, `-2.5`, `1e10` |
| String | `"hello"`, `"line1\nline2"` |
| Boolean | `true`, `false` |
| None | `none` |
| List | `[1, 2, 3]` |
| Tuple | `(1, "a")` |

## Built-in Types

| Type | Description |
|------|-------------|
| `i8`, `i16`, `i32`, `i64` | Signed integers |
| `u8`, `u16`, `u32`, `u64` | Unsigned integers |
| `f32`, `f64` | Floating point |
| `bool` | Boolean (`true`/`false`) |
| `str` | String (heap-allocated, immutable) |
| `char` | Unicode character |
| `ptr` | Raw pointer |
| `T?` | Optional (`Option<T>`) |
| `T!` | Fallible (`Result<T, Error>`) |
| `&T` | Mutable type |
| `^T` | Move/ownership type |
| `[T]` | List of T |

## Compiler Flags

| Flag | Description |
|------|-------------|
| `--release` | Optimized build (SSA + O3) |
| `--emit-ir` | Dump LLVM IR |
| `--target` | Target triple |

## CLI

| Command | Description |
|---------|-------------|
| `ky build <file>` | Compile to binary |
| `ky run <file>` | Compile and run |
| `ky check <file>` | Type-check only |
| `ky parse <file>` | Parse and dump AST |
| `ky mir <file>` | Parse and dump MIR |
| `ky fmt [file]` | Format source |
| `ky test` | Run tests |
| `ky new <project>` | Create project |
| `ky add <dep>` | Add dependency |
| `ky remove <dep>` | Remove dependency |
| `ky update` | Update lockfile |
| `ky publish` | Publish package |
| `ky login` | Login to registry |
| `ky lsp` | Start LSP server |
| `ky completions <shell>` | Generate shell completions |
