# Diagnostics

> Sistema de errores y diagnósticos del compilador.
> Crate: `kyc_core/src/diagnostics.rs` / `kyc_core/src/span.rs`

## Responsabilidad

El sistema de diagnósticos provee errores y warnings con ubicación precisa
en el código fuente, facilitando la depuración.

## Error Codes

Cada error tiene un código único:

| Código | Significado |
|--------|-------------|
| `KL-E0001` | Type mismatch |
| `KL-E0002` | Syntax error |
| `KL-E0003` | Undefined type |
| `KL-E0004` | Cannot modify immutable |
| `KL-E0005` | Undefined function |
| `KL-E0006` | Wrong number of arguments |
| `KL-E0007` | Cannot modify constant |
| `KL-E0008` | Invalid assignment target |
| `KL-E0009` | Undefined symbol |
| `KL-E0010` | Wrong number of type arguments |
| `KL-E0011` | Ambiguous type |
| `KL-E0012` | Return type mismatch |
| `KL-E0013` | Move analysis error |

## Estructura

```rust
 struct Diagnostic {
     code: ErrorCode,
     message: String,
     span: Option<Span>,
     notes: Vec<String>,
}

 struct Span {
     start: usize,    // byte offset
     end: usize,
     line: u32,
     column: u32,
}
```

## Formato de salida

```
KL-E0001: Type mismatch

  expected 'i32', found 'str'

  --> file.ky:10:5
   |
10 |     x = "hello" + 42
   |         ^^^^^^^^^^^^^ expected i32 here
   |
```

## Warnings

Además de errores, el compilador produce warnings:

| Warning | Condición |
|---------|-----------|
| Unused variable | Variable declarada pero nunca usada |
| Unused import | Import de módulo no utilizado |
| Unreachable code | Código después de return/break |
| Deprecated feature | Uso de sintaxis obsoleta |
| Implicit conversion | Conversión de tipos sin `as` |

## Ver también

- `semantic.md` — Genera la mayoría de los errores semánticos
- `parser.md` — Genera errores sintácticos
- `borrow-analysis.md` — Genera errores de ownership
