#!/bin/bash
set -e

cd /home/builder/workdir

VERSION=$(node -p "require('./package.json').version")


echo "--- Starting Debian build ---"
echo "Rust version: $(rustc --version)"
echo "Node.js version: $(node --version)"

export CARGO_TARGET_DIR="/home/builder/workdir/bin/distr/deb/target"
export CARGO_HOME="/home/builder/workdir/bin/distr/cargo-home-shared"
node ./builder/gen-version.js gui deb

cargo build -p nodeinnet-ice-commander-gtk --release

mkdir -p ./bin/gtk-app/release/
cp $CARGO_TARGET_DIR/release/nodeinnet-ice-commander ./bin/gtk-app/release/nodeinnet-ice-commander

rm -rf $CARGO_TARGET_DIR/debian
cargo deb -p nodeinnet-ice-commander-gtk

cp $CARGO_TARGET_DIR/debian/*-gtk_*.deb ./distr


FILE=$(ls -t distr/*-gtk_${VERSION}-*.deb | head -n 1)
[ -f "$FILE" ] || { echo "no GUI .deb for $VERSION in distr/" >&2; exit 1; }
echo "$(md5sum "$FILE" | awk '{print $1}') [GTK4-DEB] $(basename "$FILE")" >> distr/md5sums.txt

exit 0
