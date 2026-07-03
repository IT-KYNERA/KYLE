# Installing Kyle

## Prerequisites

- LLVM 18.1
- Rust 1.80+ (for building from source)

## macOS (Apple Silicon)

```bash
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)
```

## Linux (Ubuntu ARM)

```bash
sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev
```

## Building

```bash
git clone https://github.com/IT-KYNERA/KYLE.git
cd KYLE
cargo build --release
```

The binary is at `target/release/ky`.

## Installing

```bash
cargo install --path .
```

Or use the install script:

```bash
./install.sh
```
