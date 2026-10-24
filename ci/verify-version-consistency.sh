#!/usr/bin/env bash
# Check that the version files agree (RFC 036 § 5.7). Read-only.
#
# Usage: ci/verify-version-consistency.sh
#
# Checks:
#   1. [workspace.package] version in Cargo.toml is valid semver.
#   2. The four [workspace.dependencies] path crates carry the same version.
#   3. packaging/windows/AppxManifest.xml has a real Identity Version "X.Y.Z.R"
#      (no @…@ placeholder), where X.Y.Z equals the Cargo version core and R is 0
#      (the Microsoft Store reserves the fourth segment).
#   4. CHANGELOG.md has a heading "## [<Cargo version>]" followed by
#      " - YYYY-MM-DD" or " - unreleased".
# The repository root is the parent of this script's directory (self-tests run a
# copy of the script inside a scratch repository).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CARGO_TOML="${ROOT}/Cargo.toml"
MANIFEST="${ROOT}/packaging/windows/AppxManifest.xml"
CHANGELOG="${ROOT}/CHANGELOG.md"

fail() { echo "verify-version-consistency: $*" >&2; exit 1; }

SEMVER_RE='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$'

# --- 1. Cargo version: the first "version = ..." line inside [workspace.package]
CARGO_VERSION="$(awk '
    /^\[/ { in_pkg = ($0 == "[workspace.package]") }
    in_pkg && /^version[[:space:]]*=/ { gsub(/^[^"]*"|".*$/, ""); print; exit }
' "${CARGO_TOML}")"
[[ -n "${CARGO_VERSION}" ]] || fail "no [workspace.package] version in Cargo.toml"
[[ "${CARGO_VERSION}" =~ ${SEMVER_RE} ]] || fail "Cargo version '${CARGO_VERSION}' is not valid semver"
CORE="${CARGO_VERSION%%-*}"

# --- 2. Path dependency versions
for crate in domain pdf_engine app_services packaging; do
    dep_version="$(awk -v c="${crate}" '
        /^\[/ { in_deps = ($0 == "[workspace.dependencies]") }
        in_deps && $1 == c { if (match($0, /version[[:space:]]*=[[:space:]]*"[^"]*"/)) { v = substr($0, RSTART, RLENGTH); gsub(/^[^"]*"|"$/, "", v); print v; exit } }
    ' "${CARGO_TOML}")"
    [[ -n "${dep_version}" ]] || fail "no version for workspace dependency '${crate}'"
    [[ "${dep_version}" == "${CARGO_VERSION}" ]] \
        || fail "workspace dependency '${crate}' is ${dep_version}, Cargo version is ${CARGO_VERSION}"
done

# --- 3. MSIX manifest
[[ -f "${MANIFEST}" ]] || fail "manifest not found: ${MANIFEST}"
if grep -q -E '@[A-Za-z_]+@' "${MANIFEST}"; then
    fail "manifest contains a placeholder (@…@)"
fi
MANIFEST_VERSION="$(grep -o -E '[[:space:]]Version="[^"]*"' "${MANIFEST}" | sed -E 's/.*"([^"]*)"/\1/' || true)"
[[ "$(printf '%s\n' "${MANIFEST_VERSION}" | grep -c .)" == "1" ]] \
    || fail "expected exactly one Identity Version in the manifest, found: '${MANIFEST_VERSION}'"
[[ "${MANIFEST_VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]] \
    || fail "manifest version '${MANIFEST_VERSION}' is not X.Y.Z.R"
[[ "${MANIFEST_VERSION##*.}" == "0" ]] \
    || fail "manifest revision must be 0 (Microsoft Store reserves it), got '${MANIFEST_VERSION}'"
[[ "${MANIFEST_VERSION%.*}" == "${CORE}" ]] \
    || fail "manifest version ${MANIFEST_VERSION} does not match Cargo version core ${CORE}"

# --- 4. CHANGELOG heading
HEADING_RE="^## \[${CARGO_VERSION//./\\.}\] - ([0-9]{4}-[0-9]{2}-[0-9]{2}|unreleased)$"
grep -q -E "${HEADING_RE}" "${CHANGELOG}" \
    || fail "CHANGELOG.md has no heading '## [${CARGO_VERSION}] - YYYY-MM-DD' or '## [${CARGO_VERSION}] - unreleased'"

echo "verify-version-consistency: OK (Cargo ${CARGO_VERSION}, manifest ${MANIFEST_VERSION}, CHANGELOG heading present)"
