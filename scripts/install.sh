#!/usr/bin/env bash
# Installs E-Invoice Viewer for the current user, without administrator rights
# (spec 0012): the binary into ~/.local/bin and a desktop entry, so that the app
# is listed under "Öffnen mit" for PDF and XML files. It never becomes the
# default application: the desktop entry only declares the MIME types, and no
# default is set.
#
# Usage:
#   scripts/install.sh               build (release) and install
#   scripts/install.sh --uninstall   remove binary and desktop entry
#
# Locations follow the XDG base directories:
#   binary         ${BIN_DIR:-$HOME/.local/bin}/e-invoice-viewer
#   desktop entry  ${XDG_DATA_HOME:-$HOME/.local/share}/applications/de.loges.EInvoiceViewer.desktop
# The recent files list in ~/.config/e-invoice-viewer is the user's and is kept
# on uninstall.
set -euo pipefail

app_id=de.loges.EInvoiceViewer
bin_dir=${BIN_DIR:-$HOME/.local/bin}
applications=${XDG_DATA_HOME:-$HOME/.local/share}/applications
binary=$bin_dir/e-invoice-viewer
desktop_entry=$applications/$app_id.desktop
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

refresh_desktop_database() {
    if command -v update-desktop-database >/dev/null; then
        update-desktop-database -q "$applications"
    fi
}

case ${1:-} in
--uninstall)
    rm -f -- "$desktop_entry" "$binary"
    refresh_desktop_database
    echo "Removed $binary and $desktop_entry"
    exit 0
    ;;
"") ;;
*)
    echo "usage: $0 [--uninstall]" >&2
    exit 2
    ;;
esac

cargo build --release --locked -p e-invoice-viewer --manifest-path "$repo/Cargo.toml"
install -Dm755 "$repo/target/release/e-invoice-viewer" "$binary"

# %F: the file manager passes local paths, also for mounted network shares.
staging=$(mktemp -d)
trap 'rm -rf -- "$staging"' EXIT
staged=$staging/$app_id.desktop
cat >"$staged" <<EOF
[Desktop Entry]
Type=Application
Name=E-Invoice Viewer
GenericName=Invoice viewer
GenericName[de]=Rechnungsbetrachter
Comment=View and check electronic invoices (ZUGFeRD, Factur-X, XRechnung)
Comment[de]=Elektronische Rechnungen (ZUGFeRD, Factur-X, XRechnung) ansehen und prüfen
Exec="$binary" %F
Icon=x-office-document
Terminal=false
StartupNotify=true
Categories=Office;Viewer;
MimeType=application/pdf;application/xml;text/xml;
EOF
if command -v desktop-file-validate >/dev/null; then
    desktop-file-validate "$staged"
fi
install -Dm644 "$staged" "$desktop_entry"
refresh_desktop_database
echo "Installed $binary and $desktop_entry"
