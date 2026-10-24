#!/usr/bin/env bash
# Stage an MSIX package layout for Microsoft Store submission (Windows only).
#
# This does NOT call makeappx/signtool itself — those live in the Windows SDK
# and are invoked by the CI workflow (or a local Windows shell). This script
# only assembles the package root so the layout is identical everywhere.
#
# Usage:
#   ci/stage-msix.sh <built_exe> <pdfium_dll> <output_dir>
#
# Example (from a Windows runner, bash shell):
#   ci/stage-msix.sh \
#     target/x86_64-pc-windows-msvc/release/pdf-tile-viewer.exe \
#     resources/pdfium/windows-x86_64/pdfium.dll \
#     dist/msix-root
#
# Produces <output_dir> containing:
#   pdf-tile-viewer.exe
#   AppxManifest.xml          (copied unchanged; its version is VERIFIED, not substituted)
#   Assets/                   (Store logos)
#   resources/pdfium/windows-x86_64/pdfium.dll
# and, next to <output_dir>, a file msix-version.txt holding the package version
# (single source for the .msix file name in msix-store.yml).
#
# The MSIX version is always "<Cargo core X.Y.Z>.0" (RFC 036 § 5.7): the Microsoft
# Store reserves the fourth segment, and a pre-release suffix is not permitted.
# ci/set-version.sh writes it into the manifest; this script fails if it differs.

set -euo pipefail

BUILT_EXE="${1:?usage: stage-msix.sh <built_exe> <pdfium_dll> <output_dir>}"
PDFIUM_DLL="${2:?missing pdfium dll path}"
OUTPUT_DIR="${3:?missing output dir}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
MANIFEST_SRC="${REPO_ROOT}/packaging/windows/AppxManifest.xml"
ASSETS_SRC="${REPO_ROOT}/packaging/windows/Assets"

# ── Expected MSIX version from Cargo.toml, verified against the manifest ─────
FULL_VERSION=$(grep '^version' "${REPO_ROOT}/Cargo.toml" | head -1 | sed 's/.*= *"//;s/".*//')
SEMVER_CORE="${FULL_VERSION%%-*}"
MSIX_VERSION="${SEMVER_CORE}.0"
MANIFEST_VERSION="$(grep -o -E '[[:space:]]Version="[^"]*"' "${MANIFEST_SRC}" | sed -E 's/.*"([^"]*)"/\1/')"

echo "Cargo version    : ${FULL_VERSION}"
echo "MSIX version     : ${MSIX_VERSION}"
echo "Manifest version : ${MANIFEST_VERSION}"
if [[ "${MANIFEST_VERSION}" != "${MSIX_VERSION}" ]]; then
    echo "ERROR: manifest version '${MANIFEST_VERSION}' != '${MSIX_VERSION}' (run ci/set-version.sh)" >&2
    exit 1
fi

# ── Assemble the package root ────────────────────────────────────────────────
rm -rf "${OUTPUT_DIR}"
mkdir -p "${OUTPUT_DIR}/Assets" \
         "${OUTPUT_DIR}/resources/pdfium/windows-x86_64"

cp "${BUILT_EXE}"  "${OUTPUT_DIR}/pdf-tile-viewer.exe"
cp "${PDFIUM_DLL}" "${OUTPUT_DIR}/resources/pdfium/windows-x86_64/pdfium.dll"
cp "${ASSETS_SRC}"/*.png "${OUTPUT_DIR}/Assets/"

cp "${MANIFEST_SRC}" "${OUTPUT_DIR}/AppxManifest.xml"
printf '%s\n' "${MSIX_VERSION}" > "$(dirname "${OUTPUT_DIR}")/msix-version.txt"

echo "Staged MSIX root at ${OUTPUT_DIR}:"
find "${OUTPUT_DIR}" -type f | sort
