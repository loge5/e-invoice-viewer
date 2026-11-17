#!/usr/bin/env bash
# Produces the expected results of the EN 16931 regression test (spec 0009)
# with the official KoSIT validator. Development tool only: needs Java; the
# app and the tests do not.
#
# Usage:
#   scripts/kosit-reference.sh <validator-standalone.jar> <xrechnung-config-dir> <cen-checkout>
#
# - validator jar:   https://github.com/itplr-kosit/validator/releases (1.6.3)
# - config dir:      unpacked validator-configuration-xrechnung release
#                    (only its XML schemas and report stylesheets are used)
# - cen checkout:    https://github.com/ConnectingEurope/eInvoicing-EN16931
#                    at the tag in crates/core/rules/cen-*/SOURCE.md
#
# Inputs: the KoSIT test instances and samples under crates/core/tests/fixtures
# and the variants written by
#   cargo test -p e-invoice-viewer-core --test validate_en16931 -- --ignored write_variants
# Output: crates/core/tests/fixtures/kosit/expected-en16931.tsv
set -euo pipefail

jar=$(realpath "$1"); config=$(realpath "$2"); cen=$(realpath "$3")
root=$(cd "$(dirname "$0")/.." && pwd)
fixtures="$root/crates/core/tests/fixtures"
variants="$root/target/validation-variants"
out="$fixtures/kosit/expected-en16931.tsv"

[ -d "$variants" ] || { echo "missing $variants, run the write_variants test first" >&2; exit 1; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/repo/resources/cen"
cp -r "$config/resources/cii" "$config/resources/ubl" "$work/repo/resources/"
cp "$config"/resources/*.xsl "$work/repo/resources/"
cp "$cen/cii/xslt/EN16931-CII-validation.xslt" "$cen/ubl/xslt/EN16931-UBL-validation.xslt" "$work/repo/resources/cen/"

run_group() { # <directory> <key prefix>
    local dir=$1 prefix=$2 reports
    reports="$work/reports/$(echo "$prefix" | tr '/' '_')"
    mkdir -p "$reports"
    java -jar "$jar" -r "$work/repo" -s "$root/scripts/kosit/scenarios-en16931.xml" \
        -o "$reports" "$dir"/*.xml > "$reports.log" 2>&1 || true
    python3 -I "$root/scripts/kosit/extract.py" "$reports" "$prefix" "$dir"/*.xml
}

{
    run_group "$fixtures/cii" "cii/"
    run_group "$fixtures/ubl" "ubl/"
    for group in cii ubl-inv ubl-cn; do
        run_group "$fixtures/kosit/$group" "kosit/$group/"
    done
    run_group "$variants" "variants/"
} | LC_ALL=C sort > "$out"
echo "wrote $out ($(wc -l < "$out") invoices)"
