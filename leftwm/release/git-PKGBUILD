# Maintainer: Lex Childs <lexchilds@gmail.com>
# Maintainer: hertg <aur@her.tg>
# Contributor: éclairevoyant
# Contributor: mautam <mautam@usa.com>

pkgbase=leftwm-git
pkgname=(leftwm-nonsystemd-git leftwm-git)
pkgver=0.5.1.r0.g2ae93293
pkgrel=1
pkgdesc="A tiling window manager for the adventurer"
arch=('i686' 'x86_64')
url="https://github.com/leftwm/leftwm"
license=('MIT')
depends=(gcc-libs)
makedepends=('cargo' 'git')
optdepends=('bash: themes'
            'dmenu: default launcher'
            'eww: flexible status bar'
            'feh: used to set background images'
            'lemonbar: light weight bar'
            'polybar: light weight bar')
provides=('leftwm')
conflicts=('leftwm')
source=("$pkgbase::git+https://github.com/leftwm/leftwm.git")
md5sums=('SKIP')
install='readme.install'

prepare() {
	cd $pkgbase
	cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

pkgver() {
	cd $pkgbase
	git describe --long --tags | sed 's/\([^-]*-g\)/r\1/;s/-/./g'
}

build() {
	cd $pkgbase

	export CARGO_TARGET_DIR=target_non_systemd
	cargo build --frozen --release --no-default-features --features=lefthk,sys-log

	export CARGO_TARGET_DIR=target_systemd
	cargo build --frozen --release
}

_package() {
	install -Dm755 leftwm{,-worker,-state,-check,-command} lefthk-worker -t "$pkgdir"/usr/bin/

	cd ../../
	install -Dm644 leftwm/doc/leftwm.1 -t "$pkgdir"/usr/share/man/man1/
	install -d "$pkgdir"/usr/share/leftwm
	cp -R themes "$pkgdir"/usr/share/leftwm
	install -Dm644 leftwm.desktop -t "$pkgdir"/usr/share/xsessions/
}

package_leftwm-nonsystemd-git() {
	pkgdesc+=" (non-systemd init)"
	cd $pkgbase/target_non_systemd/release
	_package
}

package_leftwm-git() {
	pkgdesc+=" (systemd init)"
	depends+=(systemd)
	cd $pkgbase/target_systemd/release
	_package
}
