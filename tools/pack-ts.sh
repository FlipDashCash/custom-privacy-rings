#!/usr/bin/env bash
# Packs both TS SDKs into DEST, target/pack by default, and prints the SHA-256, byte length and path of each tarball and of the ring program build.
set -euo pipefail

dest="${1:-target/pack}"
mkdir -p "$dest"
dest="$(cd "$dest" && pwd)"
cd "$(dirname "${BASH_SOURCE[0]}")/.."

# prepack builds each package, its log goes to stderr so stdout holds only the digests.
npm pack -w @heliuslabs/spr -w @heliuslabs/zolana-custom-rings --pack-destination "$dest" >&2

tarball() {
    local name version
    name="$(jq -r .name "$1/package.json")"
    version="$(jq -r .version "$1/package.json")"
    name="${name#@}"
    echo "$dest/${name//\//-}-$version.tgz"
}

describe() {
    if [[ ! -f "$1" ]]; then
        echo "$1 is missing" >&2
        exit 1
    fi
    printf '%s  %s  %s\n' "$(shasum -a 256 "$1" | awk '{ print $1 }')" "$(wc -c < "$1" | tr -d ' ')" "$1"
}

describe "$(tarball zolana/sdk-libs/ts)"
describe "$(tarball sdk-libs/ts)"
program=target/deploy/custom_ring_program.so
[[ ! -f "$program" ]] || describe "$PWD/$program"
