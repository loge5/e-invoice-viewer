#!/usr/bin/env bash
# Generates the Windows icon data/icons/de.loges.EInvoiceViewer.ico from the
# SVG icon (spec 0022), with the sizes Windows uses: 16, 32, 48 and 256 px.
# Run it after changing the SVG and commit the result. Needs rsvg-convert
# (librsvg) and ImageMagick.
set -euo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
svg=$repo/data/icons/de.loges.EInvoiceViewer.svg
ico=$repo/data/icons/de.loges.EInvoiceViewer.ico
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT

pngs=()
for size in 16 32 48 256; do
    rsvg-convert -w "$size" -h "$size" "$svg" -o "$work/$size.png"
    pngs+=("$work/$size.png")
done
magick "${pngs[@]}" "$ico"
echo "Wrote $ico"
