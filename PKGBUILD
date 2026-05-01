# Maintainer: NullCode1337 
# Contributor: NullCode1337 

pkgname=adbfiles-git
pkgver=0.0.1
pkgrel=1
pkgdesc="Dead simple Android file explorer using ADB" 
arch=('x86_64')
url="https://github.com/NullCode1337/ADBFiles" 
license=('MIT') 
depends=('cairo' 'desktop-file-utils' 'gdk-pixbuf2' 'glib2' 'gtk3' 'hicolor-icon-theme' 'pango' 'webkit2gtk-4.1' 'android-tools' 'android-udev')
makedepends=('git' 'openssl' 'appmenu-gtk-module' 'libappindicator-gtk3' 'librsvg' 'cargo' 'pnpm' 'nodejs')
provides=('adbfiles')
conflicts=('adbfiles' 'adbfiles-bin')
source=("${pkgname}::git+${url}.git")
sha256sums=('SKIP')

pkgver() {
  cd "${pkgname}"
  ( set -o pipefail
    git describe --long --tags --abbrev=7 2>/dev/null | sed 's/^v//;s/\([^-]*-g\)/r\1/;s/-/./g' ||
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short=7 HEAD)"
  )
}

prepare() {
  cd "${pkgname}"
  pnpm install
  cd "src-tauri"
  cargo update
}

build() {
  cd "${pkgname}"
  pnpm tauri build -b deb 
}

package() {
  cd "${pkgname}"
  cp -a src-tauri/target/release/bundle/deb/adbfiles_*_*/data/* "${pkgdir}/"
}
