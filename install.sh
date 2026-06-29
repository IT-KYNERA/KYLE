#!/usr/bin/env bash
set -euo pipefail

REPO="IT-KYNERA/KYLE"
VERSION="v0.4.0"

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

echo "Downloading Kyle $VERSION for $OS-$ARCH..."
curl -fsSL "$URL" -o "/tmp/$ASSET"

echo "Extracting..."
tar -xzf "/tmp/$ASSET" -C /tmp

echo "Installing..."
BIN="/tmp/kl"
if [ -w /usr/local/bin ]; then
  mv "$BIN" /usr/local/bin/kl
  echo "Installed to /usr/local/bin/kl"
else
  mkdir -p "$HOME/.kl/bin"
  mv "$BIN" "$HOME/.kl/bin/kl"
  echo "Installed to $HOME/.kl/bin/kl"
  echo "Add to PATH: export PATH=\"\$HOME/.kl/bin:\$PATH\""
fi

rm -f "/tmp/$ASSET"
echo ""
echo "Kyle $VERSION installed successfully!"
kl --version
