# Test & Verification Checklist — Phase 12 (Tooling)

> Use this checklist to manually verify every feature built in Phase 12.
> Mark `[x]` when confirmed working, `[ ]` if broken.
> Report bugs with **file:line** and **actual vs expected** behavior.

---

## 1. Compiler CLI (`ky`)

### 1.1 Build & Run
- [ ] `kl build src/main.ky` — compiles to native binary
- [ ] `kl run src/main.ky` — compiles and executes
- [ ] `kl run hello.ky` (no `main()`) — runs as script
- [ ] `ky build --release` — produces optimized binary
- [ ] `kl check file.ky` — type-check only (no binary)
- [ ] `kl parse file.ky` — dumps AST
- [ ] `kl mir file.ky` — dumps MIR

### 1.2 Test Framework
- [ ] `ky test` (in project dir) — runs all `#[test]` functions
- [ ] `ky test file.ky` — runs tests in single file
- [ ] Test PASS prints correctly
- [ ] Test FAIL prints error + location
- [ ] `#[test] fn test_name():` — test function is ignored during normal build
- [ ] `assert(condition)` — panics if false
- [ ] `assert_eq(a, b)` — panics if a != b
- [ ] `assert_ne(a, b)` — panics if a == b

### 1.3 Formatter (`ky fmt`)
- [ ] `ky fmt file.ky` — formats file in-place
- [ ] `ky fmt --check file.ky` — exits 1 if would reformat
- [ ] `ky fmt` (no args, in project dir) — formats all `src/*.ky` and `tests/*.ky`
- [ ] `ky fmt src/` — formats all `.ky` in directory
- [ ] Roundtrip idempotency: `ky fmt file.ky && ky fmt --check file.ky` passes
- [ ] `&T` syntax preserved (mutable variables)
- [ ] `:=` syntax preserved (compile-time constants)
- [ ] `final class` preserved
- [ ] `abstract class` preserved
- [ ] `#[test]` preserved before test functions
- [ ] `T?` syntax preserved (optional types)
- [ ] `T!` syntax preserved (error types)
- [ ] `abstract fn` preserved
- [ ] Imports sorted: relative first, then absolute, alphabetical within groups
- [ ] Closure syntax `(params) => expr` preserved
- [ ] Tuple patterns `(a, b)` preserved
- [ ] Enum variant patterns `Enum.Variant(args)` preserved
- [ ] Or-patterns `a | b` preserved
- [ ] Match guards `pattern if cond` preserved
- [ ] `[format]` config in `ky.toml` is respected (`max_line_width`, `indent_size`)

### 1.4 Package Manager
- [ ] `ky new myproject` — creates project skeleton
- [ ] `ky add foo` — resolves and adds dependency
- [ ] `ky add foo@1.2.3` — adds specific version
- [ ] `ky remove foo` — removes dependency
- [ ] `ky info` — shows project info
- [ ] `ky update` — updates lock file
- [ ] `ky outdated` — lists outdated dependencies
- [ ] `ky publish` — publishes to registry
- [ ] `ky login` — logs into registry

### 1.5 Shell Completions
- [ ] `ky completions bash` — outputs valid bash completion script
- [ ] `ky completions zsh` — outputs valid zsh completion script
- [ ] `ky completions fish` — outputs valid fish completion script
- [ ] `ky completions powershell` — outputs valid PowerShell completion script
- [ ] `ky add <TAB>` — suggests cached package names (all shells)

---

## 2. LSP (`ky lsp`)

### 2.1 Diagnostics
- [ ] Open a `.ky` file with syntax error → red squiggly + Problems panel entry
- [ ] Open a `.ky` file with type error → error reported
- [ ] Fix error → diagnostics clear automatically
- [ ] `ky.toml` manifest errors shown (missing fields, invalid semver)
- [ ] Many quick edits → incremental sync works (no crash, no stale errors)

### 2.2 Completions
- [ ] Type `pr` → suggests `print`, `println`
- [ ] Type `i32` → completion for builtin types
- [ ] Type `fn ` → completion for function keyword
- [ ] Type `import ` → completion for modules
- [ ] Type `ma` → suggests `match`

### 2.3 Go-to-Definition
- [ ] Ctrl+click on function name → jumps to its definition (same file)
- [ ] Ctrl+click on variable → jumps to its declaration
- [ ] Ctrl+click on import → jumps to the module file (in `~/.ky/cache/` for deps)

### 2.4 Hover
- [ ] Hover over variable → shows inferred type
- [ ] Hover over function → shows signature
- [ ] Hover over type → shows type definition

### 2.5 Inlay Hints
- [ ] Variable without type annotation → shows `: Type` hint
- [ ] Function without return type → shows `-> Type` hint

### 2.6 Code Lens
- [ ] `#[test] fn test_name():` → shows "Run test" button above the function
- [ ] Click "Run test" → compiles and runs just that test

### 2.7 Format on Save
- [ ] Save `.ky` file → file is auto-formatted

---

## 3. VS Code Extension

### 3.1 Installation
- [ ] VSIX installs without errors
- [ ] Extension activates on `.ky` files
- [ ] Extension activates on `ky.toml` files
- [ ] Language icon appears for `.ky` files in file explorer

### 3.2 Commands (Ctrl+Shift+P)
- [ ] `KL: Run current file` — compiles and runs
- [ ] `KL: Build current file` — compiles to binary
- [ ] `KL: Type-check current file` — type-checks only
- [ ] `KL: Run tests in current file` — runs all #[test] functions
- [ ] `KL: Run specific test` — runs one test

### 3.3 Tasks
- [ ] Terminal > Run Task > `ky: run/build/check/test` — each works

### 3.4 Testing UI
- [ ] Open Testing panel → discovers `#[test]` functions
- [ ] Click "Run Tests" → runs all tests, shows PASS/FAIL
- [ ] File watcher: create new `#[test]` → appears in Testing panel
- [ ] Debug profile for tests works (launches with debugger)

### 3.5 Debugger
- [ ] F5 → launches debug configuration picker
- [ ] Select "KL: Launch" → compiles and runs the program
- [ ] Output appears in Debug Console
- [ ] Set breakpoint → debugger pauses (requires runtime support)

### 3.6 Snippets
- [ ] Type `fn` → snippet for function declaration
- [ ] Type `class` → snippet for class
- [ ] Type `match` → snippet for match
- [ ] Type `for` → snippet for for loop
- [ ] Type `test` → snippet for test function
- [ ] All 35+ snippets produce valid Kyle syntax

### 3.7 Syntax Highlighting
- [ ] Keywords highlighted: `fn`, `final`, `abstract`, `match`, `if`, `while`, `for`
- [ ] Types highlighted: `i32`, `str`, `bool`, `f64`
- [ ] `:=` highlighted as constant operator
- [ ] `T?`, `T!` highlighted
- [ ] Comments highlighted
- [ ] Strings and string interpolation highlighted

### 3.8 Color Theme
- [ ] Select "Kyle Pastel" theme → colors are applied
- [ ] Syntax tokens use correct pastel colors
- [ ] UI elements (sidebar, title bar, tabs) use dark theme

---

## 4. Syntax — Modern Kyle

### 4.1 Variable Declarations
- [ ] `name := value` — mutable variable (walrus)
- [ ] `name = value` — immutable variable
- [ ] `name := value` — compile-time constant
- [ ] `name: Type = value` — typed immutable
- [ ] `name: Type := value` — typed mutable

### 4.2 Classes
- [ ] `final class Name:` — lightweight class
- [ ] `final class Name < Parent:` — with inheritance
- [ ] `final class Name<T>:` — generic
- [ ] `abstract class Name:` — abstract class

### 4.3 Functions
- [ ] `fn name():` — no params, no return
- [ ] `fn name(x: i32) str:` — params + return type
- [ ] `fn name<T>(x: T) T:` — generic
- [ ] `const fn name():` — compile-time function
- [ ] `async fn name():` — async function
- [ ] `abstract fn name():` — abstract function
- [ ] `fn name(x: i32 = 5):` — default parameter

### 4.4 Pattern Matching
- [ ] `match x: literal: body` — literal patterns
- [ ] `match x: name: body` — binding patterns
- [ ] `match x: _: body` — wildcard
- [ ] `match x: Enum.Variant: body` — enum variant
- [ ] `match x: Enum.Variant(args): body` — variant with payload
- [ ] `match x: a \| b: body` — or-patterns
- [ ] `match x: pattern if cond: body` — guards

### 4.5 Error Handling
- [ ] `fn f() T!:` — function returns Result<T, Error>
- [ ] `val := expr?` — propagate error
- [ ] `return error("msg")` — create error

### 4.6 Async
- [ ] `t := async fn_call()` — spawn async task
- [ ] `val := await t` — await result

### 4.7 Types
- [ ] `T?` — optional type (Option<T>)
- [ ] `T!` — error type (Result<T, Error>)
- [ ] `fn(T) U` — function pointer type
- [ ] `fn(T) U async` — async function pointer type
- [ ] `Dict<K, V>` — dictionary type

### 4.8 Imports
- [ ] `import module` — absolute import
- [ ] `import ~module` — relative import
- [ ] `from module import name` — selective import
- [ ] `import module as alias` — aliased import

### 4.9 Closures
- [ ] `(x) => x * 2` — inline closure
- [ ] `(x) =>\n  body` — block-bodied closure

---

## 5. Standard Library

### 5.1 Core (`import core`)
- [ ] `Option<T>`, `None`, `Some(T)` — option types
- [ ] `unwrap_or`, `is_some`, `is_none` — option helpers

### 5.2 IO (`import io`)
- [ ] `print(value)` — prints without newline
- [ ] `println(value)` — prints with newline
- [ ] `input() str` — reads line from stdin

### 5.3 Math (`import math`)
- [ ] `abs`, `min`, `max` — basic math functions
- [ ] `sqrt`, `sin`, `cos`, `tan`, `floor`, `ceil` — advanced math

### 5.4 String (`import str`)
- [ ] `upper()`, `lower()`, `trim()` — string transformations
- [ ] `contains()`, `replace()`, `split()` — search/replace
- [ ] `len()`, `starts_with()`, `ends_with()` — string queries
- [ ] String interpolation: `"Hello {name}"`

### 5.5 Collections (`import collections`)
- [ ] List literals `[1, 2, 3]`
- [ ] `add()`, `pop()`, `insert()`, `remove()` — mutation
- [ ] `len()`, `contains()` — queries
- [ ] `reverse()`, `clear()` — bulk operations
- [ ] Dict literals `{"a": 1}`

### 5.6 Time (`import time`)
- [ ] `now()`, `sleep(ms)` — time functions

### 5.7 Testing (`import testing`)
- [ ] Test utilities available

---

## 6. Edge Cases & Stress Tests

### 6.1 Large Files
- [ ] Format a 1000+ line `.ky` file — completes within 2 seconds
- [ ] LSP handles a 5000+ line file without slowdown
- [ ] Compiler handles 100+ source files in a project

### 6.2 Error Recovery
- [ ] Parser reports multiple errors (not just first)
- [ ] Error messages include file:line:column
- [ ] Missing closing quote → reports error, continues parsing
- [ ] Wrong indentation → reports error

### 6.3 Concurrency
- [ ] LSP handles rapid edits without crashing
- [ ] Multiple `ky` commands can run simultaneously
- [ ] `ky test` runs tests in parallel (if applicable)

### 6.4 Package Manager
- [ ] `ky add` with network error → graceful error message
- [ ] `ky add` on existing dependency → updates version
- [ ] Circular dependencies → error reported
- [ ] Lock file conflict → manual resolution message

---

## 7. Platform-Specific

### 7.1 macOS (Apple Silicon)
- [ ] Full test suite passes
- [ ] VS Code extension loads correctly
- [ ] `ky lsp` works with VS Code on macOS

### 7.2 Linux (ARM64)
- [ ] Full test suite passes
- [ ] Binary runs correctly

### 7.3 Linux (x64)
- [ ] Full test suite passes
- [ ] Binary runs correctly

---

*Generated: 2026-06-30 · Kyle v0.4.0 · 157 Rust tests passing*
