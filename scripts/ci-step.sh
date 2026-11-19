#!/usr/bin/env bash
# Runs a build step in a workflow (CI or release) and, if it fails, reports the
# last lines of its output as a workflow annotation, which can be read
# without signing in to GitHub (specs 0022, 0026).
#
#   scripts/ci-step.sh <title> <command> [args…]
set -uo pipefail

title=$1
shift
log=$(mktemp)
"$@" 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [[ $status -ne 0 ]]; then
    # Annotations take %0A as line break; % must be escaped first.
    grep -v -E "^test .* \.\.\. (ok|ignored)" "$log" | tail -n 150 | tr -d '\r' | sed 's/%/%25/g' | awk -v title="$title" \
        'BEGIN { ORS = "%0A" } { print } END { ORS = "\n"; print "" }' \
        | sed "s/^/::error title=$title::/"
fi
rm -f -- "$log"
exit "$status"
