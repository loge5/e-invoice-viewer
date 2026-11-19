#!/usr/bin/env bash
# check-tag.sh TAG: succeeds if TAG is "v" followed by the version in
# Cargo.toml (spec 0020); the release workflow runs it before building.
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
version=$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"$/\1/p' "$repo/Cargo.toml")
if [[ ${1:-} != "v$version" ]]; then
    echo "error: tag '${1:-}' does not match the version in Cargo.toml (expected v$version)" >&2
    exit 1
fi
# The release also needs the lock file and the AppStream release entry of
# that version; the builds use --locked.
for crate in e-invoice-viewer e-invoice-viewer-core; do
    if ! grep -A1 -x "name = \"$crate\"" "$repo/Cargo.lock" | grep -q -x "version = \"$version\""; then
        echo "error: Cargo.lock has another version of $crate than $version; run cargo update --workspace" >&2
        exit 1
    fi
done
if ! grep -q "<release version=\"$version\"" "$repo/data/de.loges.EInvoiceViewer.metainfo.xml"; then
    echo "error: data/de.loges.EInvoiceViewer.metainfo.xml has no release entry for $version" >&2
    exit 1
fi
