#!/bin/bash
set -e

cd /home/builder/workdir

VERSION=$(node -p "require('./package.json').version")

export CARGO_TARGET_DIR="/home/builder/workdir/bin/distr/zst/target"
export CARGO_HOME="/home/builder/workdir/bin/distr/cargo-home-shared"
node ./builder/gen-version.js gui zst

cargo build -p nodeinnet-ice-commander-gtk --release
cd ./src/gtk-app
# Disable LTO, debug packages, and stripping from makepkg to preserve exact binary MD5
echo "OPTIONS+=(!strip !lto !debug)" > ~/.makepkg.conf

# Extract version from package.json
mkdir -p /tmp/ice-commander-arch-build

cat <<EOF > /tmp/ice-commander-arch-build/PKGBUILD
pkgname=nodeinnet-ice-commander-gtk
pkgver=$VERSION
pkgrel=1
pkgdesc="Ice Commander - Dual-Pane P2P File Manager"
arch=('x86_64')
url="https://icecommander.com"
license=('MIT')
depends=('gtk4' 'libadwaita' 'alsa-lib')
provides=('nodeinnet-ice-commander')
replaces=('nodeinnet-ice-commander')
conflicts=('nodeinnet-ice-commander')
source=()
sha256sums=()

package() {
    install -Dm755 "/home/builder/workdir/bin/distr/zst/target/release/nodeinnet-ice-commander" "\$pkgdir/usr/bin/nodeinnet-ice-commander"
    install -Dm644 "/home/builder/workdir/src/gtk-app/assets/com.nodeinnet.icecommander.gtkapp.desktop" "\$pkgdir/usr/share/applications/com.nodeinnet.icecommander.gtkapp.desktop"
    install -Dm644 "/home/builder/workdir/src/gtk-app/assets/app-logo-512.png" "\$pkgdir/usr/share/icons/hicolor/512x512/apps/com.nodeinnet.icecommander.gtkapp.png"
    install -Dm644 "/home/builder/workdir/artifacts/libpdfium.so" "\$pkgdir/usr/lib/nodeinnet-ice-commander/libpdfium.so"
    for lic in /home/builder/workdir/assets/licenses/*.txt; do
        install -Dm644 "\$lic" "\$pkgdir/usr/share/doc/nodeinnet-ice-commander/licenses/\$(basename "\$lic")"
    done
}
EOF

cd /tmp/ice-commander-arch-build
PKGDEST="/home/builder/workdir/distr/" CARGO_TARGET_DIR="/home/builder/workdir/bin/distr/zst/target" makepkg -cf
cd /home/builder/workdir


FILE=$(ls -t distr/*-gtk-${VERSION}-*.pkg.tar.zst | head -n 1)
[ -f "$FILE" ] || { echo "no GUI package for $VERSION in distr/" >&2; exit 1; }
echo "$(md5sum "$FILE" | awk '{print $1}') [GTK4-ZST] $(basename "$FILE")" >> distr/md5sums.txt
