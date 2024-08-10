#!/bin/sh
#
# This script can be used to automatically pack the resources
# and locales and place them in the default cargo-build
# compilation directory (a.k.a. target), if you don't want to
# use meson.

cd "$(dirname "$0")/.."

meson_opts="$@"
if [ -d build ]; then
	meson_opts="--reconfigure $meson_opts"
fi

meson setup build $meson_opts
rm -rf build/data

ninja -C build data/kapestr.gresource
ninja -C build kapestr-gmo
mkdir -p target/share/kapestr
mkdir -p target/share/glib-2.0/schemas
mkdir -p target/share/icons/hicolor/scalable/apps
cp -R build/data/kapestr.gresource target/share/kapestr
cp -R data/py.com.kapestr.gschema.xml target/share/glib-2.0/schemas
cp -R build/po target/share/locale
cp -R data/icons/py.com.kapestr.svg target/share/icons/hicolor/scalable/apps
cp -R data/icons/py.com.kapestr.Devel.svg target/share/icons/hicolor/scalable/apps
cp -R build/data/ui target/share/
glib-compile-schemas target/share/glib-2.0/schemas

cp /usr/share/icons/hicolor/index.theme target/share/icons/hicolor
gtk4-update-icon-cache target/share/icons/hicolor

cargo build
