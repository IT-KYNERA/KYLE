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

# Install binary
BIN="/tmp/kl/kl"
chmod +x "$BIN"

if [ -w /usr/local/bin ]; then
  mkdir -p /usr/local/lib/kl
  mv "$BIN" /usr/local/bin/kl
  mv /tmp/kl/lib/libklc_runtime.a /usr/local/lib/kl/
else
  mkdir -p "$HOME/.kl/bin" "$HOME/.kl/lib/kl"
  mv "$BIN" "$HOME/.kl/bin/kl"
  mv /tmp/kl/lib/libklc_runtime.a "$HOME/.kl/lib/kl/"
fi

rm -rf "/tmp/$ASSET" "/tmp/kl"

echo ""
echo "Kyle $VERSION installed."
echo "Create a project:  kl new myapp"
echo "Run it:           kl run myapp/src/main.kl"