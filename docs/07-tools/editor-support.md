# Editor Support

> Soporte para editores de código. Para VS Code, ver `vscode.md`.
> Para LSP, ver `language-server.md`.

## Vim / Neovim

Resaltado de sintaxis básico disponible:

```vim
autocmd BufRead,BufNewFile *.ky set filetype=python
" Workaround temporal hasta que exista un plugin oficial
```

## Helix

Configuración para Helix editor:

```toml
[[language]]
name = "kyle"
scope = "source.ky"
file-types = ["ky"]
indent = { tab-width = 4, unit = "    " }
```

## Ver también

- `vscode.md` — Extensión VS Code
- `language-server.md` — LSP features
