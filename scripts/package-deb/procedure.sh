#!/usr/bin/env bash
# Build the Debian package from a compiled binary.
#
#   scripts/package-deb/procedure.sh <binary> <version> <deb-arch> <out-dir>
#
# The package keeps the 1.x package name (internet-speed-monitor), so installing it
# upgrades a 1.x install in place: dpkg removes the old Electron files under
# /opt/Internet Speed Monitor, and preinst retires the update-alternatives link 1.x used
# for /usr/bin/internet-speed-monitor. User data in ~/.config/internet-speed-monitor is
# never touched.
set -euo pipefail

binary=$1
version=$2
arch=$3
out_dir=$4

name=internet-speed-monitor
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
root=$(mktemp -d)
trap 'rm -rf "$root"' EXIT

install -Dm755 "$binary" "$root/usr/bin/$name"
install -Dm644 "$here/$name.desktop" "$root/usr/share/applications/$name.desktop"
for size in 16 32 48 64 128 256 512; do
	install -Dm644 "$repo/docs/assets/icon-${size}x${size}.png" \
		"$root/usr/share/icons/hicolor/${size}x${size}/apps/$name.png"
done
install -Dm644 "$repo/LICENSE" "$root/usr/share/doc/$name/copyright"

mkdir -p "$root/DEBIAN"
for script in preinst postinst postrm; do
	install -m755 "$here/$script" "$root/DEBIAN/$script"
done

installed_size=$(du -sk --exclude=DEBIAN "$root" | cut -f1)
cat > "$root/DEBIAN/control" <<CONTROL
Package: $name
Version: $version
Architecture: $arch
Maintainer: Omri Lotan <omri@omrilotan.com>
Installed-Size: $installed_size
Depends: libc6 (>= 2.31), libgcc-s1, libgl1 | libegl1, libxkbcommon0
Recommends: libx11-6, libxcursor1, libxrandr2, libxi6, libxkbcommon-x11-0, libwayland-client0, xdg-desktop-portal | zenity
Section: net
Priority: optional
Homepage: https://github.com/omrilotan/internet-speed-monitor
Description: Monitor your internet connection speed over time
 Runs speed tests against Netflix's Fast.com on an interval or cron schedule,
 keeps the history locally, and shows charts, medians and CSV export.
CONTROL

mkdir -p "$out_dir"
deb="$out_dir/${name}_${version}_${arch}.deb"
dpkg-deb --root-owner-group --build "$root" "$deb"
echo "$deb"
