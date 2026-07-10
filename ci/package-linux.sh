#!/usr/bin/env bash
# Assemble a Linux x86-64 release artifact for PDF Tile Viewer (RFC 014).
#
# Usage:
#   ci/package-linux.sh [output_dir]
#
# Requires:
#   - Rust toolchain (stable, edition 2024)
#   - ci/.pdfium/libpdfium.so  (populate via ci/fetch-pdfium.sh)
#   - PDF_TILE_VIEWER_PDFIUM_DIR=ci/.pdfium (set for build smoke-test)
#
# Produces:
#   <output_dir>/pdf-tile-viewer-vX.X.X-linux-x64.tar.gz
#
# Archive layout:
#   bin/pdf-tile-viewer
#   resources/pdfium/linux-x86_64/libpdfium.so
#   LICENSE
#   NOTICE
#   CHANGELOG.md
#   README.md
#   notes.txt

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUTPUT_DIR="${1:-${REPO_ROOT}/dist}"

PDFIUM_DIR="${SCRIPT_DIR}/.pdfium"
PDFIUM_RELEASE_TAG="chromium/7920"

cd "${REPO_ROOT}"

# ── 0. Preflight ─────────────────────────────────────────────────────────────

if [[ ! -f "${PDFIUM_DIR}/libpdfium.so" ]]; then
    echo "ERROR: ${PDFIUM_DIR}/libpdfium.so not found."
    echo "       Run: bash ci/fetch-pdfium.sh"
    exit 1
fi

APP_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*= "//;s/"//')
echo "Packaging pdf-tile-viewer v${APP_VERSION} (linux-x64)"

# ── 1. Build ──────────────────────────────────────────────────────────────────

echo "Building (release)..."
env NO_STRIP=1 \
    PDFIUM_RELEASE_TAG="${PDFIUM_RELEASE_TAG}" \
    cargo build --release -p app

# ── 2. Stage ──────────────────────────────────────────────────────────────────

# Archive root dir uses a "v" prefix: pdf-tile-viewer-vX.X.X-linux-x64
ARTIFACT_NAME="pdf-tile-viewer-v${APP_VERSION}-linux-x64"
STAGE="${REPO_ROOT}/target/stage/${ARTIFACT_NAME}"

rm -rf "${STAGE}"
mkdir -p "${STAGE}/bin" \
         "${STAGE}/resources/pdfium/linux-x86_64"

cp "target/release/pdf-tile-viewer" "${STAGE}/bin/"
cp "${PDFIUM_DIR}/libpdfium.so"     "${STAGE}/resources/pdfium/linux-x86_64/"
cp LICENSE                          "${STAGE}/"
cp NOTICE                           "${STAGE}/"
cp CHANGELOG.md                     "${STAGE}/"
cp README.md                        "${STAGE}/"

cat > "${STAGE}/notes.txt" << NOTES
PDF Tile Viewer v${APP_VERSION} — Linux x86-64

This is an unsigned build.
PDFium: ${PDFIUM_RELEASE_TAG} (dynamic, bundled in resources/)

Launch:
  cd <extraction-directory>
  ./bin/pdf-tile-viewer

The binary expects the bundled resources/ directory to remain at the
top level of this extracted archive. Do not move bin/pdf-tile-viewer
away from the extracted resources/ directory.

Troubleshooting:
  If PDFium fails to load, check resources/pdfium/linux-x86_64/libpdfium.so.
  Re-extract the full archive if the resources/ directory is missing.

Report issues: https://github.com/nabbisen/pdf-tile-viewer/issues
NOTES

# ── 3. Package smoke test ────────────────────────────────────────────────────

echo "Running engine smoke tests against bundled PDFium..."
PDF_TILE_VIEWER_PDFIUM_DIR="${PDFIUM_DIR}" \
    cargo test -p pdf_engine --test smoke --release -- --test-threads=1

echo "Smoke tests passed."

# ── 4. Archive ───────────────────────────────────────────────────────────────
#
# Archive the staged contents, not the staging directory itself. Project
# release archives unpack flat into the extraction destination.

mkdir -p "${OUTPUT_DIR}"
(
    cd "${STAGE}"
    tar -czf "${OUTPUT_DIR}/${ARTIFACT_NAME}.tar.gz" .
)

echo "Running packaged artifact smoke test..."
bash ci/smoke-release-artifact.sh "${OUTPUT_DIR}/${ARTIFACT_NAME}.tar.gz"

echo "Artifact: ${OUTPUT_DIR}/${ARTIFACT_NAME}.tar.gz"
ls -lh "${OUTPUT_DIR}/${ARTIFACT_NAME}.tar.gz"
