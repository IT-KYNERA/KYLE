# Kyle Language Support for Visual Studio Code

Syntax highlighting, LSP integration, snippets, debugging, and language support for the [Kyle programming language](https://github.com/IT-KYNERA/KYLE).

## Features

- **Syntax Highlighting** — Full syntax highlighting for `.kl` files using TextMate grammar
- **Language Server Protocol** — Diagnostics, completions, go-to-definition, hover, inlay hints, code lens via `kl lsp`
- **Snippets** — 35+ snippets for Kyle constructs (`fn`, `final class`, `match`, etc.)
- **Testing UI** — Discover and run `#[test]` functions from VS Code's Testing panel
- **Tasks** — Run, build, check, and test commands via VS Code tasks
- **Debugging** — Launch `.kl` files with breakpoint support (DAP)
- **Format on Save** — Auto-format via `kl fmt` on save
- **Color Theme** — "Kyle Pastel" dark theme included

## Requirements

- **Kyle compiler** (`kl` / `klc`) must be installed and available in PATH, or configured via `kl.klcPath`
- **LLVM 18** runtime libraries for compiled binaries

## Extension Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `kl.klcPath` | `"kl"` | Path to the Kyle compiler binary |
| `kl.semanticHighlighting` | `true` | Enable semantic highlighting |

## Commands

| Command | Description |
|---------|-------------|
| `KL: Run current file` | Compile and run the active `.kl` file |
| `KL: Build current file` | Compile to native binary |
| `KL: Type-check current file` | Type-check without codegen |
| `KL: Run tests in current file` | Run `#[test]` functions |
| `KL: Run specific test` | Run a specific test function |

## Known Issues

- Full step-through debugging requires KL runtime debugger support (in development)
- Breakpoints work at the line level; variable inspection is limited

## Release Notes

See [CHANGELOG.md](CHANGELOG.md) for version history.
