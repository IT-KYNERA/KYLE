# RFC-0001: Move Semantics

**Status:** Implemented  
**Date:** 2026-06-01

## Summary

Kyle uses borrow semantics by default. Parameters borrow unless explicitly marked with `^T` for ownership transfer.

## Motivation

- Safety: prevent use-after-move by default
- Ergonomics: most parameters don't need ownership transfer
- Consistency: borrow by default, move explicit

## Design

```ky
fn read(s: str):          # borrow (default)
fn append(s: &str):       # mutable borrow
fn consume(^s: str):      # move (explicit)
```

## Copy types

Simple types (i32, i64, f32, f64, bool) are Copy — they are never moved, always copied. Complex types (str, list, dict, struct) are moved by default on assignment and explicitly borrowed or cloned.
