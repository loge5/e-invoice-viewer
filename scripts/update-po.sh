#!/usr/bin/env bash
# Extracts the texts marked for translation (ADR 0003) into po/messages.pot
# and merges them into each po/<language>.po. New texts then appear there
# untranslated; the test german_catalogue_is_complete fails until they are
# translated.
#
# Usage: scripts/update-po.sh
set -euo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo"

# Keywords as in crates/core/src/i18n.rs (XGETTEXT_ARGS).
mapfile -t sources < <(find crates/core/src crates/app/src -name '*.rs' | sort)
xgettext --language=Rust --from-code=UTF-8 \
    --keyword=tr:2 --keyword=tr_n:2,3 --keyword=n_ --keyword=term:2 \
    --package-name="E-Invoice Viewer" --no-location \
    -o po/messages.pot "${sources[@]}"

for po in po/*.po; do
    msgmerge --quiet --update --backup=none --no-fuzzy-matching --no-location "$po" po/messages.pot
    msgattrib --no-obsolete --output-file="$po" "$po"
done
