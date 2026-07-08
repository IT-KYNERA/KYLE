# Package Manager

**Status:** Available

## Commands

| Command | Description |
|---------|-------------|
| `ky add <dep>[@<ver>]` | Add a dependency |
| `ky remove <dep>` | Remove a dependency |
| `ky update` | Update lock file |
| `ky publish` | Publish current package |
| `ky login` | Login to package registry |

## How it works

1. Dependencies are declared in `ky.toml`
2. `ky add` resolves versions, downloads packages, and writes `ky.lock`
3. During compilation, packages' `src/` directories are added to the search path

## Registry

The default registry is `https://registry.kyle-lang.org/v1`. Override with `KL_REGISTRY` environment variable.

## Package format

```
package-name-1.0.0/
├── ky.toml
├── src/
│   ├── lib.ky
│   └── ...
├── tests/
├── README.md
└── LICENSE
```
