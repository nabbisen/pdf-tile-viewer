#!/usr/bin/env bash
# The only tool that changes the project version (RFC 036 § 5.7).
#
# Usage: ci/set-version.sh <X.Y.Z[-pre]>
#
#   * refuses to run on a dirty git tree;
#   * validates the version (semver, no leading zeros);
#   * requires CHANGELOG.md to contain the heading "## [<version>] - unreleased"
#     (release notes are written by people) and replaces "unreleased" with
#     today's UTC date;
#   * sets [workspace.package] version and the four [workspace.dependencies]
#     path versions in Cargo.toml, refreshes Cargo.lock (workspace crates only);
#   * sets the MSIX manifest to "<core>.0" (a pre-release suffix is dropped);
#   * finishes with ci/verify-version-consistency.sh.
# It does not commit, tag or push.

set -euo pipefail

VERSION="${1:?usage: set-version.sh <X.Y.Z[-pre]>}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CARGO_TOML="${ROOT}/Cargo.toml"
MANIFEST="${ROOT}/packaging/windows/AppxManifest.xml"
CHANGELOG="${ROOT}/CHANGELOG.md"

fail() { echo "set-version: $*" >&2; exit 1; }

SEMVER_RE='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$'
[[ "${VERSION}" =~ ${SEMVER_RE} ]] || fail "'${VERSION}' is not a valid version (expected X.Y.Z or X.Y.Z-pre)"
CORE="${VERSION%%-*}"

cd "${ROOT}"
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || fail "not a git work tree"
[[ -z "$(git status --porcelain)" ]] || fail "working tree is not clean; commit or stash first"

HEADING_UNRELEASED="## [${VERSION}] - unreleased"
grep -q -x -F -- "${HEADING_UNRELEASED}" "${CHANGELOG}" \
    || fail "CHANGELOG.md must contain the heading '${HEADING_UNRELEASED}' (write the release notes first)"
[[ "$(grep -c -x -F -- "${HEADING_UNRELEASED}" "${CHANGELOG}")" == "1" ]] \
    || fail "CHANGELOG.md contains the heading '${HEADING_UNRELEASED}' more than once"

[[ -f "${MANIFEST}" ]] || fail "manifest not found: ${MANIFEST}"
[[ "$(grep -c -E '[[:space:]]Version="' "${MANIFEST}")" == "1" ]] \
    || fail "expected exactly one Identity Version attribute in the manifest"

TODAY="$(date -u +%F)"
TMP="$(mktemp "${TMPDIR:-/tmp}/set-version.XXXXXX")"
trap 'rm -f "${TMP}"' EXIT

# --- Cargo.toml: workspace version (first match inside [workspace.package]) and path deps
awk -v v="${VERSION}" '
    /^\[/ { in_pkg = ($0 == "[workspace.package]"); in_deps = ($0 == "[workspace.dependencies]") }
    in_pkg && !done && /^version[[:space:]]*=/ { sub(/"[^"]*"/, "\"" v "\""); done = 1 }
    in_deps && ($1 == "domain" || $1 == "pdf_engine" || $1 == "app_services" || $1 == "packaging") {
        sub(/version[[:space:]]*=[[:space:]]*"[^"]*"/, "version = \"" v "\"")
    }
    { print }
' "${CARGO_TOML}" > "${TMP}"
cat "${TMP}" > "${CARGO_TOML}"

# --- Cargo.lock: only the workspace crates
cargo update --workspace --quiet

# --- MSIX manifest: <core>.0
sed -E "s/([[:space:]]Version=\")[^\"]*(\")/\1${CORE}.0\2/" "${MANIFEST}" > "${TMP}"
cat "${TMP}" > "${MANIFEST}"

# --- CHANGELOG: date the heading
sed "s/^## \[${VERSION//./\\.}\] - unreleased\$/## [${VERSION}] - ${TODAY}/" "${CHANGELOG}" > "${TMP}"
cat "${TMP}" > "${CHANGELOG}"

bash "${SCRIPT_DIR}/verify-version-consistency.sh"

echo "set-version: ${VERSION} (MSIX ${CORE}.0, CHANGELOG dated ${TODAY})"
git status --short
