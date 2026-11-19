#!/usr/bin/env bash
# Builds the Windows folder of E-Invoice Viewer (spec 0022, ADR 0005) from the
# release build and zips it. Runs in the MSYS2 MINGW64 shell of the release
# workflow, after `cargo build --release`.
#
#   target/windows/E-Invoice Viewer/               the folder (also the installer's input)
#   target/windows/E-Invoice_Viewer-<v>-windows-x86_64.zip
#   target/windows/bundled-packages-windows.txt
set -euo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
version=$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"$/\1/p' "$repo/Cargo.toml")
prefix=${MINGW_PREFIX:-/mingw64}
out=$repo/target/windows
dir=$out/E-Invoice\ Viewer
app_id=de.loges.EInvoiceViewer

rm -rf -- "$out"
mkdir -p "$dir/bin" "$dir/lib" "$dir/share"

install -m755 "$repo/target/release/e-invoice-viewer.exe" "$dir/bin/"

# GdkPixbuf loaders (SVG for the icons) with a cache that names them relative
# to the folder, as GdkPixbuf does on Windows.
loaders=lib/gdk-pixbuf-2.0/2.10.0
mkdir -p "$dir/$loaders/loaders"
cp "$prefix/$loaders/loaders/"*.dll "$dir/$loaders/loaders/"
(cd "$dir" && gdk-pixbuf-query-loaders "$loaders"/loaders/*.dll) \
    | sed "s|$(cygpath -m "$dir")/||g" >"$dir/$loaders/loaders.cache"

# Every DLL the program and the loaders load, from MinGW (not Windows).
# ntldd prints "name.dll => C:\msys64\mingw64\bin\name.dll (0x…)".
mingw_bin=$(cygpath -m "$prefix/bin")
{
    ntldd -R "$dir/bin/e-invoice-viewer.exe"
    for loader in "$dir/$loaders/loaders/"*.dll; do ntldd -R "$loader"; done
} | tr '\\' '/' | awk -v bin="$mingw_bin/" 'index(tolower($3), tolower(bin)) == 1 { print $3 }' \
    | sort -u | while IFS= read -r dll; do
    cp "$(cygpath -u "$dll")" "$dir/bin/"
done
test -f "$dir/bin/libgtk-4-1.dll"

# GSettings schemas (GTK's file dialog needs them).
mkdir -p "$dir/share/glib-2.0/schemas"
cp "$prefix/share/glib-2.0/schemas/"org.gtk.gtk4.*.gschema.xml "$dir/share/glib-2.0/schemas/"
glib-compile-schemas "$dir/share/glib-2.0/schemas"

# GtkSourceView's language and style definitions (XML view).
cp -r "$prefix/share/gtksourceview-5" "$dir/share/"

# The symbolic icons the app uses, and the app icon.
adwaita=share/icons/Adwaita
install -Dm644 "$prefix/$adwaita/index.theme" "$dir/$adwaita/index.theme"
for name in $(grep -rhoE '"[a-z0-9-]+-symbolic"' "$repo/crates/app/src" | tr -d '"' | sort -u); do
    icon=$(find "$prefix/$adwaita/symbolic" -name "$name.svg" | head -1)
    if [[ -z $icon ]]; then
        echo "error: icon $name is not in $prefix/$adwaita" >&2
        exit 1
    fi
    install -Dm644 "$icon" "$dir/${icon#"$prefix"/}"
done
install -Dm644 "$prefix/share/icons/hicolor/index.theme" "$dir/share/icons/hicolor/index.theme"
install -Dm644 "$repo/data/icons/$app_id.svg" "$dir/share/icons/hicolor/scalable/apps/$app_id.svg"

# Licence of the app, and the bundled MSYS2 packages with their licences;
# every bundled DLL must belong to a package.
install -m644 "$repo/LICENSE" "$dir/LICENSE.txt"
packages=$(mktemp)
missing=0
while IFS= read -r file; do
    owner=$(pacman -Qqo "$prefix/bin/$(basename "$file")" 2>/dev/null \
        || pacman -Qqo "$prefix/${file#"$dir"/}" 2>/dev/null || true)
    if [[ -z $owner ]]; then
        echo "error: $file belongs to no MSYS2 package" >&2
        missing=1
    else
        echo "$owner" >>"$packages"
    fi
done < <(find "$dir/bin" "$dir/lib" -name '*.dll' | sort)
[[ $missing == 0 ]]
list=$dir/bundled-packages-windows.txt
{
    echo "Libraries bundled in E-Invoice Viewer $version for Windows, unmodified MSYS2 (MINGW64) packages."
    echo "Their licence texts are in share/licenses/<package>; sources: https://repo.msys2.org/mingw/sources/"
    echo
    printf '%-44s %-24s %-40s %s\n' Package Version Licence Source
    sort -u "$packages" | while IFS= read -r package; do
        info=$(pacman -Qi "$package")
        field() { sed -n "s/^$1 *: //p" <<<"$info" | head -1; }
        base=$(pacman -Qi "$package" | sed -n 's/^Base *: //p' | head -1)
        printf '%-44s %-24s %-40s %s\n' "$package" "$(field Version)" "$(field Licenses)" \
            "https://packages.msys2.org/base/${base:-$package}"
        if [[ -d $prefix/share/licenses/${package#mingw-w64-x86_64-} ]]; then
            mkdir -p "$dir/share/licenses"
            cp -r "$prefix/share/licenses/${package#mingw-w64-x86_64-}" "$dir/share/licenses/"
        fi
    done
} >"$list"
rm -f -- "$packages"
cp "$list" "$out/bundled-packages-windows.txt"

zip_name=E-Invoice_Viewer-$version-windows-x86_64.zip
(cd "$out" && zip -qr "$zip_name" "E-Invoice Viewer")
echo "Built $out/$zip_name"
