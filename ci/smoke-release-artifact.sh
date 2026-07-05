#!/usr/bin/env bash
# Verify a final release archive after compression (RFC 018).
#
# Usage:
#   ci/smoke-release-artifact.sh <artifact.tar.gz|artifact.zip>
#
# The smoke extracts the archive into a fresh temp directory and treats that
# directory as the artifact root. Release archives intentionally unpack flat:
# bin/, resources/, and docs must appear directly at the extraction root.
# The helper uses production bundled-resource resolution, not
# PDF_TILE_VIEWER_PDFIUM_DIR.

set -euo pipefail

ARCHIVE="${1:?usage: ci/smoke-release-artifact.sh <artifact.tar.gz|artifact.zip>}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

if [[ ! -f "${ARCHIVE}" ]]; then
    echo "ERROR: artifact archive not found: ${ARCHIVE}" >&2
    exit 1
fi

TMP_PARENT="${RUNNER_TEMP:-${TMPDIR:-${REPO_ROOT}/target/tmp}}"
mkdir -p "${TMP_PARENT}"
TMP_DIR="$(mktemp -d "${TMP_PARENT%/}/pdf-tile-viewer-artifact-smoke.XXXXXX")"
cleanup() {
    rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

case "${ARCHIVE}" in
    *.tar.gz|*.tgz)
        tar -xzf "${ARCHIVE}" -C "${TMP_DIR}"
        ;;
    *.zip)
        if command -v python3 >/dev/null 2>&1; then
            python3 -m zipfile -e "${ARCHIVE}" "${TMP_DIR}"
        else
            python -m zipfile -e "${ARCHIVE}" "${TMP_DIR}"
        fi
        ;;
    *)
        echo "ERROR: unsupported artifact extension: ${ARCHIVE}" >&2
        exit 1
        ;;
esac

if [[ ! -d "${TMP_DIR}/bin" || ! -d "${TMP_DIR}/resources" ]]; then
    echo "ERROR: archive must unpack flat with bin/ and resources/ at archive root" >&2
    find "${TMP_DIR}" -mindepth 1 -maxdepth 1 -print >&2
    exit 1
fi

cd "${REPO_ROOT}"
if [[ -n "${PACKAGE_ARTIFACT_SMOKE_BIN:-}" ]]; then
    "${PACKAGE_ARTIFACT_SMOKE_BIN}" "${TMP_DIR}"
else
    cargo run --release -p app_services --bin package_artifact_smoke -- "${TMP_DIR}"
fi
