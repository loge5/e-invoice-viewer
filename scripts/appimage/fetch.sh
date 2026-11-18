# shellcheck shell=bash
# fetch NAME URL SHA256 DEST: downloads URL to DEST and checks its SHA-256.
# On a mismatch, nothing is left at DEST and the build stops (spec 0020).
fetch() {
    local name=$1 url=$2 sha256=$3 dest=$4
    rm -f -- "$dest" "$dest.part"
    if ! curl -fsSL --retry 3 -o "$dest.part" "$url"; then
        rm -f -- "$dest.part"
        echo "error: cannot download $name from $url" >&2
        return 1
    fi
    if ! echo "$sha256  $dest.part" | sha256sum --check --status; then
        rm -f -- "$dest.part"
        echo "error: wrong SHA-256 for $name ($url), expected $sha256" >&2
        return 1
    fi
    mv -- "$dest.part" "$dest"
}
