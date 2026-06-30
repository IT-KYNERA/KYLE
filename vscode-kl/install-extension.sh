#!/bin/bash
set -eu

REPO="IT-KYNERA/KYLE"
BRANCH="main"
TMP_DIR="/tmp/kl-extension-$$"

# --- Uninstall mode ---
if [ "${1:-}" = "uninstall" ]; then
  echo "Removing Kyle VS Code extension..."
  if command -v code &>/dev/null; then
    code --uninstall-extension kynera.kl 2>/dev/null || true
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

# --- Check prerequisites ---
if ! command -v node &>/dev/null; then
  echo "ERROR: Node.js required. Install from https://nodejs.org/"
  exit 1
fi
if ! command -v npm &>/dev/null; then
  echo "ERROR: npm required."
  exit 1
fi

# --- Clone the repo (shallow, single branch, just vscode-kl) ---
echo "==> Downloading extension source..."
mkdir -p "$TMP_DIR"
cd "$TMP_DIR"

# GitHub allows downloading a single directory via its SVN interface
# Fallback: download the whole repo ZIP and extract only vscode-kl
if command -v svn &>/dev/null; then
  # Fast path: svn export (GitHub serves SVN-compatible checkouts)
  svn export "https://github.com/$REPO/$BRANCH/vscode-kl" vscode-kl --quiet 2>/dev/null || true
fi

if [ ! -d "vscode-kl" ]; then
  # Fallback: download ZIP and extract
  echo "  (using ZIP download — install svn for faster download)"
  ZIP_URL="https://github.com/$REPO/archive/$BRANCH.zip"
  curl -fsSL "$ZIP_URL" -o repo.zip
  unzip -q repo.zip -d extracted
  mv extracted/KYLE-"$BRANCH"/vscode-kl ./vscode-kl 2>/dev/null || \
    mv extracted/KYLE-main/vscode-kl ./vscode-kl 2>/dev/null
  rm -rf repo.zip extracted
fi

cd vscode-kl

echo "==> Installing npm dependencies..."
npm install

echo "==> Packaging VSIX..."
VSIX_FILE="kl-extension.vsix"
npx @vscode/vsce package --out "$VSIX_FILE" 2>/dev/null || {
  echo "  vsce not found, installing..."
  npm install -g @vscode/vsce 2>/dev/null || true
  npx @vscode/vsce package --out "$VSIX_FILE"
}

if [ ! -f "$VSIX_FILE" ]; then
  echo "ERROR: failed to build VSIX."
  echo "Try: npm install -g @vscode/vsce && vsce package"
  exit 1
fi

echo "==> Installing in VS Code..."
"$CODE_CLI" --install-extension "$VSIX_FILE" --force

cd /
rm -rf "$TMP_DIR"

echo ""
echo "✅ Kyle VS Code extension installed!"
echo ""
echo "Verify:  $CODE_CLI --list-extensions | grep kl"
echo ""
echo "Uninstall: curl -fsSL https://raw.githubusercontent.com/$REPO/main/vscode-kl/install-extension.sh | sh -s uninstall"
