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
#   AppxManifest.xml          (version substituted from Cargo.toml)
#   Assets/                   (Store logos)
#   resources/pdfium/windows-x86_64/pdfium.dll
#
# The MSIX Version is a 4-part numeric X.Y.Z.R. Pre-release suffixes
# (-beta.N etc.) are NOT permitted by MSIX, so only the semver core is used.
# The 4th part (revision) defaults to 0; override with MSIX_REVISION.

set -euo pipefail

BUILT_EXE="${1:?usage: stage-msix.sh <built_exe> <pdfium_dll> <output_dir>}"
PDFIUM_DLL="${2:?missing pdfium dll path}"
OUTPUT_DIR="${3:?missing output dir}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
MANIFEST_SRC="${REPO_ROOT}/packaging/windows/AppxManifest.xml"
ASSETS_SRC="${REPO_ROOT}/packaging/windows/Assets"

# ── Derive the MSIX 4-part version from Cargo.toml ───────────────────────────
FULL_VERSION=$(grep '^version' "${REPO_ROOT}/Cargo.toml" | head -1 | sed 's/.*= *"//;s/".*//')
# Strip any pre-release / build suffix: 2.0.0-beta.7 -> 2.0.0
SEMVER_CORE="${FULL_VERSION%%-*}"
REVISION="${MSIX_REVISION:-0}"
MSIX_VERSION="${SEMVER_CORE}.${REVISION}"

echo "Cargo version : ${FULL_VERSION}"
echo "MSIX version  : ${MSIX_VERSION}"

# ── Assemble the package root ────────────────────────────────────────────────
rm -rf "${OUTPUT_DIR}"
mkdir -p "${OUTPUT_DIR}/Assets" \
         "${OUTPUT_DIR}/resources/pdfium/windows-x86_64"

cp "${BUILT_EXE}"  "${OUTPUT_DIR}/pdf-tile-viewer.exe"
cp "${PDFIUM_DLL}" "${OUTPUT_DIR}/resources/pdfium/windows-x86_64/pdfium.dll"
cp "${ASSETS_SRC}"/*.png "${OUTPUT_DIR}/Assets/"

# Substitute the version placeholder in the manifest.
sed "s/@VERSION@/${MSIX_VERSION}/" "${MANIFEST_SRC}" > "${OUTPUT_DIR}/AppxManifest.xml"

echo "Staged MSIX root at ${OUTPUT_DIR}:"
find "${OUTPUT_DIR}" -type f | sort
