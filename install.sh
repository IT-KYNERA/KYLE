#!/usr/bin/env bash
set -euo pipefail

REPO="IT-KYNERA/KYLE"

# --- Colores ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()  { printf "${BLUE}%s${NC}\n" "$*"; }
ok()    { printf "${GREEN}%s${NC}\n" "$*"; }
warn()  { printf "${YELLOW}%s${NC}\n" "$*"; }
error() { printf "${RED}%s${NC}\n" "$*"; exit 1; }

# --- Detectar plataforma ---
ARCH=$(uname -m)
OS=$(uname -s)

case "$OS-$ARCH" in
    Darwin-arm64|Darwin-aarch64) PLATFORM="macos-arm64" ;;
    Darwin-x86_64)               PLATFORM="macos-x64"   ;;
    Linux-aarch64)               PLATFORM="linux-arm64" ;;
    Linux-x86_64)                PLATFORM="linux-x64"   ;;
    *) error "Plataforma no soportada: $OS-$ARCH" ;;
esac

# --- Args ---
LOCAL_DIR=""
VERSION="latest"
if [ $# -ge 1 ] && [ "$1" = "--local" ]; then
    LOCAL_DIR="$2"
    info "Modo local: instalando desde $LOCAL_DIR"
elif [ $# -ge 1 ]; then
    VERSION="$1"
fi

# --- Detectar token / gh CLI para repos privados ---
if [ -z "$LOCAL_DIR" ]; then
    info "Plataforma detectada: $PLATFORM"

    AUTH=""
    if [ -n "${GITHUB_TOKEN:-}" ]; then
        AUTH="-H Authorization: token $GITHUB_TOKEN"
        info "Usando GITHUB_TOKEN para autenticación"
    elif command -v gh &>/dev/null; then
        info "Usando GitHub CLI (gh) para autenticación"
    fi

    if [ "$VERSION" = "latest" ]; then
        if command -v gh &>/dev/null && [ -z "${GITHUB_TOKEN:-}" ]; then
            VERSION=$(gh release view --repo "$REPO" --json tagName --jq .tagName 2>/dev/null || true)
        fi
        if [ -z "$VERSION" ] || [ "$VERSION" = "latest" ]; then
            VERSION=$(curl -fsSL $AUTH "https://api.github.com/repos/$REPO/releases/latest" \
                | grep '"tag_name"' | cut -d'"' -f4)
        fi
        if [ -z "$VERSION" ]; then
            error "No se pudo determinar la última versión.
Asegúrate de tener un token de GitHub:
  export GITHUB_TOKEN=ghp_...  y vuelve a ejecutar"
        fi
        info "Última versión: $VERSION"
    fi

    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/klc-$VERSION-$PLATFORM.tar.gz"
fi

# --- Determinar directorio de instalación ---
if [ -w /usr/local/bin ] 2>/dev/null; then
    INSTALL_DIR="/usr/local"
else
    INSTALL_DIR="$HOME/.kl"
    warn "No tienes permisos de escritura en /usr/local"
    warn "Instalando en $INSTALL_DIR"
    mkdir -p "$INSTALL_DIR/bin" "$INSTALL_DIR/lib"
fi

BIN_DIR="$INSTALL_DIR/bin"
LIB_DIR="$INSTALL_DIR/lib/klc"

# --- Descargar ---
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

if [ -n "$LOCAL_DIR" ]; then
    info "Copiando desde $LOCAL_DIR..."
    cp -r "$LOCAL_DIR/" "$TMPDIR/klc/"
else
    info "Descargando klc $VERSION..."
    if command -v gh &>/dev/null && [ -z "${GITHUB_TOKEN:-}" ]; then
        gh release download "$VERSION" --repo "$REPO" --pattern "klc-$VERSION-$PLATFORM.tar.gz" \
            --dir "$TMPDIR" --clobber
    else
        curl -fsSL $AUTH "$DOWNLOAD_URL" -o "$TMPDIR/klc.tar.gz"
    fi
    info "Extrayendo..."
    if [ -f "$TMPDIR/klc.tar.gz" ]; then
        tar xzf "$TMPDIR/klc.tar.gz" -C "$TMPDIR"
    fi
fi

# --- Instalar binario ---
mkdir -p "$BIN_DIR" "$LIB_DIR"
cp "$TMPDIR/klc/klc" "$BIN_DIR/klc"
chmod +x "$BIN_DIR/klc"
ok "klc instalado en $BIN_DIR/klc"

# --- Instalar runtime library ---
cp "$TMPDIR/klc/lib/libklc_runtime.a" "$LIB_DIR/libklc_runtime.a"
chmod 644 "$LIB_DIR/libklc_runtime.a"
ok "Runtime library instalada en $LIB_DIR/libklc_runtime.a"

# --- Crear uninstaller ---
UNINSTALL_SCRIPT="$BIN_DIR/klc-uninstall"
cat > "$UNINSTALL_SCRIPT" << 'UNINSTALL_EOF'
#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

INSTALL_DIR="$(cd "$(dirname "$(readlink -f "$0" 2>/dev/null || echo "$0")")/.." && pwd)"

echo -e "${YELLOW}Se eliminara klc de: $INSTALL_DIR${NC}"
echo -n "Continuar? [s/N] "
read -r CONFIRM
if [ "$CONFIRM" != "s" ] && [ "$CONFIRM" != "S" ]; then
    echo "Cancelado."
    exit 0
fi

rm -f "$INSTALL_DIR/bin/klc"
rm -f "$INSTALL_DIR/bin/klc-uninstall"
rm -rf "$INSTALL_DIR/lib/klc"

if [ "$INSTALL_DIR" != "/usr/local" ]; then
    rmdir "$INSTALL_DIR/bin" 2>/dev/null || true
    rmdir "$INSTALL_DIR/lib" 2>/dev/null || true
    rmdir "$INSTALL_DIR" 2>/dev/null || true
fi

echo -e "${GREEN}klc eliminado correctamente.${NC}"
echo "Elimina manualmente la entrada del PATH si la agregaste:"
echo "  ~/.zshrc  ->  export PATH=\"\$HOME/.kl/bin:\$PATH\""
UNINSTALL_EOF
chmod +x "$UNINSTALL_SCRIPT"
ok "Uninstaller creado: $UNINSTALL_SCRIPT (ejecuta 'klc-uninstall' para desinstalar)"

# --- Verificar instalación ---
info "Verificando instalación..."
if "$BIN_DIR/klc" --version >/dev/null 2>&1; then
    ok "klc funciona correctamente"
    "$BIN_DIR/klc" --version
else
    error "klc no funciona después de la instalación"
fi

# --- PATH advice ---
if [ "$INSTALL_DIR" = "$HOME/.kl" ]; then
    echo ""
    echo -e "${YELLOW}Agrega esto a tu ~/.zshrc (o ~/.bashrc):${NC}"
    echo "  export PATH=\"\$HOME/.kl/bin:\$PATH\""
    echo ""
    echo "O ejecuta ahora:"
    echo "  export PATH=\"\$HOME/.kl/bin:\$PATH\""
fi

ok "Instalacion completada. Ejecuta 'klc --help' para empezar."
