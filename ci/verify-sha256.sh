#!/usr/bin/env bash
# Verify a downloaded PDFium archive against ci/pdfium-checksums.txt (RFC 036 § 5.1).
#
# Usage: ci/verify-sha256.sh <file> <release_tag> <asset_name>
#   e.g. ci/verify-sha256.sh /tmp/pdfium.tgz chromium/7920 pdfium-linux-x64.tgz
#
# Fails closed (non-zero exit) when the file is missing, when there is no entry
# for (release_tag, asset_name), or when the SHA-256 does not match.
# The checksum file can be overridden with PDFIUM_CHECKSUMS_FILE (self-tests).

set -euo pipefail

FILE="${1:?usage: verify-sha256.sh <file> <release_tag> <asset_name>}"
TAG="${2:?missing release tag}"
ASSET="${3:?missing asset name}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHECKSUMS="${PDFIUM_CHECKSUMS_FILE:-${SCRIPT_DIR}/pdfium-checksums.txt}"

if [[ ! -f "${FILE}" ]]; then
    echo "verify-sha256: file not found: ${FILE}" >&2
    exit 1
fi
if [[ ! -f "${CHECKSUMS}" ]]; then
    echo "verify-sha256: checksum file not found: ${CHECKSUMS}" >&2
    exit 1
fi

# Exact match on the tag and asset columns (no regex, no substrings).
EXPECTED="$(awk -v tag="${TAG}" -v asset="${ASSET}" \
    '$0 !~ /^[[:space:]]*#/ && NF == 3 && $2 == tag && $3 == asset { print $1; exit }' "${CHECKSUMS}")"
if [[ -z "${EXPECTED}" ]]; then
    echo "verify-sha256: no pinned checksum for ${TAG} ${ASSET} in ${CHECKSUMS}" >&2
    exit 1
fi
if [[ ! "${EXPECTED}" =~ ^[0-9a-f]{64}$ ]]; then
    echo "verify-sha256: malformed checksum entry for ${TAG} ${ASSET}" >&2
    exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL="$(sha256sum "${FILE}" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
    ACTUAL="$(shasum -a 256 "${FILE}" | awk '{print $1}')"
else
    echo "verify-sha256: neither sha256sum nor shasum is available" >&2
    exit 1
fi

if [[ "${ACTUAL}" != "${EXPECTED}" ]]; then
    echo "verify-sha256: MISMATCH for ${ASSET} (${TAG})" >&2
    echo "  expected ${EXPECTED}" >&2
    echo "  actual   ${ACTUAL}" >&2
    exit 1
fi
echo "verify-sha256: OK ${ASSET} (${TAG}) ${ACTUAL}"
