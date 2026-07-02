#!/bin/bash
set -eu

REPO="IT-KYNERA/KYLE"
VERSION="v0.5.0"

# --- Uninstall mode ---
if [ "${1:-}" = "uninstall" ]; then
  echo "Removing Kyle..."
  for f in /usr/local/bin/ky "$HOME/.ky/bin/ky" "$HOME/.kl/bin/ky"; do
    if [ -f "$f" ]; then rm -f "$f" && echo "  Removed $f"; fi
  done
  for d in "$HOME/.ky" "$HOME/.kl"; do
    if [ -d "$d" ]; then rm -rf "$d" && echo "  Removed $d"; fi
  done
  # Clean PATH from shell config
  for rc in "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.profile"; do
    if [ -f "$rc" ]; then
      sed -i '' '/\.ky\/bin/d' "$rc" 2>/dev/null || true
      sed -i '/\.ky\/bin/d' "$rc" 2>/dev/null || true
    fi
  done
  echo "ky uninstalled."
  exit 0
fi

# --- Detect platform ---
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS-$ARCH" in
  linux-aarch64|linux-arm64|darwin-arm64|darwin-aarch64|linux-x86_64|linux-amd64)
    ASSET="ky-$VERSION"
    ;;
  *)
    echo "Unsupported: $OS-$ARCH. Build from source: https://github.com/$REPO"
    exit 1
    ;;
esac

# --- Download ---
URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"
echo "Downloading Kyle $VERSION..."
curl -fsSL "$URL" -o "/tmp/$ASSET"
chmod +x "/tmp/$ASSET"

# --- Install ---
if [ -w /usr/local/bin ]; then
  mkdir -p /usr/local/lib/ky
  mv "/tmp/$ASSET" /usr/local/bin/ky
  INSTALL_DIR="/usr/local/bin"
else
  mkdir -p "$HOME/.ky/bin"
  mv "/tmp/$ASSET" "$HOME/.ky/bin/ky"
  INSTALL_DIR="$HOME/.ky/bin"
fi

# --- Add to PATH automatically ---
SHELL_NAME=$(basename "${SHELL:-}")
case "$SHELL_NAME" in
  zsh) SHELL_CONFIG="$HOME/.zshrc" ;;
  bash) SHELL_CONFIG="$HOME/.bashrc" ;;
  *) SHELL_CONFIG="" ;;
esac
if [ -n "$SHELL_CONFIG" ] && [ -f "$SHELL_CONFIG" ]; then
  if ! grep -q "$INSTALL_DIR" "$SHELL_CONFIG" 2>/dev/null; then
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_CONFIG"
    echo "  Added $INSTALL_DIR to PATH in $SHELL_CONFIG"
  fi
fi

# Make ky available immediately (works for direct run, for pipe: source ~/.zshrc)
export PATH="$INSTALL_DIR:$PATH"

echo ""
echo "Kyle $VERSION installed."
echo "To use now:  source ~/.zshrc"
echo "Or open a new terminal, then:  ky -v"