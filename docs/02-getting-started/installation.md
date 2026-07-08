# Instalación

> Cómo instalar el compilador Kyle.

## Requisitos

- **LLVM 18.1** (necesario para compilar desde fuente)
- **Rust 1.80+** (para compilar desde fuente)
- **Git** (para clonar el repositorio)

## macOS (Apple Silicon)

```bash
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)
```

## Linux (Ubuntu ARM / x64)

```bash
sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev
```

## Compilar desde fuente

```bash
git clone https://github.com/IT-KYNERA/KYLE.git
cd KYLE
cargo build --release
```

El binario queda en `target/release/ky`.

## Instalar

```bash
# Desde el directorio del repositorio
cp target/release/ky ~/.ky/bin/
cp target/release/libkyc_runtime.a ~/.ky/bin/
```

## Script de instalación

```bash
./install.sh
```

## Verificar

```bash
ky --version    # debe mostrar v0.5.3
ky check --help # debe mostrar ayuda
```

## Variables de entorno

| Variable | Descripción |
|----------|-------------|
| `LLVM_SYS_181_PREFIX` | Ruta a LLVM 18 |
| `MACOSX_DEPLOYMENT_TARGET` | Versión de macOS target |
| `KL_WORKERS` | Workers del thread pool |

## Ver también

- `first-program.md` — Primer programa
- `build.md` — Compilar proyectos
