#!/usr/bin/env bash
# Installs E-Invoice Viewer for the current user, without administrator rights
# (spec 0012): the binary into ~/.local/bin and a desktop entry, so that the app
# is listed under "Öffnen mit" for PDF and XML files. It never becomes the
# default application: the desktop entry only declares the MIME types, and no
# default is set.
#
# Usage:
#   scripts/install.sh               build (release) and install
#   scripts/install.sh --uninstall   remove binary, desktop entry and icon
#
# Locations follow the XDG base directories:
#   binary         ${BIN_DIR:-$HOME/.local/bin}/e-invoice-viewer
#   desktop entry  ${XDG_DATA_HOME:-$HOME/.local/share}/applications/de.loges.EInvoiceViewer.desktop
#   icon           ${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/scalable/apps/de.loges.EInvoiceViewer.svg
# The recent files list in ~/.config/e-invoice-viewer is the user's and is kept
# on uninstall.
set -euo pipefail

app_id=de.loges.EInvoiceViewer
bin_dir=${BIN_DIR:-$HOME/.local/bin}
applications=${XDG_DATA_HOME:-$HOME/.local/share}/applications
binary=$bin_dir/e-invoice-viewer
desktop_entry=$applications/$app_id.desktop
hicolor=${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor
icon=$hicolor/scalable/apps/$app_id.svg
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

refresh_desktop_database() {
    if command -v update-desktop-database >/dev/null; then
        update-desktop-database -q "$applications"
    fi
}

# Icon themes are rescanned when the theme directory itself changes, not when
# one of its subdirectories does; without this, GNOME Shell keeps showing the
# placeholder icon until the next login.
refresh_icon_theme() {
    touch --no-create "$hicolor"
}

case ${1:-} in
--uninstall)
    rm -f -- "$desktop_entry" "$binary" "$icon"
    refresh_icon_theme
    refresh_desktop_database
    echo "Removed $binary, $desktop_entry and $icon"
    exit 0
    ;;
"") ;;
*)
    echo "usage: $0 [--uninstall]" >&2
    exit 2
    ;;
esac

# SOURCE_BINARY installs a given binary instead of building one (used by the
# tests).
if [[ -n ${SOURCE_BINARY:-} ]]; then
    source_binary=$SOURCE_BINARY
else
    cargo build --release --locked -p e-invoice-viewer --manifest-path "$repo/Cargo.toml"
    source_binary=$repo/target/release/e-invoice-viewer
fi
install -Dm755 "$source_binary" "$binary"

# The desktop entry from data/, with the full path of the binary (%F: the file
# manager passes local paths, also for mounted network shares).
staging=$(mktemp -d)
trap 'rm -rf -- "$staging"' EXIT
staged=$staging/$app_id.desktop
sed "s|^Exec=e-invoice-viewer |Exec=\"$binary\" |" "$repo/data/$app_id.desktop" >"$staged"
grep -q "^Exec=\"$binary\" %F$" "$staged"
if command -v desktop-file-validate >/dev/null; then
    desktop-file-validate "$staged"
fi
install -Dm644 "$repo/data/icons/$app_id.svg" "$icon"
install -Dm644 "$staged" "$desktop_entry"
refresh_icon_theme
refresh_desktop_database
echo "Installed $binary, $desktop_entry and $icon"
