# Kyle — AI Agent Context

> **Read this first.** Single entry-point for AI agents working on the Kyle codebase.

## Quick start

```bash
cargo build --release --bin ky
cargo test --workspace
ky run examples/test_kyle_syntax.ky
```

## Project structure

```
ky/
├── crates/                    → Rust crates (compiler)
│   ├── kyc_core/              → AST types, diagnostics, span
│   ├── kyc_frontend/          → Lexer + Parser
│   ├── kyc_hir/               → HIR desugaring
│   ├── kyc_semantic/          → Type checker, scope, borrow analysis
│   ├── kyc_mir/               → MIR lowering, SSA, optimizations
│   ├── kyc_backend/           → LLVM codegen, linker
│   ├── kyc_driver/            → Compilation pipeline
│   ├── kyc_cli/               → CLI binary (`ky`)
│   ├── kyc_runtime/           → Runtime library (Rust)
│   ├── kyc_platform/          → Platform API (fs, time)
│   └── kyc_ui/                → UI framework
├── docs/                      → Documentación
│   ├── 03-language/syntax/    → Sintaxis del lenguaje
│   ├── 09-specification/      → Type system, ABI
│   └── 11-project/            → Roadmap, self-hosting
├── runtimes/ky/               → Self-hosting compiler (Kyle-in-Kyle)
├── tests/syntax/              → 13 syntax tests
└── examples/                  → Example .ky projects
```

## Architecture

```
source → kyc_frontend → kyc_hir → kyc_semantic → kyc_mir → kyc_backend → binary
```

---

## Anchored Summary (last updated: 2026-07-23)

### FASE 1 — Compiler Bugs (COMPLETED)

| Bug | Estado | Fix |
|-----|--------|-----|
| #1 while-loop strings | ✅ | `ssa/mod.rs:249-254` — retain solo promotables |
| #2 Option/Result lowering | ✅ | `lower/expr.rs:1644-1712` + `lower/stmt.rs:169-199` |
| #3 &str + concat garbage | ✅ | `lower/expr.rs:4983-5008` — BorrowRef pasa ptr directo |
| #4 move analysis for loops | ✅ | `lower/stmt.rs:676-701` — usar alloca original |
| #5 fn main() return type | ✅ | `lower/function.rs` + 4 sitios |
| #6 varios menores | ✅ | genéricos, dict.get casteo, BorrowRef list/dict/set |

### FASE 2 — Syntax Redesign (DOCUMENTATION COMPLETE)

**Import system: solo `use`, nada de `from X import Y`**

```ky
use std.io                          # módulo completo
use std.io.{print, read}            # selectivo
use std.io as io                    # alias
use ~utils.helpers                  # relativo
use std.io.print                    # símbolo directo
```

**Collection types**

| Sintaxis | Descripción | Literal | Estado |
|----------|-------------|---------|--------|
| `[T]` | Lista | `[1, 2, 3]` | [~] Parcial |
| `[T, N]` | Array fijo | tipo `[T, N]` | [x] Existe |
| `set{T}` | Set | `set{1, 2, 3}` | [ ] Nuevo |
| `{K: V}` | Dict | `{"a": 1}` | [x] Existe |
| `queue{T}` | Queue | `queue{1, 2}` | [ ] Nuevo |
| `stack{T}` | Stack | `stack{"a"}` | [ ] Nuevo |

**Type orthogonality: `^` `&` `?` `!` en TODOS los tipos**

```ky
x: ^&[i32]!      # mutable borrow con error
x: ^&[str]?      # mutable borrow opcional
x: ^set<i32>!    # set mutable con error
```

### FASE 3 — Self-Hosting Blocker Bugs (ALL FIXED ✅)

| Bug | Estado | Fix |
|-----|--------|-----|
| #7 elif + strings → ky_free | ✅ | `borrow_analysis/mod.rs:322-329` — excluir ky_clone_str de cleanup |
| while-loop strings | ✅ | Fix previo `ssa/mod.rs:249-254` (Bug #1) |
| struct str field → empty | ✅ | Fix previo `lower/expr.rs:4983-5008` (Bug #3) |
| ^{str} list garbage | ✅ | Fix previo `lower/expr.rs:2282-2300` (Bug #6 dict get cast) |
| ^str.get/set → _call error | ✅ | Fix previo BorrowRef str (Bug #3 + #6) |

### Test Status

- `cargo test --workspace`: 117+ pass (kyc_runtime_wasm segfault pre-existing)
- All 13 syntax tests: **PASS**
- Failing: ninguno

### Docs Map

| Documento | Contenido |
|-----------|-----------|
| `docs/03-language/syntax/modules.md` | Sistema `use` |
| `docs/03-language/syntax/variables.md` | `^` `&` `?` `!` |
| `docs/03-language/syntax/collections.md` | Todas las colecciones |
| `docs/03-language/syntax/expressions.md` | Literales `[1,2,3]` |
| `docs/09-specification/type-system.md` | Ortogonalidad de tipos |
| `docs/15-kyle-syntax-reference.md` | Referencia completa |
| `docs/11-project/syntax-roadmap.md` | Plan de implementación |
| `docs/11-project/roadmap.md` | Roadmap general |
| `docs/11-project/self-hosting.md` | Estado self-hosting |

---

## Implementation Order (next)

1. **Parser**: `use` keyword, `[T]` type, `set{T}` type, `queue{T}`
2. **Type Checker**: `set<T>` type, orthogonality for collections
3. **MIR Lowering**: new collection types, conversion methods
4. **Tests**: update `06_collections.ky`, new syntax tests
5. **Bugs**: #7 elif strings, while-loop, struct str field
6. **Bootstrap**: ky2c.ky → self-hosting
