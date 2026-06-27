#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

error() { printf "${RED}%s${NC}\n" "$*"; exit 1; }
info()  { printf "${YELLOW}%s${NC}\n" "$*"; }
ok()    { printf "${GREEN}%s${NC}\n" "$*"; }

# Buscar kl en PATH
KL_PATH="$(command -v kl 2>/dev/null || true)"
if [ -z "$KL_PATH" ]; then
    error "kl no está instalado en el PATH"
fi

# Resolver directorio de instalación
if [ -L "$KL_PATH" ]; then
    KL_REAL="$(readlink -f "$KL_PATH" 2>/dev/null || readlink "$KL_PATH")"
    KL_DIR="$(dirname "$(dirname "$KL_REAL")")"
else
    KL_DIR="$(dirname "$(dirname "$KL_PATH")")"
fi

echo "Se eliminará kl de:"
echo "  Binario:  $KL_PATH"
echo "  Directorio: $KL_DIR"
echo ""
echo -n "Continuar? [s/N] "
read -r CONFIRM
if [ "$CONFIRM" != "s" ] && [ "$CONFIRM" != "S" ]; then
    echo "Cancelado."
    exit 0
fi

# Eliminar binarios
rm -f "$KL_DIR/bin/kl" "$KL_DIR/bin/klc"
ok "Binarios eliminados: kl + klc (legacy)"

# Eliminar runtime library
if [ -d "$KL_DIR/lib/kl" ]; then
    rm -rf "$KL_DIR/lib/kl"
    ok "Runtime library eliminada: $KL_DIR/lib/kl"
fi

# Eliminar uninstaller
rm -f "$KL_DIR/bin/kl-uninstall"

# Limpiar directorios vacíos (solo si no es /usr/local)
if [ "$KL_DIR" != "/usr/local" ]; then
    rmdir "$KL_DIR/bin" 2>/dev/null || true
    rmdir "$KL_DIR/lib" 2>/dev/null || true
    rmdir "$KL_DIR" 2>/dev/null || true
    ok "Directorios eliminados: $KL_DIR"
fi

echo ""
ok "kl ha sido eliminado completamente."
echo ""
echo "Si agregaste ~/.kl/bin al PATH, elimina esa línea de ~/.zshrc o ~/.bashrc"
