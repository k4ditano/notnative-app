# Guía para Publicar NotNative en AUR

## ¿Qué es AUR?

AUR (Arch User Repository) es un repositorio mantenido por la comunidad donde cualquiera puede publicar PKGBUILDs para que otros usuarios de Arch Linux puedan instalar fácilmente tu aplicación con todas sus dependencias.

## Ventajas de publicar en AUR

✅ **Instalación automática de dependencias** - `pacman` las instala automáticamente
✅ **Fácil para usuarios** - Solo necesitan: `yay -S notnative-app`
✅ **Actualizaciones centralizadas** - Los usuarios reciben actualizaciones automáticamente
✅ **Visibilidad** - Cualquiera puede encontrar tu app buscando en AUR
✅ **Integración con el sistema** - Se instala como cualquier paquete oficial

## Pasos para publicar en AUR

### 1. Crear cuenta en AUR

1. Ve a https://aur.archlinux.org/register
2. Crea una cuenta con tu email
3. Confirma tu email
4. Sube tu clave SSH pública en https://aur.archlinux.org/account

### 2. Configurar SSH para AUR

```bash
# Generar clave SSH si no tienes una
ssh-keygen -t ed25519 -C "tu-email@dominio.com"

# Copiar tu clave pública
cat ~/.ssh/id_ed25519.pub

# Pegarla en https://aur.archlinux.org/account
```

### 3. Preparar el PKGBUILD

Necesitas modificar el PKGBUILD actual para que descargue desde GitHub:

```bash
# Maintainer: Tu Nombre <tu-email@dominio.com>
pkgname=notnative-app
pkgver=0.1.0
pkgrel=1
pkgdesc="Note-taking application with Vim-like keybindings"
arch=('x86_64')
url="https://github.com/k4ditano/notnative-app"
license=('MIT')
depends=('gtk4' 'webkit2gtk-6.0' 'libadwaita' 'gtksourceview5' 'pulseaudio')
makedepends=('cargo' 'rust' 'git')
source=("$pkgname-$pkgver.tar.gz::https://github.com/k4ditano/notnative-app/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')  # Cambiar después del primer release

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --locked
}

check() {
    cd "$pkgname-$pkgver"
    cargo test --release --locked
}

package() {
    cd "$pkgname-$pkgver"
    
    # Instalar el binario
    install -Dm755 "target/release/notnative-app" "$pkgdir/usr/bin/notnative-app"
    
    # Instalar el archivo .desktop
    install -Dm644 "notnative.desktop" "$pkgdir/usr/share/applications/notnative.desktop"
    
    # Instalar assets
    install -Dm644 "assets/style.css" "$pkgdir/usr/share/notnative/assets/style.css"
    
    # Instalar iconos
    install -Dm644 "assets/logo/logo.svg" "$pkgdir/usr/share/icons/hicolor/scalable/apps/notnative.svg"
    install -Dm644 "assets/logo/logo.png" "$pkgdir/usr/share/icons/hicolor/256x256/apps/notnative.png"
    install -Dm644 "assets/logo/logo.png" "$pkgdir/usr/share/pixmaps/notnative.png"
    
    # Instalar documentación
    install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
```

### 4. Crear .SRCINFO

```bash
cd /ruta/a/tu/pkgbuild
makepkg --printsrcinfo > .SRCINFO
```

### 5. Clonar el repositorio AUR

```bash
# Clonar el repo vacío
git clone ssh://aur@aur.archlinux.org/notnative-app.git
cd notnative-app

# Copiar archivos
cp /ruta/al/PKGBUILD .
cp /ruta/al/.SRCINFO .

# Hacer commit
git add PKGBUILD .SRCINFO
git commit -m "Initial commit: notnative-app v0.1.0"

# Subir a AUR
git push origin master
```

### 6. Crear un release en GitHub

Antes de publicar en AUR, necesitas crear un release/tag en GitHub:

```bash
# Crear tag
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0

# O crear release desde GitHub web:
# https://github.com/k4ditano/notnative-app/releases/new
```

### 7. Actualizar sha256sum

```bash
# Descargar el tarball
wget https://github.com/k4ditano/notnative-app/archive/refs/tags/v0.1.0.tar.gz

# Calcular checksum
sha256sum v0.1.0.tar.gz

# Copiar el hash y ponerlo en PKGBUILD:
sha256sums=('el_hash_que_obtuviste')
```

### 8. Probar el PKGBUILD

```bash
# Limpiar build anterior
rm -rf src/ pkg/ *.tar.gz

# Construir
makepkg -si

# Si funciona, actualizar .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Commit y push
git add PKGBUILD .SRCINFO
git commit -m "Update checksums"
git push
```

## Cómo los usuarios instalarán tu app

### Con yay (AUR helper más popular):
```bash
yay -S notnative-app
```

### Con paru:
```bash
paru -S notnative-app
```

### Manualmente:
```bash
git clone https://aur.archlinux.org/notnative-app.git
cd notnative-app
makepkg -si
```

## Mantener el paquete actualizado

Cuando saques una nueva versión:

```bash
# 1. Actualizar version en GitHub
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0

# 2. Actualizar PKGBUILD en AUR repo
cd ~/aur/notnative-app
# Editar PKGBUILD: cambiar pkgver=0.2.0 y pkgrel=1
# Actualizar sha256sums

# 3. Regenerar .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# 4. Commit y push
git add PKGBUILD .SRCINFO
git commit -m "Update to v0.2.0"
git push
```

### Actualización rápida para v0.1.1 (AHORA)

```bash
# 1. Clonar el repo AUR si no lo tienes
git clone ssh://aur@aur.archlinux.org/notnative-app.git ~/notnative-app-aur
cd ~/notnative-app-aur

# 2. Copiar el PKGBUILD actualizado
cp ~/Programacion/notnative/notnative-app/PKGBUILD.aur ./PKGBUILD
cp ~/Programacion/notnative/notnative-app/disable-bundled-sqlite.patch ./

# 3. Actualizar checksums
updpkgsums

# 4. Probar build
makepkg -scf

# 5. Si funciona, instalar para probar
makepkg -sfi

# 6. Regenerar .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# 7. Commit y push
git add PKGBUILD .SRCINFO disable-bundled-sqlite.patch
git commit -m "Update to v0.1.1 - Fix Omarchy theme loading

- Fixed theme not loading when installed from AUR
- Improved CSS loading order
- Updated README with English translation
- Added troubleshooting guide"
git push
```

## Estructura de archivos en AUR

```
notnative-app/     # Tu repo AUR
├── PKGBUILD       # Receta de construcción
└── .SRCINFO       # Metadata generada automáticamente
```

**IMPORTANTE:** Solo subes PKGBUILD y .SRCINFO. El código fuente se descarga de GitHub.

## Checklist antes de publicar

- [ ] Tienes cuenta en AUR
- [ ] SSH configurado
- [ ] Creado release/tag en GitHub (v0.1.0)
- [ ] PKGBUILD apunta a tu repo de GitHub
- [ ] sha256sums correcto
- [ ] Probado con `makepkg -si`
- [ ] .SRCINFO generado
- [ ] LICENSE file en tu repo de GitHub

## Recursos

- AUR Guidelines: https://wiki.archlinux.org/title/AUR_submission_guidelines
- PKGBUILD: https://wiki.archlinux.org/title/PKGBUILD
- AUR: https://aur.archlinux.org/

## Tips

💡 **Nombre del paquete**: Debe coincidir con el repo AUR (`notnative-app`)
💡 **Versioning**: Usa semantic versioning (x.y.z)
💡 **pkgrel**: Empieza en 1, incrementa si cambias PKGBUILD sin cambiar versión
💡 **Tests**: La función `check()` es opcional pero recomendada
💡 **Licencia**: Asegúrate de tener un archivo LICENSE en GitHub

## Alternativa: AUR -git package

Para desarrollo activo, puedes crear `notnative-app-git` que instala desde master:

```bash
pkgname=notnative-app-git
pkgver=r123.abc1234  # Se actualiza automáticamente
source=("git+https://github.com/k4ditano/notnative-app.git")
# ... resto del PKGBUILD similar pero usa 'git pull' en pkgver()
```

Esto permite a usuarios probar la última versión de desarrollo.
