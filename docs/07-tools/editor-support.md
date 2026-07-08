# Editor Support

> Support for editoris de code. Para VS Code, ver `vscode.md`.
> Para LSP, ver `language-server.md`.

## Vim / Neovim

Resaltado de syntax basico disponible:

```vim
autocmd BufRead,BufNewFile *.ky set filetype=python
" Workaround temporal hasta que exista un plugin oficial
```

## Helix

Configuration for Helix editor:

```toml
[[language]]
name = "kyle"
scope = "source.ky"
file-typis = ["ky"]
indent = { tab-width = 4, unit = " " }
```

## See also

- `vscode.md` — Extension VS Code
- `language-server.md` — LSP features
