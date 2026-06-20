# Kyle Error Catalog Specification v1.0

## Philosophy

Every compiler error must be:
- Human-readable
- Actionable (tells the developer what to fix)
- Consistent format
- Machine-parseable

---

## Format

```text
Kyle-EXXXX: <title>

  <description>

  --> file.kl:line:col

  <suggestion>
```

---

## Error Codes

### E0001 - Type Mismatch

```text
Kyle-E0001: Type mismatch

  Expected <type>, received <type>

  --> file.kl:10:5

  The value provided does not match the expected type.
  Check the variable declaration or function signature.
```

---

### E0002 - Unhandled Error

```text
Kyle-E0002: Unhandled error value

  A fallible function returns an error that must be handled.

  --> file.kl:10:5

  Use match, ?, or try to handle the error:
    match result:
        ok(val):
        error(e):
```

---

### E0003 - Unsafe Operation Outside Unsafe Block

```text
Kyle-E0003: Unsafe operation outside unsafe block

  Pointer arithmetic and FFI calls require an unsafe block.

  --> file.kl:10:5

  Wrap the operation in an unsafe block:
    unsafe:
        <operation>
```

---

### E0004 - Non-Exhaustive Match

```text
Kyle-E0004: Non-exhaustive match

  Match does not cover all cases.

  --> file.kl:10:5

  Add missing arms for: <variants>
```

---

### E0005 - Unreachable Code

```text
Kyle-E0005: Unreachable code

  Code after return, break, or panic cannot be executed.

  --> file.kl:10:5

  Remove or restructure the unreachable code.
```

---

### E0006 - Circular Dependency

```text
Kyle-E0006: Circular dependency detected

  Module A imports module B which imports A.

  --> file.kl:10:5

  Extract the shared dependency into a separate module.
```

---

### E0007 - Cannot Modify Constant

```text
Kyle-E0007: Cannot modify constant

  UPPERCASE names are constants and cannot be reassigned.

  --> file.kl:10:5

  Use a lowercase variable name if reassignment is needed.
```

This error also occurs when attempting to reassign an immutable variable:

```text
Kyle-E0007: Cannot modify constant

  cannot assign to immutable variable 'x'

  --> file.kl:10:5

  Declare with 'mut' to make it mutable
```

---

### E0008 - Optional Not Checked

```text
Kyle-E0008: Optional value not checked

  Accessing an optional value without checking for None.

  --> file.kl:10:5

  Check with if or match before accessing:
    if value:
        print(value)
```

---

### E0009 - Undefined Symbol

```text
Kyle-E0009: Undefined symbol

  <name> is not defined in the current scope.

  --> file.kl:10:5

  Check the spelling or add an import statement.
```

---

### E0010 - Potential Data Loss

```text
Kyle-E0010: Potential data loss

  Narrowing conversion from <type> to <type> may lose data.

  --> file.kl:10:5

  Use an explicit cast: <type>(value)
```

---

### E0011 - Integer Overflow

```text
Kyle-E0011: Integer overflow

  Arithmetic operation overflowed in debug build.

  --> file.kl:10:5

  Use wrapping operators (+%, -%, *%) if overflow is intended.
```

---

### E0012 - Division By Zero

```text
Kyle-E0012: Division by zero

  Division or modulo by zero is not allowed.

  --> file.kl:10:5

  Check the divisor before the operation.
```

---

### E0013 - Invalid UTF-8

```text
Kyle-E0013: Invalid UTF-8 sequence

  String literal contains invalid UTF-8 bytes.

  --> file.kl:10:5

  Use valid UTF-8 encoding in string literals.
```

---

### E0014 - Private Access

```text
Kyle-E0014: Cannot access private member

  <name> is private (__ prefix) to its module.

  --> file.kl:10:5

  Use the module's public API instead of accessing internals.
```

---

### E0015 - Unused Import

```text
Kyle-E0015: Unused import (warning)

  <name> is imported but never used.

  --> file.kl:10:5

  Remove the unused import.
```

---

### E0016 - Dead Code

```text
Kyle-E0016: Dead code (warning)

  <name> is declared but never used.

  --> file.kl:10:5

  Remove or prefix with _ to suppress.
```

---

### E0017 - Generic Type Not Found

```text
Kyle-E0017: Unknown generic type

  <name> is not a valid type argument.

  --> file.kl:10:5

  Check that the type exists and is imported.
```

---

### E0018 - Fallible Function Not Handled

```text
Kyle-E0018: Fallible function not handled

  A function marked with ! returns a Result that must be handled.

  --> file.kl:10:5

  Use match, ?, or propagate the error with !.
```

---

### E0019 - Cannot Inherit From Final Class

```text
Kyle-E0019: Cannot inherit from final class

  Class <name> is not marked as inheritable.

  --> file.kl:10:5

  Remove the inheritance or mark the base class as open.
```

---

### E0020 - Invalid Attribute Argument

```text
Kyle-E0020: Invalid attribute argument

  Attribute #[<name>] received an invalid argument.

  --> file.kl:10:5

  Check the attribute documentation for valid arguments.
```

---

## Warnings

### W0001 - Unused Variable

```text
Kyle-W0001: Unused variable (warning)

  <name> is assigned but never read.

  --> file.kl:10:5

  Prefix with _ to suppress: _<name>
```

---

### W0002 - Shadowed Variable

```text
Kyle-W0002: Shadowed variable (warning)

  <name> shadows a previous declaration in the same scope.

  --> file.kl:10:5

  Rename one of the variables.
```

---

### W0003 - Redundant Cast

```text
Kyle-W0003: Redundant cast (warning)

  Cast from <type> to <type> is unnecessary.

  --> file.kl:10:5

  Remove the explicit cast.
```

---

## Compiler Panic

### P0001 - Internal Compiler Error

```text
Kyle-P0001: Internal compiler error

  Unexpected error in <compiler_stage>.

  Please report this at:
  https://github.com/IT-KYNERA/KYLE/issues
```

---

## Lint Rules — 🔶 Not Yet Implemented

```text
L0001  Prefer explicit types on public API          🔶
L0002  Avoid deep nesting (>4 levels)               🔶
L0003  Function too long (>100 lines)                🔶
L0004  File too long (>1000 lines)                   🔶
L0005  Missing error handling on fallible call       🔶
L0006  Prefer early return over nested if           🔶
L0007  Use guard instead of if-not-return pattern   🔶
L0008  Prefer object literal over class for simple data 🔶
```

---

# Version

```text
Kyle Error Catalog Specification v2.0
Last updated: 2026-11-19
```
