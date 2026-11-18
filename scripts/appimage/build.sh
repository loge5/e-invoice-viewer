#!/usr/bin/env bash
# Runs inside the build container (started by scripts/appimage.sh): builds the
# binary, deploys it with its libraries into an AppDir and packs the AppImage.
set -euo pipefail

app_id=de.loges.EInvoiceViewer
src=/src
out=$src/target/appimage
appdir=$out/AppDir
version=$1

export CARGO_HOME=$out/cargo-home CARGO_TARGET_DIR=$out/build
export PATH=/opt:$PATH APPIMAGE_EXTRACT_AND_RUN=1 NO_APPSTREAM=1
# Same paths in every run, so that the binary is the same.
export RUSTFLAGS="--remap-path-prefix=$src=. --remap-path-prefix=$CARGO_HOME=cargo"

cargo build --release --locked -p e-invoice-viewer

rm -rf -- "$appdir"
install -Dm755 "$CARGO_TARGET_DIR/release/e-invoice-viewer" "$appdir/usr/bin/e-invoice-viewer"
install -Dm644 "$src/data/$app_id.metainfo.xml" "$appdir/usr/share/metainfo/$app_id.metainfo.xml"
# GtkSourceView's language and style definitions for the XML view; the GTK
# plugin does not know them.
mkdir -p "$appdir/usr/share"
cp -r /usr/share/gtksourceview-5 "$appdir/usr/share/"

# The symbolic icons the app uses, from the Adwaita theme, for systems whose
# icon theme lacks them.
adwaita=/usr/share/icons/Adwaita
install -Dm644 "$adwaita/index.theme" "$appdir$adwaita/index.theme"
for name in $(grep -rhoE '"[a-z0-9-]+-symbolic"' "$src/crates/app/src" | tr -d '"' | sort -u); do
    icon=$(find "$adwaita/symbolic" -name "$name.svg" | head -1)
    if [[ -z $icon ]]; then
        echo "error: icon $name is not in $adwaita" >&2
        exit 1
    fi
    install -Dm644 "$icon" "$appdir$icon"
done

# Copies the libraries, GTK's modules, loaders and schemas, and adds the start
# hook. linuxdeploy leaves out libraries it expects on every system; those that
# a minimal desktop install may lack are bundled explicitly (spec 0020, AC2).
lib=/usr/lib/x86_64-linux-gnu
DEPLOY_GTK_VERSION=4 linuxdeploy --appdir "$appdir" \
    --executable "$appdir/usr/bin/e-invoice-viewer" \
    --library "$lib/libharfbuzz.so.0" --library "$lib/libfribidi.so.0" \
    --library "$lib/libcom_err.so.2" --library "$lib/libgpg-error.so.0" \
    --desktop-file "$src/data/$app_id.desktop" \
    --icon-file "$src/data/icons/$app_id.svg" \
    --plugin gtk

# The plugin's hook forces X11 and a fixed theme, which breaks Wayland
# scaling and libadwaita's dark style; the app needs neither (ADR 0004).
hook=$appdir/apprun-hooks/linuxdeploy-plugin-gtk.sh
sed -i -e '/^export GDK_BACKEND=/d' -e '/^export GTK_THEME=/d' "$hook"
if grep -q -E '^export (GDK_BACKEND|GTK_THEME)=' "$hook"; then
    echo "error: cannot remove GDK_BACKEND/GTK_THEME from $hook" >&2
    exit 1
fi

# The bundled Ubuntu packages, with version, licence and source; every bundled
# library must belong to one of them (spec 0020).
docs=$appdir/usr/share/doc
list=$docs/e-invoice-viewer/bundled-packages.txt
mkdir -p "$docs/e-invoice-viewer"
packages=$(mktemp)
missing=0
while IFS= read -r file; do
    name=$(basename "$file")
    owner=$(dpkg -S "*/$name" 2>/dev/null | grep -v -E '^diversion' | head -1 | cut -d: -f1 || true)
    if [[ -z $owner ]]; then
        echo "error: $file belongs to no Ubuntu package" >&2
        missing=1
    else
        echo "$owner" >>"$packages"
    fi
done < <(find "$appdir/usr/lib" -type f \( -name '*.so' -o -name '*.so.*' \) | sort)
[[ $missing == 0 ]]
echo adwaita-icon-theme >>"$packages"
{
    echo "Libraries and icons bundled in E-Invoice Viewer $version, unmodified from Ubuntu 24.04."
    echo "Their licence texts are in usr/share/doc/<package>/copyright inside the AppImage."
    echo
    printf '%-32s %-36s %-40s %s\n' Package Version Licence Source
    sort -u "$packages" | while IFS= read -r package; do
        IFS=' ' read -r pkg_version source source_version < <(
            dpkg-query -W -f '${Version} ${source:Package} ${source:Version}\n' "$package")
        copyright=/usr/share/doc/$package/copyright
        licence=$(sed -n -E '0,/^License: +[^ ]/s/^License: +(.+)$/\1/p' "$copyright" 2>/dev/null || true)
        install -Dm644 "$copyright" "$docs/$package/copyright"
        printf '%-32s %-36s %-40s %s\n' "$package" "$pkg_version" "${licence:-see copyright}" \
            "https://launchpad.net/ubuntu/+source/$source/$source_version"
    done
} >"$list"
rm -f -- "$packages"

cd "$out"
rm -f -- ./*.AppImage
LINUXDEPLOY_OUTPUT_VERSION=$version linuxdeploy --appdir "$appdir" --output appimage
cp "$list" "$out/bundled-packages.txt"
