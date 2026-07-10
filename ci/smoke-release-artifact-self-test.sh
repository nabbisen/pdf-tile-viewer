#!/usr/bin/env bash
# Negative self-tests for ci/smoke-release-artifact.sh.
#
# These tests do not need a real PDFium library. They assert that malformed
# final archives fail before any successful production bind can occur.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

if [[ -z "${PACKAGE_ARTIFACT_SMOKE_BIN:-}" ]]; then
    cargo build --release -p app_services --bin package_artifact_smoke
    PACKAGE_ARTIFACT_SMOKE_BIN="target/release/package_artifact_smoke"
    if [[ "${OS:-}" == "Windows_NT" ]]; then
        PACKAGE_ARTIFACT_SMOKE_BIN="${PACKAGE_ARTIFACT_SMOKE_BIN}.exe"
    fi
    export PACKAGE_ARTIFACT_SMOKE_BIN
fi

TMP_PARENT="${RUNNER_TEMP:-${TMPDIR:-${REPO_ROOT}/target/tmp}}"
mkdir -p "${TMP_PARENT}"
TMP_DIR="$(mktemp -d "${TMP_PARENT%/}/pdf-tile-viewer-artifact-smoke-self-test.XXXXXX")"
cleanup() {
    rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

make_docs() {
    local root="$1"
    cp LICENSE "${root}/"
    cp NOTICE "${root}/"
    cp README.md "${root}/"
    cp CHANGELOG.md "${root}/"
}

make_archive() {
    local source_dir="$1"
    local archive="$2"
    (
        cd "${source_dir}"
        tar -czf "${archive}" .
    )
}

assert_fails() {
    local name="$1"
    local archive="$2"

    if bash ci/smoke-release-artifact.sh "${archive}" >"${TMP_DIR}/${name}.out" 2>"${TMP_DIR}/${name}.err"; then
        echo "ERROR: expected ${name} to fail" >&2
        cat "${TMP_DIR}/${name}.out" >&2
        cat "${TMP_DIR}/${name}.err" >&2
        exit 1
    fi

    echo "expected failure passed: ${name}"
}

old_parent="${TMP_DIR}/old-parent"
mkdir -p "${old_parent}/pdf-tile-viewer/bin" \
         "${old_parent}/pdf-tile-viewer/resources/pdfium/linux-x86_64"
touch "${old_parent}/pdf-tile-viewer/bin/pdf-tile-viewer"
touch "${old_parent}/pdf-tile-viewer/resources/pdfium/linux-x86_64/libpdfium.so"
make_docs "${old_parent}/pdf-tile-viewer"
make_archive "${old_parent}" "${TMP_DIR}/old-parent.tar.gz"
assert_fails "old-parent-root" "${TMP_DIR}/old-parent.tar.gz"

missing_bin="${TMP_DIR}/missing-bin"
mkdir -p "${missing_bin}/resources/pdfium/linux-x86_64"
touch "${missing_bin}/resources/pdfium/linux-x86_64/libpdfium.so"
make_docs "${missing_bin}"
make_archive "${missing_bin}" "${TMP_DIR}/missing-bin.tar.gz"
assert_fails "missing-bin" "${TMP_DIR}/missing-bin.tar.gz"

wrong_resources="${TMP_DIR}/wrong-resources"
mkdir -p "${wrong_resources}/bin" \
         "${wrong_resources}/share/resources/pdfium/linux-x86_64"
touch "${wrong_resources}/bin/pdf-tile-viewer"
touch "${wrong_resources}/share/resources/pdfium/linux-x86_64/libpdfium.so"
make_docs "${wrong_resources}"
make_archive "${wrong_resources}" "${TMP_DIR}/wrong-resources.tar.gz"
assert_fails "wrong-resources-root" "${TMP_DIR}/wrong-resources.tar.gz"

missing_pdfium="${TMP_DIR}/missing-pdfium"
mkdir -p "${missing_pdfium}/bin" \
         "${missing_pdfium}/resources/pdfium/linux-x86_64" \
         "${TMP_DIR}/dev-pdfium"
touch "${missing_pdfium}/bin/pdf-tile-viewer"
touch "${TMP_DIR}/dev-pdfium/libpdfium.so"
make_docs "${missing_pdfium}"
make_archive "${missing_pdfium}" "${TMP_DIR}/missing-pdfium.tar.gz"
PDF_TILE_VIEWER_PDFIUM_DIR="${TMP_DIR}/dev-pdfium" \
    assert_fails "production-ignores-dev-fallback" "${TMP_DIR}/missing-pdfium.tar.gz"

echo "smoke release artifact self-test passed"
