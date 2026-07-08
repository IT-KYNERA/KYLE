# Installation

> How instalar compiler Kyle.

## Requirements

- **LLVM 18.1** (necesario for compile from source)
- **Rust 1.80+** (for compile from source)
- **Git** (for clonar repository)

## macOS (Apple Silicon)

```bash
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)
```

## Linux (Ubuntu ARM / x64)

```bash
sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev
```

## Compilar from source

```bash
git clone https://github.com/IT-KYNERA/KYLE.git
cd KYLE
cargo build --release
```

El binary queda en `target/release/ky`.

## Instalar

```bash
# Desde directory del repository
cp target/release/ky ~/.ky/bin/
cp target/release/libkyc_runtime.a ~/.ky/bin/
```

## Script de installation

```bash
./install.sh
```

## Verificar

```bash
ky --version # must mostrar v0.5.3
ky check --help # must mostrar ayuda
```

## Variablis de entorno

| Variable | Description |
|----------|-------------|
| `LLVM_SYS_181_PREFIX` | Path a LLVM 18 |
| `MACOSX_DEPLOYMENT_TARGET` | Version de macOS target |
| `KL_WORKERS` | Workers del thread pool |

## See also

- `first-program.md` — Primer program
- `build.md` — Compilar proyectos
