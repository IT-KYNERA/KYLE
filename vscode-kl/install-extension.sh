#!/bin/bash
set -eu

REPO="IT-KYNERA/KYLE"
VERSION="v0.4.0"

# --- Uninstall mode ---
if [ "${1:-}" = "uninstall" ]; then
  echo "Removing Kyle VS Code extension..."
  if command -v code &>/dev/null; then
    code --uninstall-extension kynera.kl 2>/dev/null || true
    echo "Extension uninstalled."
  else
    echo "VS Code 'code' CLI not found — remove the extension manually."
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
  echo ""
  echo "Make sure VS Code is installed and the 'code' command is in your PATH."
  echo "  - macOS: Open VS Code → Cmd+Shift+P → 'Shell Command: Install code command in PATH'"
  echo "  - Linux: 'code' should be in PATH after installing VS Code"
  echo ""
  echo "Alternatively, download the VSIX manually from:"
  echo "  https://github.com/$REPO/releases/tag/$VERSION"
  exit 1
fi

# --- Download VSIX ---
VSIX_NAME="kl-$VERSION.vsix"
VSIX_URL="https://github.com/$REPO/releases/download/$VERSION/$VSIX_NAME"
TMP_VSIX="/tmp/$VSIX_NAME"

echo "Downloading $VSIX_NAME..."
if command -v curl &>/dev/null; then
  curl -fsSL "$VSIX_URL" -o "$TMP_VSIX"
elif command -v wget &>/dev/null; then
  wget -q "$VSIX_URL" -O "$TMP_VSIX"
else
  echo "ERROR: neither curl nor wget found"
  exit 1
fi

# --- Install ---
echo "Installing extension..."
"$CODE_CLI" --install-extension "$TMP_VSIX" --force

rm -f "$TMP_VSIX"

echo ""
echo "✅ Kyle VS Code extension installed!"
echo ""
echo "To verify: open VS Code → Extensions → search '@installed kyle'"
echo "Or run:   $CODE_CLI --list-extensions | grep kl"
echo ""
echo "To uninstall later: curl -fsSL https://raw.githubusercontent.com/$REPO/main/vscode-kl/install-extension.sh | sh -s uninstall"
