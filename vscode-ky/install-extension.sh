#!/bin/bash
set -eu

REPO="IT-KYNERA/KYLE"
BRANCH="main"
TAG="v0.6.2"
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

mkdir -p "$TMP_DIR"
cd "$TMP_DIR"

# --- Node.js check: build from source if >=20, otherwise download pre-built ---
NODE_VERSION=$(node -v 2>/dev/null | sed 's/v//' | cut -d. -f1 || echo "0")
if [ "$NODE_VERSION" -ge 20 ] 2>/dev/null && command -v npm &>/dev/null; then
  echo "==> Building from source (Node.js $NODE_VERSION detected)..."
  ZIP_URL="https://github.com/$REPO/archive/$BRANCH.zip"
  curl -fsSL "$ZIP_URL" -o repo.zip
  unzip -q repo.zip -d extracted
  mv extracted/KYLE-"$BRANCH"/vscode-ky ./vscode-ky 2>/dev/null || \
    mv extracted/KYLE-main/vscode-ky ./vscode-ky 2>/dev/null
  rm -rf repo.zip extracted

  cd vscode-ky
  npm install --silent 2>/dev/null
  VSIX_FILE="ky-extension.vsix"
  npx @vscode/vsce package --out "$VSIX_FILE" 2>/dev/null || {
    npm install -g @vscode/vsce 2>/dev/null || true
    npx @vscode/vsce package --out "$VSIX_FILE"
  }
else
  echo "==> Downloading pre-built VSIX (Node.js $NODE_VERSION < 20)..."
  VSIX_URL="https://github.com/$REPO/releases/download/$TAG/ky-extension.vsix"
  VSIX_FILE="ky-extension.vsix"
  curl -fsSL "$VSIX_URL" -o "$VSIX_FILE"
fi

if [ ! -f "$VSIX_FILE" ]; then
  echo "ERROR: failed to get VSIX."
  echo "Install Node.js 20+ and try again, or download from:"
  echo "  https://github.com/$REPO/releases"
  exit 1
fi

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
