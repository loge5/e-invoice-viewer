#!/usr/bin/env bash
# Produces the expected results of the validation regression tests (specs 0009
# and 0010) with the official KoSIT validator. Development tool only: needs Java; the
# app and the tests do not.
#
# Usage:
#   scripts/kosit-reference.sh <validator-standalone.jar> <xrechnung-config-dir> <cen-checkout>
#
# - validator jar:   https://github.com/itplr-kosit/validator/releases (1.6.3)
# - config dir:      unpacked validator-configuration-xrechnung release
#                    (2026-08-31, see crates/core/rules/kosit-config-*)
# - cen checkout:    https://github.com/ConnectingEurope/eInvoicing-EN16931
#                    at the tag in crates/core/rules/cen-*/SOURCE.md
#
# Inputs: the KoSIT test instances, the samples and the public sample
# collections (spec 0025) under crates/core/tests/fixtures
# and the variants written by
#   cargo test -p e-invoice-viewer-core --test validate_reference -- --ignored write_variants
# Output, in crates/core/tests/fixtures/kosit/:
# - expected-en16931.tsv:  EN 16931 rules only (scripts/kosit/scenarios-en16931.xml)
# - expected-xrechnung.tsv: the configuration's own scenarios, with the CEN
#   rules replaced by the bundled CEN release (the configuration ships an
#   older one); only invoices assigned to an XRechnung scenario
set -euo pipefail

jar=$(realpath "$1"); config=$(realpath "$2"); cen=$(realpath "$3")
root=$(cd "$(dirname "$0")/.." && pwd)
fixtures="$root/crates/core/tests/fixtures"
variants="$root/target/validation-variants"
out="$fixtures/kosit/expected-en16931.tsv"
out_xr="$fixtures/kosit/expected-xrechnung.tsv"

[ -d "$variants" ] || { echo "missing $variants, run the write_variants test first" >&2; exit 1; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/repo/resources/cen"
cp -r "$config/resources/cii" "$config/resources/ubl" "$work/repo/resources/"
cp "$config"/resources/*.xsl "$work/repo/resources/"
cp "$cen/cii/xslt/EN16931-CII-validation.xslt" "$cen/ubl/xslt/EN16931-UBL-validation.xslt" "$work/repo/resources/cen/"

# The XRechnung configuration as released, with the bundled CEN rules.
cp -r "$config" "$work/xr-repo"
cp "$cen/cii/xslt/EN16931-CII-validation.xslt" "$work/xr-repo/resources/cii/16b/xsl/EN16931-CII-validation.xsl"
cp "$cen/ubl/xslt/EN16931-UBL-validation.xslt" "$work/xr-repo/resources/ubl/2.1/xsl/EN16931-UBL-validation.xsl"

run_group() { # <mode: en16931|xrechnung> <directory> <key prefix>
    local mode=$1 dir=$2 prefix=$3 reports repo scenarios flag=()
    reports="$work/reports-$mode/$(echo "$prefix" | tr '/' '_')"
    mkdir -p "$reports"
    if [ "$mode" = xrechnung ]; then
        repo="$work/xr-repo"; scenarios="$work/xr-repo/scenarios.xml"; flag=(--xrechnung)
    else
        repo="$work/repo"; scenarios="$root/scripts/kosit/scenarios-en16931.xml"
    fi
    java -jar "$jar" -r "$repo" -s "$scenarios" \
        -o "$reports" "$dir"/*.xml > "$reports.log" 2>&1 || true
    python3 -I "$root/scripts/kosit/extract.py" "${flag[@]}" "$reports" "$prefix" "$dir"/*.xml
}

run_all() { # <mode>
    run_group "$1" "$fixtures/cii" "cii/"
    run_group "$1" "$fixtures/ubl" "ubl/"
    for group in cii ubl-inv ubl-cn; do
        run_group "$1" "$fixtures/kosit/$group" "kosit/$group/"
    done
    run_group "$1" "$variants" "variants/"
    # Public sample collections (spec 0025).
    run_group "$1" "$fixtures/public/xrechnung-testsuite/standard" "public/xrechnung-testsuite/standard/"
    run_group "$1" "$fixtures/public/xrechnung-testsuite/extension" "public/xrechnung-testsuite/extension/"
    run_group "$1" "$fixtures/public/cen/cii" "public/cen/cii/"
    run_group "$1" "$fixtures/public/cen/ubl" "public/cen/ubl/"
}

run_all en16931 | LC_ALL=C sort > "$out"
echo "wrote $out ($(wc -l < "$out") invoices)"
run_all xrechnung 2>/dev/null | LC_ALL=C sort > "$out_xr"
echo "wrote $out_xr ($(wc -l < "$out_xr") invoices)"
