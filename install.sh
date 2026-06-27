#!/usr/bin/env bash
set -euo pipefail

REPO="IT-KYNERA/KYLE"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
info()  { printf "${BLUE}%s${NC}\n" "$*"; }
ok()    { printf "${GREEN}%s${NC}\n" "$*"; }
warn()  { printf "${YELLOW}%s${NC}\n" "$*"; }
error() { printf "${RED}%s${NC}\n" "$*"; exit 1; }

# --- Detect platform ---
ARCH=$(uname -m)
OS=$(uname -s)

case "$OS-$ARCH" in
    Darwin-arm64|Darwin-aarch64) PLATFORM="macos-arm64" ;;
    Darwin-x86_64)               PLATFORM="macos-x64"   ;;
    Linux-aarch64)               PLATFORM="linux-arm64" ;;
    Linux-x86_64)                PLATFORM="linux-x64"   ;;
    *) error "Unsupported platform: $OS-$ARCH. Currently supported: macOS (ARM/x64) and Linux (ARM/x64)." ;;
esac

# --- Parse args ---
LOCAL_DIR=""
VERSION="latest"
if [ $# -ge 1 ] && [ "$1" = "--local" ]; then
    LOCAL_DIR="$2"
elif [ $# -ge 1 ]; then
    VERSION="$1"
fi

if [ -z "$LOCAL_DIR" ]; then
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
            | grep '"tag_name"' | cut -d'"' -f4) || true
        if [ -z "$VERSION" ]; then
            error "Could not determine latest version.\n  Try: curl -fsSL https://raw.githubusercontent.com/$REPO/main/install.sh | sh -s v0.2.0"
        fi
    fi
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/kl-$VERSION-$PLATFORM.tar.gz"
fi

# --- Install dir ---
if [ -w /usr/local/bin ] 2>/dev/null; then
    INSTALL_DIR="/usr/local"
else
    INSTALL_DIR="$HOME/.kl"
    mkdir -p "$INSTALL_DIR/bin" "$INSTALL_DIR/lib"
fi

BIN_DIR="$INSTALL_DIR/bin"
LIB_DIR="$INSTALL_DIR/lib/kl"

# --- Download ---
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

if [ -n "$LOCAL_DIR" ]; then
    info "Installing from $LOCAL_DIR..."
    cp -r "$LOCAL_DIR/" "$TMPDIR/kl/"
else
    info "Downloading kl $VERSION ($PLATFORM)..."
    curl -fsSL "$DOWNLOAD_URL" -o "$TMPDIR/kl.tar.gz"
    tar xzf "$TMPDIR/kl.tar.gz" -C "$TMPDIR"
fi

# --- Install binary ---
mkdir -p "$BIN_DIR" "$LIB_DIR"
cp "$TMPDIR/kl/kl" "$BIN_DIR/kl"
chmod +x "$BIN_DIR/kl"
# Legacy alias
ln -sf "kl" "$BIN_DIR/klc"

# --- Install runtime library ---
cp "$TMPDIR/kl/lib/libklc_runtime.a" "$LIB_DIR/libklc_runtime.a"
chmod 644 "$LIB_DIR/libklc_runtime.a"

# --- Uninstaller ---
UNINSTALL_SCRIPT="$BIN_DIR/kl-uninstall"
cat > "$UNINSTALL_SCRIPT" << 'UNINSTALL_EOF'
#!/usr/bin/env bash
set -euo pipefail
INSTALL_DIR="$(cd "$(dirname "$(readlink -f "$0" 2>/dev/null || echo "$0")")/.." && pwd)"
echo "Removing kl from $INSTALL_DIR..."
rm -f "$INSTALL_DIR/bin/kl" "$INSTALL_DIR/bin/klc" "$INSTALL_DIR/bin/kl-uninstall"
rm -rf "$INSTALL_DIR/lib/kl"
if [ "$INSTALL_DIR" != "/usr/local" ]; then
    rmdir "$INSTALL_DIR/bin" 2>/dev/null || true
    rmdir "$INSTALL_DIR/lib" 2>/dev/null || true
    rmdir "$INSTALL_DIR" 2>/dev/null || true
fi
echo "kl uninstalled."
UNINSTALL_EOF
chmod +x "$UNINSTALL_SCRIPT"

# --- Verify ---
if "$BIN_DIR/kl" --version >/dev/null 2>&1; then
    ok "kl $VERSION installed successfully"
    "$BIN_DIR/kl" --version
    # Verify legacy alias too
    if "$BIN_DIR/klc" --version >/dev/null 2>&1; then
        ok "klc legacy alias also installed"
    fi
else
    error "Installation verification failed"
fi

# --- PATH advice ---
if [ "$INSTALL_DIR" = "$HOME/.kl" ]; then
    echo ""
    echo "Add to your ~/.zshrc (or ~/.bashrc):"
    echo "  export PATH=\"\$HOME/.kl/bin:\$PATH\""
    echo ""
    echo "Or run now:"
    echo "  export PATH=\"\$HOME/.kl/bin:\$PATH\""
fi

ok "Run 'kl --help' to get started."
