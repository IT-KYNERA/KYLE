#!/bin/bash
set -eu

REPO="IT-KYNERA/KYLE"
VERSION="v0.5.2"

# --- Uninstall mode ---
if [ "${1:-}" = "uninstall" ]; then
  echo "Removing Kyle..."
  for f in /usr/local/bin/ky "$HOME/.ky/bin/ky" "$HOME/.kl/bin/ky"; do
    if [ -f "$f" ]; then rm -f "$f" && echo "  Removed $f"; fi
  done
  for d in "$HOME/.ky" "$HOME/.kl"; do
    if [ -d "$d" ]; then rm -rf "$d" && echo "  Removed $d"; fi
  done
  for rc in "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.profile"; do
    if [ -f "$rc" ]; then
      sed -i '' '/\.ky\/bin/d' "$rc" 2>/dev/null || true
      sed -i '/\.ky\/bin/d' "$rc" 2>/dev/null || true
    fi
  done
  echo "ky uninstalled."
  exit 0
fi

echo "Downloading Kyle $VERSION..."

# Download compressed binary from GitHub Releases
curl -fsSL "https://github.com/$REPO/releases/download/$VERSION/ky.gz" -o "/tmp/ky.gz"
gunzip -f "/tmp/ky.gz"
chmod +x "/tmp/ky"

# --- Install ---
if [ -w /usr/local/bin ]; then
  mkdir -p /usr/local/lib/ky
  mv /tmp/ky /usr/local/bin/ky
  INSTALL_DIR="/usr/local/bin"
else
  mkdir -p "$HOME/.ky/bin"
  mv /tmp/ky "$HOME/.ky/bin/ky"
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

export PATH="$INSTALL_DIR:$PATH"

echo ""
echo "✅ Kyle $VERSION installed. PATH added to $SHELL_CONFIG"
echo ""
echo "  Use now:     source ~/.zshrc && ky -v"
echo "  Or (faster): export PATH=\"$INSTALL_DIR:\$PATH\" && ky -v"
