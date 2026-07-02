# RFC 0001: Borrow Semantics (prev: Move Semantics)

- **Status:** Refactored (Phase 7 + Phase 14, v0.5.0)
- **Date:** 2026-06-01 (originally) / 2026-07-01 (refactored)
- **Author:** Kyle Compiler Team

---

## Summary (Refactored)

Kyle originally used **move-by-default** (v0.4.0). This RFC was updated to
reflect the **borrow-by-default** decision (v0.5.0+):

| Original (v0.4.0) | Current (v0.5.0+) |
|---|---|
| `fn f(s: str)` = move ownership | `fn f(s: str)` = borrow immutably |
| `fn f(s: &str)` = borrow (read-only) | `fn f(s: &str)` = borrow mutably |
| Hardcoded borrowing function list | Full borrow checker via `&T` / `^T` |
| `name := value` = mutable variable | `name: &T = value` = mutable variable |
| `name ::= value` = constant | `name := value` = constant |

The original content is preserved below for historical reference.

---

## Original: Move Semantics (v0.4.0)

### Motivation

Without move semantics:
- Every assignment of a string would copy the underlying buffer (slow)
- Or we'd need a GC/reference counting (runtime overhead, non-zero-cost)
- Or we'd need explicit `free()` calls (unsafe, error-prone)

Move semantics gives us:
- Zero-cost abstraction — ownership transfers are just pointer copies
- Safety — use-after-move is a compile-time error
- Predictability — no GC pauses, no refcount cycles

---

## Copy vs Move Type Classification

### Copy Types (implicitly duplicated)

| Type | Reason |
| :--- | :--- |
| `i8`–`i64` | Fits in register (≤8 bytes) |
| `u8`–`u64` | Fits in register |
| `f32`, `f64` | Fits in register |
| `bool` | Single byte |
| `char` | Single byte |
| `void` | Zero bytes |
| `ptr` | 8-byte pointer |

### Move Types (ownership transfers)

| Type | Reason |
| :--- | :--- |
| `str` | Heap-allocated buffer |
| `[T]` | Heap-allocated dynamic array |
| `{K:V}` | Heap-allocated hash map |
| `final class` | Heap-allocated struct |
| `class` | Heap-allocated object |

---

## Borrow-Assignment Distinction (v0.5.0+)

The original move-vs-copy classification still applies for **assignment**:

```kl
# Copy types: always implicit copy
x: i32 = 42
y = x              # x and y are independent

# Move types: ownership transfers on ASSIGNMENT
a = "hello"
b = a              # a is moved → use-after-move if read
```

But for **function parameters**, the default is now **borrow**:

```kl
fn read(s: str):       # BORROW: s is not moved
    println(s)
# s released at end, caller still owns

fn consume(^s: str):   # MOVE: ownership transfers
    # s destroyed at end, caller loses access
```

| Operation | Copy type | Move type |
|-----------|-----------|-----------|
| Assignment `y = x` | Bitwise copy | Ownership transfer (move) |
| Function param `s: T` | Bitwise copy | Borrow (no move) |
| Function param `^s: T` | Ownership transfer | Ownership transfer |
| Function param `s: &T` | Mutable borrow | Mutable borrow |
| Return value | Bitwise copy | Ownership transferred to caller |
| `.clone()` | N/A | Deep copy, both valid |
| Scope exit | Nothing | Release if owned |

---

## Dataflow Analysis Approach

The analysis is a **forward dataflow pass** over MIR basic blocks:

1. **State:** For each variable, track whether it is `Live` or `Moved`.
2. **Transfer function:** Assignment of a Move type marks the source as
   `Moved`. Assignment of a Copy type leaves the source `Live`.
3. **Join:** At CFG merge points, intersect the `Live` sets from all
   predecessors (a variable is `Live` at join only if `Live` on ALL
   incoming paths).
4. **Error detection:** If a variable is used while `Moved`, emit a
   compile-time error.
5. **Clone:** `.clone()` copies the value without moving the source.

### Borrowing (v0.5.0+)

The analysis also tracks borrowing state:

- `s: T` param → `s` is borrowed, caller keeps ownership
- `s: &T` param → `s` is borrowed mutably, caller keeps ownership
- `^s: T` param → ownership transferred, source invalidated

---

## Syntax Summary (v0.5.0+)

| Concept | Syntax |
|---------|--------|
| Immutable variable | `name = value` |
| Mutable variable | `name: &T = value` or `name = &value` |
| Constant | `NAME := value` |
| Immutable borrow param | `fn f(s: T)` |
| Mutable borrow param | `fn f(s: &T)` |
| Move param | `fn f(^s: T)` |
| Call: immutable borrow | `f(x)` |
| Call: mutable borrow | `f(&x)` |
| Call: move | `f(^x)` |
| Immutable field | `name: T` |
| Mutable field | `name: &T` |
| Mutable field with default | `name: &T = expr` |

---

## Edge Cases

### Conditional Moves

```kl
s = "hello"
if cond:
    x = s       # s is moved here
else:
    y = s       # s is also moved here (alternative path)
# s is moved on both paths → error to use s here

# But if only one path moves:
if cond:
    x = s       # s moved
else:
    pass        # s NOT moved
# s is Live at join (one path didn't move) → OK
```

### Return After Move

```kl
fn get_length(s: str) i32:   # borrow (not move)
    len = s.len()
    return len
# s is still Live → OK
```

### Clone and Use

```kl
s = "hello"
t = s.clone()   # s is borrowed (not moved), t is a new copy
println(s)      # OK: s is still Live
println(t)      # OK: t is Live
```

### Borrow at Call Site

```kl
fn append(s: &str):
    s = s + "!"

name: &str = "Kyle"
append(name)            # ✅ &str → &str (mutable, direct)

nick = "Ana"            # immutable
append(&nick)           # ✅ str → &str with & coercion
append(nick)            # ❌ missing & → compile error
```

---

## Implementation Details

- **File:** `kyc_mir/src/borrow_analysis.rs` (formerly `move_analysis.rs`)
- **Pass location:** Between MIR lowering and codegen
- **Pipeline integration:** Build fails on use-after-move errors
- **Parameter rules:** Tracked via MIR parameter annotations (Borrow, MutableBorrow, Move)
- **Testing:** 9+ end-to-end tests in `kyc_driver` covering copy types,
  clone, borrowing, params, if/else branches, use-after-move, return after move
