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
#   dist/          (release outputs)
#   .git-exclude/  (local review/scratch data)
#   ci/.pdfium/    (downloaded native library — too large, not source)
#   .git/          (VCS objects)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPO_DIR_NAME="$(basename "${REPO_ROOT}")"
OUTPUT_DIR="${1:-${REPO_ROOT}/dist}"

APP_VERSION=$(grep '^version' "${REPO_ROOT}/Cargo.toml" | head -1 | sed 's/.*= "//;s/"//')
ARCHIVE_ROOT="pdf-tile-viewer-v${APP_VERSION}"
ARCHIVE_FILE="${ARCHIVE_ROOT}.tar.gz"

mkdir -p "${OUTPUT_DIR}"
OUTPUT_ABS="$(cd "${OUTPUT_DIR}" && pwd)"
TMP_ARCHIVE="$(mktemp "${OUTPUT_ABS}/${ARCHIVE_FILE}.tmp.XXXXXX")"
trap 'rm -f "${TMP_ARCHIVE}"' EXIT

TAR_EXCLUDES=(
    --exclude="${REPO_DIR_NAME}/target"
    --exclude="${REPO_DIR_NAME}/dist"
    --exclude="${REPO_DIR_NAME}/.git-exclude"
    --exclude="${REPO_DIR_NAME}/ci/.pdfium"
    --exclude="${REPO_DIR_NAME}/.git"
)

if [[ "${OUTPUT_ABS}" == "${REPO_ROOT}/"* ]]; then
    OUTPUT_REL="${OUTPUT_ABS#"${REPO_ROOT}/"}"
    TAR_EXCLUDES+=(--exclude="${REPO_DIR_NAME}/${OUTPUT_REL}")
elif [[ "${OUTPUT_ABS}" == "${REPO_ROOT}" ]]; then
    TAR_EXCLUDES+=(--exclude="${REPO_DIR_NAME}/$(basename "${TMP_ARCHIVE}")")
fi

# GNU tar --transform renames the top-level directory inside the archive
# to "pdf-tile-viewer-vX.X.X" so extraction gives a self-contained,
# version-stamped directory.
tar \
    "${TAR_EXCLUDES[@]}" \
    --transform="s|^${REPO_DIR_NAME}|${ARCHIVE_ROOT}|" \
    -czf "${TMP_ARCHIVE}" \
    -C "$(dirname "${REPO_ROOT}")" \
    "${REPO_DIR_NAME}"

mv "${TMP_ARCHIVE}" "${OUTPUT_DIR}/${ARCHIVE_FILE}"
trap - EXIT

echo "Source archive: ${OUTPUT_DIR}/${ARCHIVE_FILE}"
ls -lh "${OUTPUT_DIR}/${ARCHIVE_FILE}"
echo ""
echo "Verify layout:"
tar -tzf "${OUTPUT_DIR}/${ARCHIVE_FILE}" | awk 'NR <= 12 { print }'
