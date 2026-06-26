#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

error() { printf "${RED}❌ %s${NC}\n" "$*"; exit 1; }
info()  { printf "${YELLOW}ℹ️  %s${NC}\n" "$*"; }
ok()    { printf "${GREEN}✅ %s${NC}\n" "$*"; }

# Buscar klc en PATH
KLC_PATH="$(command -v klc 2>/dev/null || true)"
if [ -z "$KLC_PATH" ]; then
    error "klc no está instalado en el PATH"
fi

# Resolver directorio de instalación
# Si es un symlink, seguirlo
if [ -L "$KLC_PATH" ]; then
    KLC_REAL="$(readlink -f "$KLC_PATH" 2>/dev/null || readlink "$KLC_PATH")"
    KLC_DIR="$(dirname "$(dirname "$KLC_REAL")")"
else
    KLC_DIR="$(dirname "$(dirname "$KLC_PATH")")"
fi

echo -e "${YELLOW}⚠️  Se eliminará klc de:${NC}"
echo "  Binario:  $KLC_PATH"
echo "  Directorio: $KLC_DIR"
echo ""
echo -n "¿Continuar? [s/N] "
read -r CONFIRM
if [ "$CONFIRM" != "s" ] && [ "$CONFIRM" != "S" ]; then
    echo "Cancelado."
    exit 0
fi

# Eliminar binario
rm -f "$KLC_PATH"
ok "Binario eliminado: $KLC_PATH"

# Eliminar runtime library
if [ -d "$KLC_DIR/lib/klc" ]; then
    rm -rf "$KLC_DIR/lib/klc"
    ok "Runtime library eliminada: $KLC_DIR/lib/klc"
fi

# Eliminar uninstaller
if [ -f "$KLC_DIR/bin/klc-uninstall" ]; then
    rm -f "$KLC_DIR/bin/klc-uninstall"
fi

# Limpiar directorios vacíos (solo si no es /usr/local)
if [ "$KLC_DIR" != "/usr/local" ]; then
    rmdir "$KLC_DIR/bin" 2>/dev/null || true
    rmdir "$KLC_DIR/lib" 2>/dev/null || true
    rmdir "$KLC_DIR" 2>/dev/null || true
    ok "Directorios eliminados: $KLC_DIR"
fi

echo ""
ok "klc ha sido eliminado completamente."
echo ""
echo "Si agregaste ~/.kl/bin al PATH, elimina esa línea de ~/.zshrc o ~/.bashrc"
