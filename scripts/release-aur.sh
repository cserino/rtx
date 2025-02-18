#!/usr/bin/env bash
set -euxo pipefail

VERSION=$(curl https://rtx.pub/VERSION | sed -e "s/^v//")
SHA512=$(curl -L "https://github.com/jdxcode/rtx/archive/v$VERSION.tar.gz" | sha512sum | awk '{print $1}')

if [ ! -d aur ]; then
  git clone ssh://aur@aur.archlinux.org/rtx.git aur
fi

cat >aur/PKGBUILD <<EOF
# Maintainer: Jeff Dickey <releases at chim dot sh>

pkgname=rtx
pkgver=$VERSION
pkgrel=1
pkgdesc='Polyglot runtime manager'
arch=('x86_64')
url='https://github.com/jdxcode/rtx'
license=('MIT')
makedepends=('cargo')
provides=('rtx')
conflicts=('rtx')
source=("\$pkgname-\$pkgver.tar.gz::https://github.com/jdxcode/\$pkgname/archive/v\$pkgver.tar.gz")
sha512sums=('$SHA512')

prepare() {
    cd "\$srcdir/\$pkgname-\$pkgver"
    cargo fetch --locked --target "\$CARCH-unknown-linux-gnu"
}

build() {
    cd "\$srcdir/\$pkgname-\$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release
}

package() {
    cd "\$srcdir/\$pkgname-\$pkgver"
    install -Dm0755 -t "\$pkgdir/usr/bin/" "target/release/\$pkgname"
}

check() {
    cd "\$srcdir/\$pkgname-\$pkgver"
    ./target/release/rtx --version
}
EOF

cat >aur/.SRCINFO <<EOF
pkgbase = rtx
	pkgdesc = Polyglot runtime manager
	pkgver = $VERSION
	pkgrel = 1
	url = https://github.com/jdxcode/rtx
	arch = x86_64
	license = MIT
	makedepends = cargo
	provides = rtx
	conflicts = rtx
	source = rtx-$VERSION.tar.gz::https://github.com/jdxcode/rtx/archive/v$VERSION.tar.gz
	sha512sums = $SHA512

pkgname = rtx
EOF

cd aur
git add .SRCINFO PKGBUILD
git commit -m "rtx $VERSION"
git push
