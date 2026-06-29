#!/usr/bin/env bash
# Fetch a pinned prebuilt PDFium dynamic library for tests and local
# development (RFC 003 §10, RFC 015 §7).
#
# Source: https://github.com/bblanchon/pdfium-binaries (BSD-licensed builds
# of Google's PDFium). The release tag is PINNED so smoke tests are
# reproducible; bump deliberately and re-run the suite.
#
# Usage:   ci/fetch-pdfium.sh [target]
#   target: linux-x64 (default) | linux-arm64 | mac-x64 | mac-arm64 | win-x64
# Output:  ci/.pdfium/<library file>
# Then:    PDF_TILE_VIEWER_PDFIUM_DIR="$(pwd)/ci/.pdfium" cargo test -p pdf_engine

set -euo pipefail

PDFIUM_RELEASE_TAG="chromium/7763"
TARGET="${1:-linux-x64}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEST_DIR="${SCRIPT_DIR}/.pdfium"

URL="https://github.com/bblanchon/pdfium-binaries/releases/download/${PDFIUM_RELEASE_TAG}/pdfium-${TARGET}.tgz"

mkdir -p "${DEST_DIR}"
TMP_TGZ="$(mktemp /tmp/pdfium-XXXXXX.tgz)"
trap 'rm -f "${TMP_TGZ}"' EXIT

echo "Fetching ${URL}"
curl -fSL --retry 3 -o "${TMP_TGZ}" "${URL}"

tar -xzf "${TMP_TGZ}" -C "${DEST_DIR}" --strip-components=1 lib 2>/dev/null \
  || tar -xzf "${TMP_TGZ}" -C "${DEST_DIR}" --strip-components=1 bin 2>/dev/null

ls -l "${DEST_DIR}"
echo "PDFium (${PDFIUM_RELEASE_TAG}, ${TARGET}) ready in ${DEST_DIR}"
echo "Run: PDF_TILE_VIEWER_PDFIUM_DIR=\"${DEST_DIR}\" cargo test -p pdf_engine"
