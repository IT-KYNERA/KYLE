# Editor Support

## VS Code

The Kyle VS Code extension **"Kyle Language Support"** provides full IDE support:

- Syntax highlighting
- LSP integration (diagnostics, completions, go-to-def)
- Debugger (DAP)
- Task provider (run, build, check)
- Test UI
- Code snippets

Install: search "Kyle" in the VS Code marketplace.

## Vim / Neovim

Basic syntax highlighting can be added by associating `.ky` with a Python-like grammar:

```vim
" ~/.vimrc
autocmd BufRead,BufNewFile *.ky set filetype=python
```

For full LSP support, use any LSP-compatible plugin (coc.nvim, built-in LSP):

```vim
" coc.nvim
" :CocInstall coc-lsp
" Configure to run: ky lsp
```

## Helix

Helix editor supports Kyle via its language configuration:

```toml
[[language]]
name = "kyle"
scope = "source.ky"
file-types = ["ky"]
command = "ky lsp"
```

## Language Server Protocol

All editors support Kyle through the LSP:

```bash
ky lsp
```

The LSP communicates over stdin/stdout and provides:
- Diagnostics
- Autocomplete
- Go-to-definition
- Hover information
- Find references
- Rename symbol
