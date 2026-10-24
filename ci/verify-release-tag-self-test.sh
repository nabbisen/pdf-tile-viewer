#!/usr/bin/env bash
# Self-test for ci/verify-release-tag.sh (RFC 036 § 5.2): temporary git
# repositories; "main" stands in for origin/main (RELEASE_MAIN_REF).

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=ci/self-test-fixture.sh
source "${SCRIPT_DIR}/self-test-fixture.sh"

TMP_PARENT="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
WORK="$(mktemp -d "${TMP_PARENT%/}/verify-release-tag-self-test.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
export RELEASE_MAIN_REF=main

fresh_copy() { rm -rf "${WORK:?}/${1:?}"; cp -R "${WORK}/base" "${WORK}/$1"; echo "${WORK}/$1"; }
verify() { local d="$1"; shift; (cd "$d" && bash ci/verify-release-tag.sh "$@"); }
make_fixture "${WORK}/base"

# fixture version is 1.2.3 with a dated CHANGELOG heading, committed on main
d="$(fresh_copy match)"; (cd "$d" && git tag -a 1.2.3 -m 1.2.3)
run_expect_ok   "matching tag on main" verify "$d" 1.2.3

d="$(fresh_copy wrong-tag)"; (cd "$d" && git tag -a 1.2.4 -m 1.2.4)
run_expect_fail "tag differs from the workspace version" "does not equal the workspace version" verify "$d" 1.2.4

d="$(fresh_copy stray-tag-old-commit)"   # the 2.0.1-on-2.0.0 incident: tag name ahead of the code
(cd "$d" && git tag -a 1.3.0 -m stray)
run_expect_fail "stray tag on a commit with an older version" "does not equal the workspace version" verify "$d" 1.3.0

d="$(fresh_copy no-heading)"
(cd "$d" && sed -i.bak '/^## \[1\.2\.3\]/d' CHANGELOG.md && rm -f CHANGELOG.md.bak && git commit -q -am "drop heading" && git tag -a 1.2.3 -m 1.2.3)
run_expect_fail "CHANGELOG heading missing" "no dated heading" verify "$d" 1.2.3

d="$(fresh_copy unreleased)"
(cd "$d" && sed -i.bak 's/^## \[1\.2\.3\] - 2026-01-01$/## [1.2.3] - unreleased/' CHANGELOG.md && rm -f CHANGELOG.md.bak && git commit -q -am "unreleased" && git tag -a 1.2.3 -m 1.2.3)
run_expect_fail "CHANGELOG heading not dated (release not prepared)" "no dated heading" verify "$d" 1.2.3

d="$(fresh_copy off-main)"
(cd "$d" && git checkout -q -b side && echo x > side.txt && git add side.txt && git commit -q -m side && git tag -a 1.2.3 -m 1.2.3 && git checkout -q main)
run_expect_fail "commit not on main" "not an ancestor" verify "$d" 1.2.3

d="$(fresh_copy missing-tag)"
run_expect_fail "tag not found" "not found" verify "$d" 1.2.3

d="$(fresh_copy missing-main)"; (cd "$d" && git tag -a 1.2.3 -m 1.2.3)
run_expect_fail "main reference missing" "not found" bash -c "cd '$d' && RELEASE_MAIN_REF=origin/main bash ci/verify-release-tag.sh 1.2.3"

echo "verify-release-tag self-test passed"
