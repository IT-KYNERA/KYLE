#!/bin/bash
set -eu

REPO="IT-KYNERA/KYLE"
VERSION="v0.5.0"

# --- Uninstall mode ---
if [ "${1:-}" = "uninstall" ]; then
  if [ -f /usr/local/bin/ky ]; then
    echo "Removing ky from /usr/local/bin..."
    rm -f /usr/local/bin/ky /usr/local/lib/ky/libkyc_runtime.a
    echo "ky uninstalled."
  elif [ -f "$HOME/.ky/bin/ky" ]; then
    echo "Removing ky from $HOME/.ky/bin..."
    rm -rf "$HOME/.ky"
    echo "ky uninstalled."
  else
    echo "ky is not installed."
  fi
  exit 0
fi

# --- Install mode ---

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS-$ARCH" in
  linux-aarch64|linux-arm64)
    ASSET="ky-$VERSION-linux-arm64.tar.gz"
    ;;
  linux-x86_64|linux-amd64)
    ASSET="ky-$VERSION-linux-x64.tar.gz"
    ;;
  darwin-arm64|darwin-aarch64)
    ASSET="ky-$VERSION-macos-arm64.tar.gz"
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

# Install binary
BIN="/tmp/ky/ky"
chmod +x "$BIN"

if [ -w /usr/local/bin ]; then
  mkdir -p /usr/local/lib/ky
  mv "$BIN" /usr/local/bin/ky
  mv /tmp/ky/lib/libkyc_runtime.a /usr/local/lib/ky/
else
  mkdir -p "$HOME/.ky/bin" "$HOME/.ky/lib/ky"
  mv "$BIN" "$HOME/.ky/bin/ky"
  mv /tmp/ky/lib/libkyc_runtime.a "$HOME/.ky/lib/ky/"
fi

rm -rf "/tmp/$ASSET" "/tmp/ky"

echo ""
echo "Kyle $VERSION installed."
echo "Add to PATH: export PATH=\"\$HOME/.ky/bin:\$PATH\""
echo "Create a project:  ky new myapp"
echo "Run it:           ky run myapp/src/main.ky"