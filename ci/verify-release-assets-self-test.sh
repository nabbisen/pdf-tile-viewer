#!/usr/bin/env bash
# Self-test for ci/verify-release-assets.sh (RFC 036 § 5.4a).

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=ci/self-test-fixture.sh
source "${SCRIPT_DIR}/self-test-fixture.sh"

TMP_PARENT="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
WORK="$(mktemp -d "${TMP_PARENT%/}/verify-release-assets-self-test.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
V=2.0.2
gate() { bash "${SCRIPT_DIR}/verify-release-assets.sh" "$@"; }

good_set() {
    local d="$1"; rm -rf "${d:?}"; mkdir -p "$d"
    for n in "pdf-tile-viewer-v${V}-linux-x64.tar.gz" "pdf-tile-viewer-v${V}-macos-arm64.tar.gz" \
             "pdf-tile-viewer-v${V}-macos-x64.tar.gz" "pdf-tile-viewer-v${V}-windows-x64.zip" \
             "pdf-tile-viewer-v${V}.tar.gz"; do echo data > "$d/$n"; done
}

d="${WORK}/ok"; good_set "$d";                       run_expect_ok   "exactly the five expected assets" gate "$d" "${V}"

d="${WORK}/stray"; good_set "$d"; echo old > "$d/pdf-tile-viewer-v2.0.0-linux-x64.tar.gz"
run_expect_fail "stray old-version archive (the 2.0.1 incident)" "UNEXPECTED pdf-tile-viewer-v2.0.0-linux-x64.tar.gz" gate "$d" "${V}"

d="${WORK}/extra"; good_set "$d"; echo x > "$d/notes.txt"
run_expect_fail "unrelated extra file" "UNEXPECTED notes.txt" gate "$d" "${V}"

d="${WORK}/missing"; good_set "$d"; rm "$d/pdf-tile-viewer-v${V}-windows-x64.zip"
run_expect_fail "missing platform archive" "MISSING pdf-tile-viewer-v${V}-windows-x64.zip" gate "$d" "${V}"

d="${WORK}/nosource"; good_set "$d"; rm "$d/pdf-tile-viewer-v${V}.tar.gz"
run_expect_fail "missing source archive" "MISSING pdf-tile-viewer-v${V}.tar.gz" gate "$d" "${V}"

d="${WORK}/wrongver"; good_set "$d"
run_expect_fail "wrong version (all five files belong to 2.0.2, gate asked for 2.0.3)" "MISSING" gate "$d" "2.0.3"

d="${WORK}/subdir"; good_set "$d"; mkdir "$d/nested"
run_expect_fail "sub-directory" "UNEXPECTED nested" gate "$d" "${V}"

run_expect_fail "not a directory" "not a directory" gate "${WORK}/nope" "${V}"

echo "verify-release-assets self-test passed"
