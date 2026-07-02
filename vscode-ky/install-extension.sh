#!/bin/bash
set -eu

REPO="IT-KYNERA/KYLE"
TAG="v0.5.0"
TMP_DIR="/tmp/kl-extension-$$"

# --- Uninstall mode ---
if [ "${1:-}" = "uninstall" ]; then
  echo "Removing Kyle VS Code extension..."
  if command -v code &>/dev/null; then
    code --uninstall-extension kynera.ky 2>/dev/null || true
    echo "Extension uninstalled."
  else
    echo "VS Code 'code' CLI not found — remove manually."
  fi
  exit 0
fi

echo "==> Kyle VS Code Extension — Installer"
echo ""

# --- Find VS Code CLI ---
CODE_CLI=""
if command -v code &>/dev/null; then
  CODE_CLI="code"
elif [ -f "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code" ]; then
  CODE_CLI="/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code"
elif [ -f "$HOME/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code" ]; then
  CODE_CLI="$HOME/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code"
elif [ -f "/usr/share/code/code" ]; then
  CODE_CLI="/usr/share/code/code"
elif [ -f "/snap/bin/code" ]; then
  CODE_CLI="/snap/bin/code"
fi

if [ -z "$CODE_CLI" ]; then
  echo "ERROR: VS Code 'code' CLI not found."
  echo "Open VS Code → Cmd+Shift+P → 'Shell Command: Install code command in PATH'"
  exit 1
fi

# --- Download pre-built VSIX from GitHub Releases ---
echo "==> Downloading pre-built VSIX (no Node.js needed)..."
mkdir -p "$TMP_DIR"
cd "$TMP_DIR"

VSIX_URL="https://github.com/$REPO/releases/download/$TAG/kl-0.2.2.vsix"
VSIX_FILE="ky-extension.vsix"

curl -fsSL "$VSIX_URL" -o "$VSIX_FILE" || {
  echo "ERROR: failed to download VSIX from $VSIX_URL"
  echo "Check the latest release at: https://github.com/$REPO/releases"
  exit 1
}

echo "==> Installing in VS Code..."
"$CODE_CLI" --install-extension "$VSIX_FILE" --force

cd /
rm -rf "$TMP_DIR"

echo ""
echo "✅ Kyle VS Code extension installed!"
echo ""
echo "Verify:  $CODE_CLI --list-extensions | grep ky"
echo ""
echo "Uninstall: curl -fsSL https://raw.githubusercontent.com/$REPO/main/vscode-ky/install-extension.sh | sh -s uninstall"
