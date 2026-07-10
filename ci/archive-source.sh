#!/usr/bin/env bash
# Build a source-only archive of the Cargo workspace.
#
# Usage:
#   ci/archive-source.sh [output_dir]
#
# Produces:
#   <output_dir>/pdf-tile-viewer-vX.X.X.tar.gz
#
# Archive layout (flat — no nested pdf-tile-viewer/ subdirectory):
#   pdf-tile-viewer-vX.X.X/
#     Cargo.toml
#     Cargo.lock
#     crates/
#     rfcs/
#     docs/
#     fixtures/
#     ci/
#     README.md
#     CHANGELOG.md
#     ROADMAP.md
#     LICENSE
#     NOTICE
#     ...
#
# Excluded:
#   target/        (build artefacts)
#   ci/.pdfium/    (downloaded native library — too large, not source)
#   .git/          (VCS objects)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUTPUT_DIR="${1:-${REPO_ROOT}/dist}"

APP_VERSION=$(grep '^version' "${REPO_ROOT}/Cargo.toml" | head -1 | sed 's/.*= "//;s/"//')
ARCHIVE_ROOT="pdf-tile-viewer-v${APP_VERSION}"
ARCHIVE_FILE="${ARCHIVE_ROOT}.tar.gz"

mkdir -p "${OUTPUT_DIR}"

# GNU tar --transform renames the top-level directory inside the archive
# from "pdf-tile-viewer" to "pdf-tile-viewer-vX.X.X" so extraction gives
# a self-contained, version-stamped directory.
tar \
    --exclude='pdf-tile-viewer/target' \
    --exclude='pdf-tile-viewer/ci/.pdfium' \
    --exclude='pdf-tile-viewer/.git' \
    --transform="s|^pdf-tile-viewer|${ARCHIVE_ROOT}|" \
    -czf "${OUTPUT_DIR}/${ARCHIVE_FILE}" \
    -C "$(dirname "${REPO_ROOT}")" \
    "$(basename "${REPO_ROOT}")"

echo "Source archive: ${OUTPUT_DIR}/${ARCHIVE_FILE}"
ls -lh "${OUTPUT_DIR}/${ARCHIVE_FILE}"
echo ""
echo "Verify layout:"
tar -tzf "${OUTPUT_DIR}/${ARCHIVE_FILE}" | head -12
