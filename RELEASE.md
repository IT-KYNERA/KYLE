# Release Guide — Kyle Compiler

## Prerrequisitos

```bash
# Ubuntu ARM (la máquina actual)
sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev

# macOS ARM (Apple Silicon)
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)
```

## 1. Compilar el binario release

```bash
# Desde la raíz del proyecto
cargo build --release --bin kl
```

Esto genera: `target/release/kl`

## 2. Verificar que funciona

```bash
./target/release/kl build --release examples/src/main.kl
./target/release/kl run examples/src/main.kl
```

## URL del release actual

**v0.4.0:** https://github.com/IT-KYNERA/KYLE/releases/tag/v0.4.0

**Descargas directas:**
- Linux ARM64: `kl-v0.4.0-linux-arm64.tar.gz`
- macOS ARM64: `kl-v0.4.0-macos-arm64.tar.gz`

## 3. Subir release a GitHub

### Opción A: Manual (rápida)

```bash
# 1. Crear tag
git tag -a v0.4.0 -m "v0.4.0 — Kyle language compiler"
git push origin v0.4.0

# 2. Ir a GitHub → Releases → Create Release
#    Tag: v0.4.0
#    Title: "Kyle v0.4.0"
#    Description: (copiar de CHANGELOG.md o escribir resumen)

# 3. Subir el binario comprimido
cd target/release
tar -czf kl-v0.4.0-linux-arm64.tar.gz kl
# Subir este archivo al release en GitHub

# 4. Si estás en macOS ARM:
tar -czf kl-v0.4.0-macos-arm64.tar.gz kl
# Subir también
```

### Opción B: Con GitHub Actions (automática)

Crear `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-24.04-arm, macos-15-arm64]
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4

      - name: Install LLVM 18 (Linux ARM)
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y llvm-18-dev libpolly-18-dev libzstd-dev
          echo "LLVM_SYS_181_PREFIX=/usr/lib/llvm-18" >> $GITHUB_ENV

      - name: Install LLVM 18 (macOS ARM)
        if: runner.os == 'macOS'
        run: |
          brew install llvm@18
          echo "LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)" >> $GITHUB_ENV

      - name: Build
        run: cargo build --release --bin kl

      - name: Package
        run: |
          cd target/release
          tar -czf ../../kl-${{ github.ref_name }}-${{ runner.os }}.tar.gz kl

      - name: Upload
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ github.event.release.upload_url }}
          asset_path: ./kl-${{ github.ref_name }}-${{ runner.os }}.tar.gz
          asset_name: kl-${{ github.ref_name }}-${{ runner.os }}.tar.gz
          asset_content_type: application/gzip
```

## 4. Instalar localmente

```bash
# Descomprimir
tar -xzf kl-v0.4.0-linux-arm64.tar.gz

# Instalar en el sistema
sudo cp kl /usr/local/bin/kl

# Verificar
kl --help
kl build --release mi_proyecto/src/main.kl
kl run mi_proyecto/src/main.kl
```

## 5. Probar el compilador instalado

```bash
# Crear un proyecto de prueba
mkdir -p test_project/src
cat > test_project/src/main.kl << 'EOF'
fn main() i32:
    println("Hello from Kyle!")
    0
EOF

# Compilar y ejecutar
kl build --release test_project/src/main.kl
./target/release/main
```
