#!/usr/bin/env bash
# Fetch a pinned prebuilt PDFium dynamic library for tests and local
# development (RFC 003 §10, RFC 015 §7).
#
# Source: https://github.com/bblanchon/pdfium-binaries (BSD-licensed builds
# of Google's PDFium). The release tag is PINNED so smoke tests are
# reproducible; bump deliberately and re-run the suite. Every download is
# verified against ci/pdfium-checksums.txt (SHA-256) before extraction.
#
# Usage:   ci/fetch-pdfium.sh [target]
#   target: linux-x64 (default) | linux-arm64 | mac-x64 | mac-arm64 | win-x64
# Output:  ci/.pdfium/<library file>
# Then:    PDF_TILE_VIEWER_PDFIUM_DIR="$(pwd)/ci/.pdfium" cargo test -p pdf_engine

set -euo pipefail

PDFIUM_RELEASE_TAG="chromium/7920"
TARGET="${1:-linux-x64}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEST_DIR="${SCRIPT_DIR}/.pdfium"

URL="https://github.com/bblanchon/pdfium-binaries/releases/download/${PDFIUM_RELEASE_TAG}/pdfium-${TARGET}.tgz"

# The dynamic library inside the archive: lib/ on unix, bin/ on Windows (where lib/
# only holds the import library).
case "${TARGET}" in
    linux-*) LIB_FILE="libpdfium.so" ;;
    mac-*)   LIB_FILE="libpdfium.dylib" ;;
    win-*)   LIB_FILE="pdfium.dll" ;;
    *) echo "ERROR: unsupported target '${TARGET}'" >&2; exit 1 ;;
esac

mkdir -p "${DEST_DIR}"
TMP_TGZ="$(mktemp /tmp/pdfium.XXXXXX)"  # no suffix after the Xs (BSD/macOS mktemp); /tmp also on Windows (git bash)
TMP_EXTRACT="$(mktemp -d /tmp/pdfium-extract.XXXXXX)"
trap 'rm -rf "${TMP_TGZ}" "${TMP_EXTRACT}"' EXIT

echo "Fetching ${URL}"
curl -fSL --retry 3 -o "${TMP_TGZ}" "${URL}"

# Fail closed on a wrong or unpinned archive, before anything is extracted (RFC 036 § 5.1).
bash "${SCRIPT_DIR}/verify-sha256.sh" "${TMP_TGZ}" "${PDFIUM_RELEASE_TAG}" "pdfium-${TARGET}.tgz"

tar -xzf "${TMP_TGZ}" -C "${TMP_EXTRACT}"
if [[ -f "${TMP_EXTRACT}/lib/${LIB_FILE}" ]]; then
    cp "${TMP_EXTRACT}/lib/${LIB_FILE}" "${DEST_DIR}/"
elif [[ -f "${TMP_EXTRACT}/bin/${LIB_FILE}" ]]; then
    cp "${TMP_EXTRACT}/bin/${LIB_FILE}" "${DEST_DIR}/"
else
    echo "ERROR: ${LIB_FILE} not found in pdfium-${TARGET}.tgz" >&2
    exit 1
fi

ls -l "${DEST_DIR}"
echo "PDFium (${PDFIUM_RELEASE_TAG}, ${TARGET}) ready in ${DEST_DIR}"
echo "Run: PDF_TILE_VIEWER_PDFIUM_DIR=\"${DEST_DIR}\" cargo test -p pdf_engine"
