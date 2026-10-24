#!/usr/bin/env bash
# Release guard (RFC 036 § 5.2): refuse to release from a tag that does not match
# the code it points at. Used by the `verify-tag` job of release.yml.
#
# Usage: ci/verify-release-tag.sh <tag>
#
# Passes only if, for the commit the tag points to:
#   1. the tag equals [workspace.package] version in Cargo.toml (as committed),
#   2. CHANGELOG.md has a DATED heading "## [<version>] - YYYY-MM-DD"
#      ("unreleased" means the release was not prepared),
#   3. the commit is an ancestor of origin/main (override: RELEASE_MAIN_REF).
# Run from a checkout with full history (`fetch-depth: 0`).

set -euo pipefail

TAG="${1:?usage: verify-release-tag.sh <tag>}"
MAIN_REF="${RELEASE_MAIN_REF:-origin/main}"

fail() { echo "verify-release-tag: $*" >&2; exit 1; }

COMMIT="$(git rev-parse --verify --quiet "refs/tags/${TAG}^{commit}")" \
    || fail "tag '${TAG}' not found"

CARGO_VERSION="$(git show "${COMMIT}:Cargo.toml" | awk '
    /^\[/ { in_pkg = ($0 == "[workspace.package]") }
    in_pkg && /^version[[:space:]]*=/ { gsub(/^[^"]*"|".*$/, ""); print; exit }
')"
[[ -n "${CARGO_VERSION}" ]] || fail "cannot read [workspace.package] version at ${COMMIT}"

if [[ "${TAG}" != "${CARGO_VERSION}" ]]; then
    fail "tag '${TAG}' does not equal the workspace version '${CARGO_VERSION}' at ${COMMIT}"
fi

HEADING_RE="^## \[${CARGO_VERSION//./\\.}\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$"
git show "${COMMIT}:CHANGELOG.md" | grep -q -E "${HEADING_RE}" \
    || fail "CHANGELOG.md at ${COMMIT} has no dated heading '## [${CARGO_VERSION}] - YYYY-MM-DD' (prepare the release first)"

git rev-parse --verify --quiet "${MAIN_REF}" >/dev/null \
    || fail "reference '${MAIN_REF}' not found (checkout needs full history and the remote branch)"
git merge-base --is-ancestor "${COMMIT}" "${MAIN_REF}" \
    || fail "commit ${COMMIT} is not an ancestor of ${MAIN_REF}"

echo "verify-release-tag: OK tag ${TAG} -> ${COMMIT}"
