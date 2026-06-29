#!/bin/bash
set -eu

REPO="IT-KYNERA/KYLE"
VERSION="v0.4.0"

# --- Uninstall mode ---
if [ "${1:-}" = "uninstall" ]; then
  if [ -f /usr/local/bin/kl ]; then
    echo "Removing kl from /usr/local/bin..."
    rm -f /usr/local/bin/kl
    echo "kl uninstalled."
  elif [ -f "$HOME/.kl/bin/kl" ]; then
    echo "Removing kl from $HOME/.kl/bin..."
    rm -f "$HOME/.kl/bin/kl"
    echo "kl uninstalled."
  else
    echo "kl is not installed."
  fi
  exit 0
fi

# --- Install mode ---

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS-$ARCH" in
  linux-aarch64|linux-arm64)
    ASSET="kl-$VERSION-linux-arm64.tar.gz"
    ;;
  linux-x86_64|linux-amd64)
    ASSET="kl-$VERSION-linux-x64.tar.gz"
    ;;
  darwin-arm64|darwin-aarch64)
    ASSET="kl-$VERSION-macos-arm64.tar.gz"
    ;;
  *)
    echo "Unsupported platform: $OS-$ARCH"
    echo "Build from source: https://github.com/$REPO#build-from-source"
    exit 1
    ;;
esac

URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"

# Download
echo "Downloading Kyle $VERSION..."
curl -fsSL "$URL" -o "/tmp/$ASSET"

# Extract
tar -xzf "/tmp/$ASSET" -C /tmp
BIN="/tmp/kl"
chmod +x "$BIN"

# Install
if [ -w /usr/local/bin ]; then
  mv "$BIN" /usr/local/bin/kl
  echo "Installed to /usr/local/bin/kl"
else
  mkdir -p "$HOME/.kl/bin"
  mv "$BIN" "$HOME/.kl/bin/kl"
  echo "Installed to $HOME/.kl/bin/kl"
  echo "Add to your shell profile: export PATH=\"\$HOME/.kl/bin:\$PATH\""
fi

rm -f "/tmp/$ASSET"
echo ""
echo "Kyle $VERSION ready."
kl --version 2>/dev/null || echo "Run 'kl --version' to verify."
echo ""
echo "To uninstall later: curl -fsSL $URL | bash -s uninstall"
