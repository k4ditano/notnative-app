# Maintainer: Abel <tu-email@dominio.com>
pkgname=notnative-app
pkgver=0.1.0
pkgrel=1
pkgdesc="Note-taking application with Vim-like keybindings"
arch=('x86_64')
url="https://github.com/tu-usuario/notnative"
license=('MIT')
depends=('gtk4' 'webkit2gtk-6.0' 'libadwaita' 'gtksourceview5' 'pulseaudio')
makedepends=('cargo' 'rust')
source=()
sha256sums=()

build() {
    cd "$startdir"
    cargo build --release
}

package() {
    cd "$startdir"
    
    # Instalar el binario
    install -Dm755 "target/release/notnative-app" "$pkgdir/usr/bin/notnative-app"
    
    # Instalar el archivo .desktop
    install -Dm644 "notnative.desktop" "$pkgdir/usr/share/applications/notnative.desktop"
    
    # Instalar el CSS (si es necesario)
    install -Dm644 "assets/style.css" "$pkgdir/usr/share/notnative/assets/style.css"
    
    # Instalar README
    install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
    
    # Instalar iconos (SVG es el principal, PNG para compatibilidad)
    install -Dm644 "assets/logo/logo.svg" "$pkgdir/usr/share/icons/hicolor/scalable/apps/notnative.svg"
    install -Dm644 "assets/logo/logo.png" "$pkgdir/usr/share/icons/hicolor/256x256/apps/notnative.png"
    install -Dm644 "assets/logo/logo.png" "$pkgdir/usr/share/pixmaps/notnative.png"
}
