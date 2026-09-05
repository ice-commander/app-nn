#!/bin/bash
set -e

cd /home/builder/workdir

VERSION=$(node -p "require('./package.json').version")


echo "--- Starting Arch build (console) ---"

export CARGO_TARGET_DIR="/home/builder/workdir/bin/distr/zst/target"
export CARGO_HOME="/home/builder/workdir/bin/distr/cargo-home-shared"
node ./builder/gen-version.js console zst

cargo build -p console-app --release

# Disable LTO, debug packages, and stripping from makepkg to preserve exact binary MD5
echo "OPTIONS+=(!strip !lto !debug)" > ~/.makepkg.conf

mkdir -p /tmp/nodeinnet-ice-commander-console-arch-build

# No gtk/adwaita/alsa — a single GTK-free terminal binary.
cat <<EOF > /tmp/nodeinnet-ice-commander-console-arch-build/PKGBUILD
pkgname=nodeinnet-ice-commander-console
pkgver=$VERSION
pkgrel=1
pkgdesc="Ice Commander — terminal file manager (dual-pane TUI, à la Midnight Commander)"
arch=('x86_64')
url="https://icecommander.com"
license=('MIT')
depends=()
source=()
sha256sums=()

package() {
    install -Dm755 "/home/builder/workdir/bin/distr/zst/target/release/nodeinnet-ice-console" "\$pkgdir/usr/bin/nodeinnet-ice-console"
}
EOF

cd /tmp/nodeinnet-ice-commander-console-arch-build
PKGDEST="/home/builder/workdir/distr/" CARGO_TARGET_DIR="/home/builder/workdir/bin/distr/zst/target" makepkg -cf
cd /home/builder/workdir


FILE=$(ls -t distr/nodeinnet-ice-commander-console*.pkg.tar.zst | head -n 1)
echo "$(md5sum "$FILE" | awk '{print $1}') Arch Linux console" >> distr/md5sums.txt
