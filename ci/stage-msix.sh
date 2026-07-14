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
#
# The Visual C++ runtime framework dependency is normally resolved from the
# Windows SDK by msix-store.yml. The defaults below keep local staging usable
# when the script is run by hand.

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
# Strip any pre-release / build suffix: 2.1.0-beta.1 -> 2.1.0
SEMVER_CORE="${FULL_VERSION%%-*}"
REVISION="${MSIX_REVISION:-0}"
MSIX_VERSION="${SEMVER_CORE}.${REVISION}"
VCLIBS_NAME="${MSIX_VCLIBS_NAME:-Microsoft.VCLibs.140.00.UWPDesktop}"
VCLIBS_MIN_VERSION="${MSIX_VCLIBS_MIN_VERSION:-14.0.27323.0}"
VCLIBS_PUBLISHER="${MSIX_VCLIBS_PUBLISHER:-CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US}"

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
sed \
    -e "s/@VERSION@/${MSIX_VERSION}/g" \
    -e "s/@VCLIBS_NAME@/${VCLIBS_NAME}/g" \
    -e "s/@VCLIBS_MIN_VERSION@/${VCLIBS_MIN_VERSION}/g" \
    -e "s/@VCLIBS_PUBLISHER@/${VCLIBS_PUBLISHER}/g" \
    "${MANIFEST_SRC}" > "${OUTPUT_DIR}/AppxManifest.xml"

echo "Staged MSIX root at ${OUTPUT_DIR}:"
find "${OUTPUT_DIR}" -type f | sort
