# RFC 0001: Move Semantics

- **Status:** Implemented (Phase 7, v0.4.0)
- **Date:** 2026-06-01
- **Author:** Kyle Compiler Team
- **PR:** [Move semantics implementation](#)

---

## Summary

Kyle uses **move semantics by default** for heap-allocated types (`str`,
`[T]`, `{K:V}`, classes). Copy types (integers, floats, bools, chars) are
implicitly duplicated on assignment. This design eliminates the need for a
garbage collector while keeping the language safe from use-after-free bugs.

---

## Motivation

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

---

## Borrowing Functions Concept

Not all functions take ownership. Functions like `print()`, `println()`,
`len()`, and list mutators (`list_push`, `list_get`, `list_set`) are
designated as **borrowing functions** — they read or mutate the value
without consuming it.

The move analysis has a hardcoded list of borrowing function names. When
a value is passed to a borrowing function, the source remains `Live`.

This is an interim solution. A full borrow checker (references `&T`/`&mut T`)
is planned post-v1.0.

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
fn get_length(s: str) i32:
    len = s.len()     # s is borrowed, not moved
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

---

## Implementation Details

- **File:** `klc_mir/src/move_analysis.rs`
- **Pass location:** Between MIR lowering and codegen
- **Pipeline integration:** Build fails on use-after-move errors
- **Testing:** 9 end-to-end tests in `klc_driver` covering copy types,
  clone, borrowing functions, params, if/else branches, use-after-move,
  and return after move
