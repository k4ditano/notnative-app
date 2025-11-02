#!/bin/bash

# Script de instalación local para NotNative en ArchLinux
# Instala la aplicación sin necesidad de crear un paquete

set -e

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "==> Verificando dependencias..."

# Lista de dependencias requeridas
DEPENDENCIES=("gtk4" "webkit2gtk-6.0" "libadwaita" "gtksourceview5" "pulseaudio" "cargo")
MISSING_DEPS=()

for dep in "${DEPENDENCIES[@]}"; do
    if ! pacman -Qi "$dep" &> /dev/null; then
        MISSING_DEPS+=("$dep")
    fi
done

if [ ${#MISSING_DEPS[@]} -ne 0 ]; then
    echo -e "${YELLOW}Las siguientes dependencias no están instaladas:${NC}"
    printf '  - %s\n' "${MISSING_DEPS[@]}"
    echo ""
    read -p "¿Deseas instalarlas ahora? [S/n] " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Ss]$ ]] || [[ -z $REPLY ]]; then
        echo "==> Instalando dependencias..."
        sudo pacman -S --needed "${MISSING_DEPS[@]}"
    else
        echo -e "${RED}Error: Se requieren todas las dependencias para compilar e instalar NotNative${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}✓ Todas las dependencias están instaladas${NC}"
echo ""

echo "==> Compilando NotNative en modo release..."
cargo build --release

echo "==> Instalando binario..."
sudo install -Dm755 "target/release/notnative-app" "/usr/bin/notnative-app"

echo "==> Instalando archivo .desktop..."
sudo install -Dm644 "notnative.desktop" "/usr/share/applications/notnative.desktop"

echo "==> Instalando assets..."
sudo mkdir -p "/usr/share/notnative/assets"
sudo install -Dm644 "assets/style.css" "/usr/share/notnative/assets/style.css"

echo "==> Instalando iconos..."
sudo install -Dm644 "assets/logo/logo.svg" "/usr/share/icons/hicolor/scalable/apps/notnative.svg"
sudo install -Dm644 "assets/logo/logo.png" "/usr/share/icons/hicolor/256x256/apps/notnative.png"
sudo install -Dm644 "assets/logo/logo.png" "/usr/share/pixmaps/notnative.png"

echo "==> Actualizando base de datos de aplicaciones..."
sudo update-desktop-database /usr/share/applications 2>/dev/null || true
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true

echo ""
echo "✓ NotNative instalado correctamente!"
echo "  Puedes ejecutarlo con: notnative-app"
echo "  O buscarlo en el menú de aplicaciones"
echo ""
echo "Para desinstalar, ejecuta: sudo rm /usr/bin/notnative-app /usr/share/applications/notnative.desktop && sudo rm -rf /usr/share/notnative"
