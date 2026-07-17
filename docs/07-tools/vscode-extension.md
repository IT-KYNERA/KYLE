# VS Code Extension for Kyle

> File: `vscode-ky/` — Extension runtime (TypeScript)
> File: `crates/kyc_tools/src/lsp.rs` — LSP server (Rust, via `ky lsp`)

## Architecture

```
VS Code (extension.ts)
    │
    ├── LSP Client ↔ ky lsp (Rust binary)
    │     ├── textDocument/didOpen     → publishDiagnostics
    │     ├── textDocument/didChange   → publishDiagnostics (on every keystroke)
    │     ├── textDocument/completion  → module paths, symbols, keywords, exports
    │     ├── textDocument/hover       → type info + docs
    │     ├── textDocument/definition  → go-to-definition
    │     ├── textDocument/references  → find references (text-based)
    │     ├── textDocument/documentSymbol → outline (flat, not hierarchical)
    │     ├── textDocument/semanticTokens/full → syntax coloring (9 token types)
    │     ├── textDocument/codeAction  → quick fixes (E0009 only)
    │     ├── textDocument/formatting  → ky fmt
    │     ├── textDocument/rename      → rename symbol
    │     ├── textDocument/signatureHelp → param hints
    │     ├── textDocument/codeLens    → test run buttons
    │     └── textDocument/inlayHint   → type hints on vars/fns
    │
    ├── Commands (Terminal tasks)
    │     ├── ky.run      → ky run current file
    │     ├── ky.build    → ky build current file
    │     ├── ky.check    → ky check current file
    │     └── ky.test     → ky test
    │
    └── Debugger (DAP)
          └── ky debug (VSCode Debug Adapter Protocol)

Language Grammar (TextMate)
    ├── syntaxes/ky.tmLanguage.json    → .ky files
    └── syntaxes/kyx.tmLanguage.json   → .kyx files (UI templates)

Snippets
    ├── snippets/ky.json               → .ky snippets
    └── snippets/kyx.json              → .kyx snippets

Theme
    └── themes/kl-color-theme.json     → Custom Kyle theme
```

## Key Files and Their Locations

### Extension (TypeScript) — `vscode-ky/src/`

| File | Purpose | Key Lines |
|------|---------|:---------:|
| `extension.ts` | Activation, commands, LSP client, debugger | 238 lines |
| `tasks.ts` | Terminal tasks (ky run/build/check/test) | — |
| `debugger.ts` | DAP debugger adapter | — |
| `testUI.ts` | Testing UI | — |

### LSP Server (Rust) — `crates/kyc_tools/src/lsp.rs`

| Section | Lines | Description |
|---------|:-----:|-------------|
| Server capabilities | 55–100 | What features the LSP advertises |
| Request handlers | 110–135 | Route table for 15 handlers |
| Notification handlers | 137–144 | didOpen, didChange, didSave, didClose |
| `publish_diagnostics` | 378–441 | Parse + analyze + send diagnostics |
| `resolve_and_analyze` | 219–327 | The CORE analysis pipeline for LSP |
| `collect_search_paths` | 330–375 | Filesystem paths for module resolution |
| `handle_completion` | 637–861 | Autocomplete: keywords, symbols, modules |
| `handle_hover` | 867–935 | Hover info with type/docs |
| `handle_definition` | 1640–1700 | Go-to-definition |
| `handle_references` | 1826–1843 | Find references (text-based) |
| `handle_document_symbol` | 1750–1758 | Outline (flat list) |
| `handle_semantic_tokens_full` | 1992–2177 | All semantic token computation |
| Semantic token encoding | 2178–2210 | Delta encoding for VS Code |
| `walk_semantic_block` | 2221–2340 | Recursive block tokenization |
| `walk_semantic_type` | 2461–2502 | Type annotation tokenization |

### Syntax Highlighting — `vscode-ky/syntaxes/ky.tmLanguage.json`

| Rule | Lines | Description |
|------|:-----:|-------------|
| `comments` | 35–39 | `#` single-line, `##` doc-comment |
| `strings` | 41–50 | `"..."` with escape sequences + interpolation |
| `interpolation` | 53–57 | `{expr}` inside strings |
| `storage_modifiers` | 104–108 | `^` (mutable), `&` (borrow) as type modifiers |
| `keywords_declaration` | 89–102 | `class`, `fn`, `enum`, `contract`, etc. |
| `types_primitive` | 120–122 | `i32`, `str`, `bool`, `f64`, etc. |
| `types_complex` | 124–130 | `[T;N]`, `{T}`, `{K:V}`, `T<U>` |
| `operators` | 146–157 | `::`, `:=`, `==`, `+=`, etc. |
| `class_def` | 159–178 | `class Name :: Parent, Contract:` highlight |
| `function_def` | 241–264 | `fn name(params) Type:` highlight |
| `variable_decl` | 303–308 | `name = value` highlight (excludes `==`) |
| `builtins` | 138–140 | `print`, `len`, `push`, etc. |
| `constants` | 142–144 | `SCREAMING_SNAKE` constants |
| `constructor_call` | 283–297 | `Name(...)` and `Name{...}` as type refs |

## How the LSP Analysis Pipeline Works

### `resolve_and_analyze(uri, source)` — Lines 219–327

1. Lex + parse the file into AST
2. Create `ModuleResolver` with search paths
3. Resolve `from X import Y` → fetch file, parse, cache, splice ALL declarations
4. Resolve transitive imports (recursive in cached modules)
5. Pull contracts from cached modules (for imported classes with contract dependencies)
6. Run `SemanticAnalyzer::analyze(&program)` (Phase 1–3)
7. Return `(program, analyzer, file_name)`

### How Diagnostics Work

```rust
fn publish_diagnostics(uri) {
    let (program, analyzer, _) = resolve_and_analyze(uri, source);
    for diagnostic in analyzer.reporter().diagnostics() {
        // map span → Range, severity → DiagnosticSeverity
        // send textDocument/publishDiagnostics notification
    }
}
```

Called on:
- `didOpen` — when file opens
- `didChange` — on every keystroke (NOT debounced)
- `didSave` — runs analysis but diagnostic publishing is handled by `didChange`

## Semantic Tokens (Coloring)

9 token types, 3 modifiers:

| Index | Type | Used For |
|:-----:|------|----------|
| 0 | `VARIABLE` | Variables (with `DECLARATION`, `MODIFICATION`, or `READONLY`) |
| 1 | `TYPE` | Type annotations, imported types |
| 2 | `CLASS` | Class names (declaration + reference) |
| 3 | `STRUCT` | Struct names |
| 4 | `ENUM` | Enum names |
| 5 | `FUNCTION` | Function names |
| 6 | `METHOD` | Method calls via `.name()` |
| 7 | `PARAMETER` | Function parameters |
| 8 | `PROPERTY` | Struct/class fields, property access |

Tokenized in `walk_semantic_block` (lines 2221+) by walking the AST.

## Common Issues & Fixes

### 1. Semantic tokens offset by 1 (first letter wrong color)

**File**: `lsp.rs`, lines 2201–2202
**Fix**: Convert 1-indexed spans to 0-indexed before delta encoding:
```rust
let line0 = line.saturating_sub(1);
let col0 = col.saturating_sub(1);
```

### 2. LSP shows "undefined symbol" for imported types

**File**: `lsp.rs`, lines 277–284
**Fix**: Import ALL declarations from the module, not just the requested names:
```rust
import_decls.push((i, module.program.declarations.clone()));
```

### 3. LSP shows "contract not found" for imported classes

**File**: `lsp.rs`, lines 303–319
**Fix**: After resolving imports, scan cached modules for contracts referenced by imported classes.

### 4. Syntax highlight breaks on `::`

**File**: `ky.tmLanguage.json`, `class_def` end pattern
**Fix**: Use `end: "(?=(?:[^:]):|$|\\n)"` instead of `end: "(?=:|\n)"` to avoid matching the first `:` of `::`.

### 5. `fn` keyword not colored

**File**: `ky.tmLanguage.json`, `keywords_declaration`
**Fix**: Add explicit pattern:
```json
{ "name": "keyword.declaration.fn.ky", "match": "\\bfn\\b" }
```

### 6. Variables highlighted on `==`

**File**: `ky.tmLanguage.json`, `variable_decl`
**Fix**: Use `(?=\\s*=(?!=))` instead of `(?=\\s*=)` to exclude `==`.

### 7. No debouncing on diagnostics

**File**: `lsp.rs`, `handle_did_change`
**Impact**: Every keystroke triggers full parse + analyze + publish. Slow on large files.
**Fix**: Add a 300ms debounce timer before calling `publish_diagnostics`.

### 8. `kyx` files not supported by LSP

**File**: `vscode-ky/src/extension.ts`, line 128
**Fix**: Add `kyx` to `documentSelector`:
```typescript
documentSelector: [{ scheme: 'file', language: 'kl' }, { scheme: 'file', language: 'kyx' }],
```

### 9. `@link` directive only matches at line start

**File**: `ky.tmLanguage.json`, `link_directive`
**Fix**: Remove `^` anchor: `"match": "@link\\s+\"(.*?)\""`.

## How to Test LSP Changes

1. **Build**: `cargo build --release --bin ky`
2. **Start LSP manually**:
   ```bash
   echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}' | ./target/release/ky lsp
   ```
3. **Test with VS Code**: `code --disable-extensions --extensionDevelopmentPath=./vscode-ky`
4. **Test diagnostics**: Open a `.ky` file, check Problems panel
5. **Test semantic tokens**: Open a `.ky` file, check colors match expected scopes

## How to Release a New Extension Version

1. Bump version in `vscode-ky/package.json`
2. Update `vscode-ky/install-extension.sh` TAG
3. Push tag → CI builds `.vsix` and uploads to GitHub Release
4. User runs `install-extension.sh` or downloads from Marketplace

## Files Not to Modify Without Cross-Platform Testing

| File | Risk |
|------|------|
| `install-extension.sh` | Wrong tag breaks install |
| `package.json` | Wrong engine version breaks VS Code compatibility |
| `src/extension.ts` | LSP client configuration affects ALL platforms |
