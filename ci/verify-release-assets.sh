#!/usr/bin/env bash
# Release-asset gate (RFC 036 § 5.4a): exactly the five expected, correctly
# versioned files may reach a GitHub Release.
#
# Usage: ci/verify-release-assets.sh <dir> <version>
#
# <dir> must contain exactly:
#   pdf-tile-viewer-v<version>-linux-x64.tar.gz
#   pdf-tile-viewer-v<version>-macos-arm64.tar.gz
#   pdf-tile-viewer-v<version>-macos-x64.tar.gz
#   pdf-tile-viewer-v<version>-windows-x64.zip
#   pdf-tile-viewer-v<version>.tar.gz              (source archive)

set -euo pipefail

DIR="${1:?usage: verify-release-assets.sh <dir> <version>}"
VERSION="${2:?missing version}"

fail() { echo "verify-release-assets: $*" >&2; exit 1; }

[[ -d "${DIR}" ]] || fail "not a directory: ${DIR}"

EXPECTED=(
    "pdf-tile-viewer-v${VERSION}-linux-x64.tar.gz"
    "pdf-tile-viewer-v${VERSION}-macos-arm64.tar.gz"
    "pdf-tile-viewer-v${VERSION}-macos-x64.tar.gz"
    "pdf-tile-viewer-v${VERSION}-windows-x64.zip"
    "pdf-tile-viewer-v${VERSION}.tar.gz"
)

status=0
for name in "${EXPECTED[@]}"; do
    if [[ ! -f "${DIR}/${name}" ]]; then
        echo "verify-release-assets: MISSING ${name}" >&2
        status=1
    fi
done
# Anything else in the directory (stray file, other version, sub-directory) fails.
while IFS= read -r found; do
    known=0
    for name in "${EXPECTED[@]}"; do
        [[ "${found}" == "${name}" ]] && known=1
    done
    if [[ "${known}" == 0 ]]; then
        echo "verify-release-assets: UNEXPECTED ${found}" >&2
        status=1
    fi
done < <(find "${DIR}" -mindepth 1 -maxdepth 1 -exec basename {} \; | sort)

if [[ "${status}" != 0 ]]; then
    fail "release assets do not match the expected set for version ${VERSION}"
fi
echo "verify-release-assets: OK ${#EXPECTED[@]} assets for ${VERSION}:"
for name in "${EXPECTED[@]}"; do ls -l "${DIR}/${name}"; done
