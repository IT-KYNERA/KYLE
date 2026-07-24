#!/bin/bash
set -eu

REPO="IT-KYNERA/KYLE"
# Default a latest release; override con KY_VERSION=vX.Y.Z para pin versión
VERSION="${KY_VERSION:-v0.8.6}"

# ─── Functions ──────────────────────────────────────────────

usage() {
    echo "Usage: curl -fsSL https://raw.githubusercontent.com/$REPO/main/install.sh | sh"
    echo ""
    echo "Environment variables:"
    echo "  KY_VERSION=v0.8.6     Version to install (default: latest)"
    echo "  KY_PREFIX=/custom/path Install directory (default: ~/.ky or /usr/local)"
    echo ""
    echo "  install.sh uninstall   Remove Kyle from the system"
    exit 0
}

detect_platform() {
    local os arch
    case "$(uname -s)" in
        Darwin) os="macos" ;;
        Linux)  os="linux" ;;
        MINGW*|MSYS*|CYGWIN*)
            echo "Error: este script es para macOS/Linux."
            echo "En Windows, usa PowerShell:"
            echo "  iwr -Uri \"https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.ps1\" | iex"
            exit 1 ;;
        *)      echo "Error: unsupported OS ($(uname -s))"; exit 1 ;;
    esac
    case "$(uname -m)" in
        arm64|aarch64) arch="arm64" ;;
        x86_64|amd64)  arch="x64" ;;
        *)      echo "Error: unsupported architecture ($(uname -m))"; exit 1 ;;
    esac
    echo "${os}-${arch}"
}

cleanup() {
    rm -rf "/tmp/ky-install" 2>/dev/null || true
}

# ─── Uninstall ──────────────────────────────────────────────

if [ "${1:-}" = "uninstall" ]; then
    echo "Removing Kyle..."
    # Remove binary from all known locations
    for f in /usr/local/bin/ky "$HOME/.ky/bin/ky" "$HOME/.kl/bin/ky"; do
        if [ -f "$f" ]; then rm -f "$f" && echo "  Removed $f"; fi
    done
    # Remove runtime library from all known locations
    for f in /usr/local/lib/libkyc_runtime.a /usr/local/lib/ky/libkyc_runtime.a "$HOME/.ky/lib/libkyc_runtime.a" "$HOME/.kl/lib/libkyc_runtime.a"; do
        if [ -f "$f" ]; then rm -f "$f" && echo "  Removed $f"; fi
    done
    # Remove install directories
    for d in "$HOME/.ky" "$HOME/.kl" /usr/local/lib/ky; do
        if [ -d "$d" ]; then rm -rf "$d" && echo "  Removed $d"; fi
    done
    # Remove PATH additions from shell configs
    for rc in "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.profile"; do
        if [ -f "$rc" ]; then
            sed -i '' '/\.ky\/bin/d' "$rc" 2>/dev/null || true  # macOS
            sed -i '/\.ky\/bin/d' "$rc" 2>/dev/null || true     # Linux
        fi
    done
    echo "ky uninstalled."
    echo "Close and reopen your terminal, or run: source ~/.zshrc (or ~/.bashrc)"
    exit 0
fi

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    usage
fi

# ─── Detect platform ────────────────────────────────────────

PLATFORM=$(detect_platform)
echo "Detected: $PLATFORM"

BUNDLE="ky-${PLATFORM}.tar.gz"
if [ "$VERSION" = "latest" ]; then
    BUNDLE_URL="https://github.com/$REPO/releases/latest/download/$BUNDLE"
    SHA256_URL="$BUNDLE_URL.sha256"
else
    BUNDLE_URL="https://github.com/$REPO/releases/download/$VERSION/$BUNDLE"
    SHA256_URL="$BUNDLE_URL.sha256"
fi

# ─── Download ───────────────────────────────────────────────

cleanup
mkdir -p /tmp/ky-install
cd /tmp/ky-install

echo "Downloading Kyle $VERSION for $PLATFORM..."
echo "  $BUNDLE_URL"

curl -fsSL "$BUNDLE_URL" -o "$BUNDLE" || {
    echo "Error: failed to download $BUNDLE"
    echo "Check that $VERSION exists at:"
    echo "  https://github.com/$REPO/releases"
    exit 1
}

# ─── Verify checksum ────────────────────────────────────────

if curl -fsSL "$SHA256_URL" -o "${BUNDLE}.sha256" 2>/dev/null; then
    echo "Verifying checksum..."
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 -c "${BUNDLE}.sha256" || {
            echo "Error: checksum verification failed"
            exit 1
        }
    elif command -v sha256sum >/dev/null 2>&1; then
        sha256sum -c "${BUNDLE}.sha256" || {
            echo "Error: checksum verification failed"
            exit 1
        }
    fi
else
    echo "Warning: no checksum file found, skipping verification"
fi

# ─── Pre-install cleanup ────────────────────────────────────

# Remove legacy runtime from old /ky/ subdirectory
if [ -f /usr/local/lib/ky/libkyc_runtime.a ]; then
    rm -f /usr/local/lib/ky/libkyc_runtime.a 2>/dev/null
    rmdir /usr/local/lib/ky 2>/dev/null || true
fi

# ─── Extract ────────────────────────────────────────────────

tar xzf "$BUNDLE"

if [ ! -f "ky" ]; then
    echo "Error: 'ky' not found in archive"
    ls -la
    exit 1
fi
if [ ! -f "libkyc_runtime.a" ]; then
    echo "Warning: libkyc_runtime.a not found in archive"
fi

# macOS: ad-hoc sign binary + bundled dylibs to avoid Gatekeeper kills
if [ "$(uname -s)" = "Darwin" ]; then
    echo "  Signing binary for macOS..."
    xattr -d com.apple.quarantine ky 2>/dev/null || true
    # Sign bundled dylibs (extracted from lib/ subdirectory)
    if [ -d "lib" ]; then
        for f in lib/*.dylib; do
            [ -f "$f" ] && codesign -f -s - "$f" 2>/dev/null || true
        done
    fi
    codesign -f -s - ky 2>/dev/null && echo "  Code signature applied"
fi

chmod +x ky

# ─── Install ─────────────────────────────────────────────────

INSTALL_TO_USR=false
if [ "${KY_PREFIX:-}" = "" ]; then
    if [ -w /usr/local/bin ]; then
        INSTALL_TO_USR=true
    else
        # Try with sudo
        if command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
            INSTALL_TO_USR=true
        fi
    fi
fi

if [ "$INSTALL_TO_USR" = true ] && [ "${KY_PREFIX:-}" = "" ]; then
    echo "Installing to /usr/local..."
    if [ "$(id -u)" -eq 0 ]; then
        mkdir -p /usr/local/bin /usr/local/lib
        cp ky /usr/local/bin/ky
        if [ -f libkyc_runtime.a ]; then
            cp libkyc_runtime.a /usr/local/lib/libkyc_runtime.a
        fi
        [ -d lib ] && cp lib/*.dylib /usr/local/lib/ 2>/dev/null || true
    else
        sudo mkdir -p /usr/local/bin /usr/local/lib
        sudo cp ky /usr/local/bin/ky
        if [ -f libkyc_runtime.a ]; then
            sudo cp libkyc_runtime.a /usr/local/lib/libkyc_runtime.a
        fi
        [ -d lib ] && sudo cp lib/*.dylib /usr/local/lib/ 2>/dev/null || true
    fi
    INSTALL_DIR="/usr/local/bin"
    KY_PREFIX="/usr/local"

elif [ -n "${KY_PREFIX:-}" ]; then
    echo "Installing to $KY_PREFIX..."
    mkdir -p "$KY_PREFIX/bin" "$KY_PREFIX/lib"
    cp ky "$KY_PREFIX/bin/ky"
    if [ -f libkyc_runtime.a ]; then
        cp libkyc_runtime.a "$KY_PREFIX/lib/libkyc_runtime.a"
    fi
    if [ -d lib ]; then
        cp lib/*.dylib "$KY_PREFIX/lib/" 2>/dev/null || true
    fi
    INSTALL_DIR="$KY_PREFIX/bin"
else
    echo "Installing to $HOME/.ky..."
    mkdir -p "$HOME/.ky/bin" "$HOME/.ky/lib"
    cp ky "$HOME/.ky/bin/ky"
    if [ -f libkyc_runtime.a ]; then
        cp libkyc_runtime.a "$HOME/.ky/lib/libkyc_runtime.a"
    fi
    if [ -d lib ]; then
        cp lib/*.dylib "$HOME/.ky/lib/" 2>/dev/null || true
    fi
    INSTALL_DIR="$HOME/.ky/bin"
    KY_PREFIX="$HOME/.ky"
fi

# ─── Add to PATH ─────────────────────────────────────────────

SHELL_NAME=$(basename "${SHELL:-}")
case "$SHELL_NAME" in
    zsh)  SHELL_CONFIG="$HOME/.zshrc" ;;
    bash) SHELL_CONFIG="$HOME/.bashrc" ;;
    *)    SHELL_CONFIG="" ;;
esac

if [ -n "$SHELL_CONFIG" ] && [ -w "$SHELL_CONFIG" ]; then
    if ! grep -q "$INSTALL_DIR" "$SHELL_CONFIG" 2>/dev/null; then
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_CONFIG"
        echo "  Added $INSTALL_DIR to PATH in $SHELL_CONFIG"
    else
        echo "  PATH already configured in $SHELL_CONFIG"
    fi
elif [ -n "$SHELL_CONFIG" ]; then
    echo "  Warning: $SHELL_CONFIG not writable, add to PATH manually:"
    echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
fi

export PATH="$INSTALL_DIR:$PATH"

# ─── Verify ──────────────────────────────────────────────────

echo ""
if ky --version 2>/dev/null; then
    echo ""
    echo "✅ Kyle $VERSION installed successfully!"
    echo ""
    echo "  Binary:  $INSTALL_DIR/ky"
    if [ -f "$KY_PREFIX/lib/libkyc_runtime.a" ] || [ -f "$KY_PREFIX/lib/ky/libkyc_runtime.a" ]; then
        echo "  Runtime: installed"
    fi
    echo ""
    echo "  Use now:     source ${SHELL_CONFIG:-~/.profile} && ky -v"
    echo "  Or (faster): export PATH=\"$INSTALL_DIR:\$PATH\" && ky -v"
    echo "  Try:         ky run examples/hello.ky"
else
    echo "⚠️  Installation completed but 'ky --version' failed."
    echo "   Check that $INSTALL_DIR is in your PATH."
fi

cleanup
