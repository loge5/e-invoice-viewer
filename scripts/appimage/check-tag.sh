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
