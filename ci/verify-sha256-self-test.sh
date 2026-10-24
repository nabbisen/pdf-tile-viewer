#!/usr/bin/env bash
# Self-test for ci/verify-sha256.sh: match, mismatch, missing entry, missing file,
# malformed entry. Needs no network.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TMP_PARENT="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
TMP_DIR="$(mktemp -d "${TMP_PARENT%/}/verify-sha256-self-test.XXXXXX")"
trap 'rm -rf "${TMP_DIR}"' EXIT

printf 'pdfium test payload\n' > "${TMP_DIR}/asset.tgz"
if command -v sha256sum >/dev/null 2>&1; then
    GOOD="$(sha256sum "${TMP_DIR}/asset.tgz" | awk '{print $1}')"
else
    GOOD="$(shasum -a 256 "${TMP_DIR}/asset.tgz" | awk '{print $1}')"
fi
BAD="$(printf '%064d' 0)"

export PDFIUM_CHECKSUMS_FILE="${TMP_DIR}/checksums.txt"
{
    echo "# comment line"
    echo "${GOOD}  chromium/1  asset.tgz"
    echo "${BAD}  chromium/2  asset.tgz"
    echo "nothex  chromium/3  asset.tgz"
} > "${PDFIUM_CHECKSUMS_FILE}"

expect_ok() {
    local name="$1"; shift
    if bash "${SCRIPT_DIR}/verify-sha256.sh" "$@" >"${TMP_DIR}/${name}.out" 2>"${TMP_DIR}/${name}.err"; then
        echo "expected success passed: ${name}"
    else
        echo "ERROR: expected ${name} to succeed" >&2
        cat "${TMP_DIR}/${name}.err" >&2
        exit 1
    fi
}
expect_fail() {
    # expect_fail NAME EXPECTED_STDERR_FRAGMENT ARGS...  (the failure must be for the stated reason)
    local name="$1" fragment="$2"; shift 2
    if bash "${SCRIPT_DIR}/verify-sha256.sh" "$@" >"${TMP_DIR}/${name}.out" 2>"${TMP_DIR}/${name}.err"; then
        echo "ERROR: expected ${name} to fail" >&2
        cat "${TMP_DIR}/${name}.out" >&2
        exit 1
    fi
    if ! grep -q -- "${fragment}" "${TMP_DIR}/${name}.err"; then
        echo "ERROR: ${name} failed for the wrong reason (wanted '${fragment}'):" >&2
        cat "${TMP_DIR}/${name}.err" >&2
        exit 1
    fi
    echo "expected failure passed: ${name}"
}

expect_ok   match            "${TMP_DIR}/asset.tgz" chromium/1 asset.tgz
expect_fail mismatch         MISMATCH "${TMP_DIR}/asset.tgz" chromium/2 asset.tgz
expect_fail missing-tag      "no pinned checksum" "${TMP_DIR}/asset.tgz" chromium/9 asset.tgz
expect_fail missing-asset    "no pinned checksum" "${TMP_DIR}/asset.tgz" chromium/1 other.tgz
expect_fail missing-file     "file not found" "${TMP_DIR}/nope.tgz"  chromium/1 asset.tgz
expect_fail malformed-entry  "malformed checksum" "${TMP_DIR}/asset.tgz" chromium/3 asset.tgz
expect_fail substring-tag    "no pinned checksum" "${TMP_DIR}/asset.tgz" chromium/ asset.tgz
rm -f "${PDFIUM_CHECKSUMS_FILE}"
expect_fail missing-checksums "checksum file not found" "${TMP_DIR}/asset.tgz" chromium/1 asset.tgz

# The committed checksum file must have a well-formed entry for every pinned asset.
unset PDFIUM_CHECKSUMS_FILE
while read -r hash _tag _asset; do
    [[ -z "${hash}" || "${hash}" == \#* ]] && continue
    [[ "${hash}" =~ ^[0-9a-f]{64}$ ]] || { echo "ERROR: malformed hash entry" >&2; exit 1; }
done < "${SCRIPT_DIR}/pdfium-checksums.txt"

echo "verify-sha256 self-test passed"
