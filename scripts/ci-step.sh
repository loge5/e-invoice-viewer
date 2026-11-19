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
    # What explains a failure comes first: the failure blocks of the test
    # harness, tests running too long, panics and error lines; then the end
    # of the output.
    tr -d '\r' <"$log" | awk '
        /^failures:$/ { block = 1 }
        block || /has been running for over|panicked at|^error/ { print }
        /^test result:/ { block = 0 }' | tail -n 60 >"$log.key"
    grep -v -E "^test .* \.\.\. (ok|ignored)|^ *(Compiling|Checking|Downloaded|Downloading|Updating|Locking|Adding|Fresh|Running|Doc-tests) " "$log" | tr -d '\r' | tail -n 20 >"$log.end"
    cat "$log.key" - "$log.end" <<<"----- end of output -----" | sed 's/%/%25/g' | awk -v title="$title" \
        'BEGIN { ORS = "%0A" } { print } END { ORS = "\n"; print "" }' \
        | sed "s/^/::error title=$title::/"
fi
rm -f -- "$log" "$log.key" "$log.end"
exit "$status"
