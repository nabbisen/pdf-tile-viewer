#!/usr/bin/env bash
# Self-test for ci/verify-version-consistency.sh (RFC 036 § 5.7): runs the script
# against scratch copies of a fixture repository.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=ci/self-test-fixture.sh
source "${SCRIPT_DIR}/self-test-fixture.sh"

TMP_PARENT="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
WORK="$(mktemp -d "${TMP_PARENT%/}/verify-version-consistency-self-test.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
make_fixture "${WORK}/base"

# fresh_copy <name>: independent copy so that each case starts from the baseline
fresh_copy() { rm -rf "${WORK:?}/${1:?}"; cp -R "${WORK}/base" "${WORK}/$1"; echo "${WORK}/$1"; }
check() { bash "$1/ci/verify-version-consistency.sh"; }

d="$(fresh_copy ok)";            run_expect_ok   "consistent baseline" check "$d"

d="$(fresh_copy unreleased)"
sed -i.bak 's/^## \[1\.2\.3\] - 2026-01-01$/## [1.2.3] - unreleased/' "$d/CHANGELOG.md"
run_expect_ok   "unreleased heading is accepted" check "$d"

d="$(fresh_copy manifest-mismatch)"
sed -i.bak 's/Version="1\.2\.3\.0"/Version="1.2.4.0"/' "$d/packaging/windows/AppxManifest.xml"
run_expect_fail "manifest/Cargo mismatch" "does not match Cargo version core" check "$d"

d="$(fresh_copy nonzero-revision)"
sed -i.bak 's/Version="1\.2\.3\.0"/Version="1.2.3.1"/' "$d/packaging/windows/AppxManifest.xml"
run_expect_fail "non-zero revision" "revision must be 0" check "$d"

d="$(fresh_copy placeholder)"
sed -i.bak 's/Version="1\.2\.3\.0"/Version="@VERSION@"/' "$d/packaging/windows/AppxManifest.xml"
run_expect_fail "placeholder" "placeholder" check "$d"

d="$(fresh_copy prerelease-cargo)"
sed -i.bak -e 's/^version     = "1\.2\.3"/version     = "1.2.3-rc.1"/' "$d/Cargo.toml"
run_expect_fail "dependency versions differ from the Cargo version" "workspace dependency" check "$d"

d="$(fresh_copy dep-mismatch)"
sed -i.bak 's/^packaging                   = { version = "1\.2\.3"/packaging                   = { version = "1.2.2"/' "$d/Cargo.toml"
run_expect_fail "path dependency mismatch" "workspace dependency 'packaging'" check "$d"

d="$(fresh_copy heading-missing)"
sed -i.bak '/^## \[1\.2\.3\]/d' "$d/CHANGELOG.md"
run_expect_fail "CHANGELOG heading missing" "has no heading" check "$d"

d="$(fresh_copy heading-undated)"
sed -i.bak 's/^## \[1\.2\.3\] - 2026-01-01$/## [1.2.3]/' "$d/CHANGELOG.md"
run_expect_fail "CHANGELOG heading without date" "has no heading" check "$d"

d="$(fresh_copy invalid-semver)"
sed -i.bak 's/^version     = "1\.2\.3"/version     = "01.2"/' "$d/Cargo.toml"
run_expect_fail "invalid Cargo version" "not valid semver" check "$d"

echo "verify-version-consistency self-test passed"
