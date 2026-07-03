# Principles

## 1. Readability

Code reads like prose. No `;`, no `{}`, no `let`/`var`/`const`. Just indentation (4 spaces) and direct assignment.

## 2. Strong typing with inference

The compiler knows all types at compile time. The programmer writes the minimum necessary.

## 3. Radical simplicity

One way to do each thing. No exceptions.

## 4. Zero-cost performance

Borrow semantics by default, ownership via `^`. No GC, no implicit refcounting. You don't pay for what you don't use.

## 5. Syntactic consistency

What looks the same behaves the same. `T?` replaces `Option<T>`. `final class` replaces `struct`. No surprises.
