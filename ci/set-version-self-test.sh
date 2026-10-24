#!/usr/bin/env bash
# Self-test for ci/set-version.sh (RFC 036 § 5.7): runs the script in scratch
# repositories (fixture = a minimal cargo workspace of the real shape).

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=ci/self-test-fixture.sh
source "${SCRIPT_DIR}/self-test-fixture.sh"

TMP_PARENT="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
WORK="$(mktemp -d "${TMP_PARENT%/}/set-version-self-test.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
make_fixture "${WORK}/base"
TODAY="$(date -u +%F)"

fresh_copy() { rm -rf "${WORK:?}/${1:?}"; cp -R "${WORK}/base" "${WORK}/$1"; echo "${WORK}/$1"; }
setv() { local d="$1"; shift; (cd "$d" && bash ci/set-version.sh "$@"); }
assert_contains() { grep -q -F -- "$2" "$1" || { echo "ERROR: $1 does not contain: $2" >&2; exit 1; }; }
assert_clean() { [[ -z "$(cd "$1" && git status --porcelain)" ]] || { echo "ERROR: a refused run changed files in $1" >&2; (cd "$1" && git status --short >&2); exit 1; }; }

# 1. clean run
d="$(fresh_copy clean)"
run_expect_ok "clean run (1.3.0)" setv "$d" 1.3.0
assert_contains "$d/Cargo.toml" 'version     = "1.3.0"'
for crate in domain pdf_engine app_services packaging; do
    assert_contains "$d/Cargo.toml" "${crate}"' '
    grep -E "^${crate} +=" "$d/Cargo.toml" | grep -q 'version = "1.3.0"' || { echo "ERROR: ${crate} path version not updated" >&2; exit 1; }
done
assert_contains "$d/packaging/windows/AppxManifest.xml" 'Version="1.3.0.0"'
assert_contains "$d/packaging/windows/AppxManifest.xml" 'MinVersion="10.0.17763.0"'
assert_contains "$d/CHANGELOG.md" "## [1.3.0] - ${TODAY}"
assert_contains "$d/CHANGELOG.md" "## [1.4.0-rc.1] - unreleased"
# Cargo.lock: only the five workspace crates, all at the new version
lock_versions="$(awk '/^name = /{n=$3} /^version = / && n != "" {print n, $3}' "$d/Cargo.lock" | LC_ALL=C sort)"
expected="$(printf '"app" "1.3.0"\n"app_services" "1.3.0"\n"domain" "1.3.0"\n"packaging" "1.3.0"\n"pdf_engine" "1.3.0"')"
[[ "${lock_versions}" == "${expected}" ]] || { echo "ERROR: unexpected Cargo.lock versions:" >&2; echo "${lock_versions}" >&2; exit 1; }
changed="$(cd "$d" && git status --porcelain | awk '{print $2}' | LC_ALL=C sort | tr '\n' ' ')"
[[ "${changed}" == "CHANGELOG.md Cargo.lock Cargo.toml packaging/windows/AppxManifest.xml " ]] \
    || { echo "ERROR: unexpected changed files: ${changed}" >&2; exit 1; }

# 2. pre-release suffix: kept in Cargo, dropped in the manifest only
d="$(fresh_copy prerelease)"
run_expect_ok "pre-release (1.4.0-rc.1)" setv "$d" 1.4.0-rc.1
assert_contains "$d/Cargo.toml" 'version     = "1.4.0-rc.1"'
assert_contains "$d/packaging/windows/AppxManifest.xml" 'Version="1.4.0.0"'
assert_contains "$d/CHANGELOG.md" "## [1.4.0-rc.1] - ${TODAY}"

# 3. dirty tree refused (and nothing touched)
d="$(fresh_copy dirty)"; echo "x" > "$d/stray.txt"
run_expect_fail "dirty tree" "not clean" setv "$d" 1.3.0
[[ "$(cd "$d" && git status --porcelain)" == "?? stray.txt" ]] || { echo "ERROR: dirty-tree refusal changed files" >&2; exit 1; }

# 4. invalid versions refused
d="$(fresh_copy invalid)"
for bad in 1.3 v1.3.0 01.3.0 1.3.0- 1.3.0-a..b "1.3.0;rm" "1.3.0 " ""; do
    run_expect_fail "invalid version '${bad}'" "version" setv "$d" "${bad}"
done
assert_clean "$d"

# 5. CHANGELOG heading missing / already dated refused
d="$(fresh_copy no-heading)"
run_expect_fail "CHANGELOG heading missing" "must contain the heading" setv "$d" 9.9.9
run_expect_fail "CHANGELOG heading already dated" "must contain the heading" setv "$d" 1.2.3
assert_clean "$d"

echo "set-version self-test passed"
