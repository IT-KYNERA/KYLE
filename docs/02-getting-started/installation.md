# Installation

> How to install the Kyle compiler on any platform.

## Requirements

- **LLVM 18.1** (required to compile from source)
- **Rust 1.80+** (required to compile from source)
- **Git** (to clone the repository)

## Quick Install (one command)

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh | sh
```

### Windows (PowerShell)

```powershell
iwr -Uri "https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.ps1" | iex
```

Both scripts:
1. Detect OS + architecture automatically
2. Download the correct bundle from GitHub Releases
3. Verify SHA-256 checksum
4. Install `ky` + `libkyc_runtime.a`
5. Configure PATH automatically

---

## Install LLVM 18 (for building from source)

### macOS

```bash
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)
```

### Linux (Debian/Ubuntu)

```bash
sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev
```

### Windows

**Option A — Chocolatey (recommended):**
```powershell
choco install llvm --version=18.1.8
$env:LLVM_SYS_181_PREFIX = "C:\Program Files\LLVM"
```

**Option B — Manual download:**
1. Download: https://github.com/llvm/llvm-project/releases/tag/llvmorg-18.1.8
2. Run `LLVM-18.1.8-win64.exe`
3. Set environment variable:
   ```powershell
   $env:LLVM_SYS_181_PREFIX = "C:\Program Files\LLVM"
   ```

**Option C — Portable (no admin):**
```powershell
Invoke-WebRequest -Uri "https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/LLVM-18.1.8-win64.zip" -OutFile "$env:TEMP\llvm-18.zip"
Expand-Archive -Path "$env:TEMP\llvm-18.zip" -DestinationPath "$env:USERPROFILE\llvm-18"
$env:LLVM_SYS_181_PREFIX = "$env:USERPROFILE\llvm-18\LLVM-18.1.8-win64"
```

---

## Build from Source

```bash
git clone https://github.com/IT-KYNERA/KYLE.git
cd KYLE

# Build compiler + runtime
cargo build --release --bin ky
cargo build --release -p kyc_runtime

# Binary is at:
#   macOS/Linux: target/release/ky
#   Windows:     target/release/ky.exe
# Runtime:
#   target/release/libkyc_runtime.a
```

### Install locally after building

```bash
# macOS / Linux
cp target/release/ky ~/.ky/bin/
cp target/release/libkyc_runtime.a ~/.ky/lib/

# Windows (PowerShell)
Copy-Item target/release/ky.exe "$env:USERPROFILE\.ky\bin\"
Copy-Item target/release/libkyc_runtime.a "$env:USERPROFILE\.ky\lib\"
```

---

## Verify Installation

```bash
ky --version    # Should show v0.6.0
ky check --help # Should show help
ky run examples/hello.ky  # Should print "Hello, World!"
```

---

## Environment Variables

| Variable | Platform | Description |
|----------|----------|-------------|
| `LLVM_SYS_181_PREFIX` | All | Path to LLVM 18 installation |
| `MACOSX_DEPLOYMENT_TARGET` | macOS | Deployment target version |
| `KL_WORKERS` | All | Thread pool workers (default: CPU count) |
| `KY_VERSION` | All | Version to install (for install scripts) |
| `KY_PREFIX` | All | Custom install directory |

---

## See also

- `first-program.md` — First program
- `build.md` — Building projects
- `testing.md` — Writing and running tests
- `package-manager.md` — Package manager
