# IDE Support

> Soporte para editores e IDEs en Kyle.

## VS Code

Kyle tiene una extensión oficial para VS Code:

```bash
# Instalar desde VS Code marketplace
# O manualmente:
code --install-extension ky-0.5.3.vsix
```

### Características

| Feature | Detalle |
|---------|---------|
| Syntax highlighting | Resaltado de sintaxis `.ky` |
| LSP (Language Server) | Diagnostics, completado, hover |
| Debugging | DAP integration |
| Test runner | UI para `#[test]` |

### Configuración

```json
{
    "ky.kycPath": "/usr/local/bin/ky",
    "ky.semanticHighlighting": true
}
```

## Otros editores

| Editor | Soporte |
|--------|---------|
| Vim/Neovim | Syntax highlighting básico vía `vim-ky` |
| Emacs | Syntax highlighting básico |
| IntelliJ | Plugin comunitario (terceros) |

## LSP

Kyle implementa el Language Server Protocol para integración con cualquier editor
que lo soporte:

```bash
ky lsp                       # inicia el servidor LSP
```

## Ver también

- `07-tools/language-server.md` — Documentación del LSP
- `07-tools/vscode.md` — Extensión VS Code
