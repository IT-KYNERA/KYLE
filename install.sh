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

# --- Detect OS ---
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS-$ARCH" in
  linux-aarch64|linux-arm64|darwin-arm64|darwin-aarch64)
    ASSET="ky-$VERSION"
    ;;
  linux-x86_64|linux-amd64)
    ASSET="ky-$VERSION"
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
chmod +x "/tmp/$ASSET"

# Install
if [ -w /usr/local/bin ]; then
  mv "/tmp/$ASSET" /usr/local/bin/ky
else
  mkdir -p "$HOME/.ky/bin"
  mv "/tmp/$ASSET" "$HOME/.ky/bin/ky"
fi

echo ""
echo "Kyle $VERSION installed."
echo "Add to PATH: export PATH=\"\$HOME/.ky/bin:\$PATH\""
echo "Create a project:  ky new myapp"
echo "Run it:           ky run myapp/src/main.ky"
