#!/bin/bash
# Script para instalar las fuentes 8-bit de NotNative

set -e

echo "==================================="
echo "NotNative - Instalador de Fuentes"
echo "==================================="
echo ""

# Directorio de fuentes del sistema
FONTS_DIR="/usr/share/fonts/truetype/notnative"
LOCAL_FONTS_DIR="$HOME/.local/share/fonts/notnative"

# Detectar si tenemos permisos de root
if [ "$EUID" -eq 0 ]; then
    echo "→ Instalando fuentes en el sistema (modo root)..."
    INSTALL_DIR="$FONTS_DIR"
    USE_SUDO=""
else
    echo "→ Instalando fuentes para el usuario actual..."
    INSTALL_DIR="$LOCAL_FONTS_DIR"
    USE_SUDO=""
fi

# Crear directorio de destino
echo "→ Creando directorio $INSTALL_DIR..."
$USE_SUDO mkdir -p "$INSTALL_DIR"

# Copiar fuentes
echo "→ Copiando fuentes..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
$USE_SUDO cp "$SCRIPT_DIR/fonts/"*.ttf "$INSTALL_DIR/"

# Actualizar caché de fuentes
echo "→ Actualizando caché de fuentes..."
if [ "$EUID" -eq 0 ]; then
    fc-cache -fv
else
    fc-cache -f "$LOCAL_FONTS_DIR"
fi

echo ""
echo "✓ Fuentes instaladas correctamente!"
echo ""
echo "Fuentes disponibles:"
fc-list | grep -i "vt323\|notnative" || echo "  - VT323 (retro terminal)"
echo ""
echo "Ya puedes usar el modo 8BIT en NotNative"
echo ""
