#!/usr/bin/env bash
# Builds the AppImage (spec 0020, ADR 0004) in an Ubuntu 24.04 container:
#   target/appimage/E-Invoice_Viewer-<version>-x86_64.AppImage
#   target/appimage/bundled-packages.txt
# Needs Docker. The files in target/appimage belong to the calling user.
set -euo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
# Ubuntu packages are taken from this date of snapshot.ubuntu.com.
snapshot=20261117T000000Z
version=$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"$/\1/p' "$repo/Cargo.toml")
rust_version=$(sed -n '/^\[workspace.package\]/,/^\[/s/^rust-version = "\(.*\)"$/\1/p' "$repo/Cargo.toml")
image=e-invoice-viewer-appimage:$snapshot-$rust_version

docker build --tag "$image" \
    --build-arg SNAPSHOT="$snapshot" --build-arg RUST_VERSION="$rust_version" \
    "$repo/scripts/appimage"

mkdir -p "$repo/target/appimage"
docker run --rm --user "$(id -u):$(id -g)" \
    --volume "$repo:/src" --workdir /src --env HOME=/tmp \
    --env SOURCE_DATE_EPOCH="$(git -C "$repo" log -1 --format=%ct)" \
    "$image" scripts/appimage/build.sh "$version"

appimage=$repo/target/appimage/E-Invoice_Viewer-$version-x86_64.AppImage
test -x "$appimage"
echo "Built $appimage"
