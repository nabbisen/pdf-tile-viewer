#!/usr/bin/env bash
# Self-test for ci/extract-changelog.sh: found, missing, last section, section
# with sub-headings, undated heading, empty section.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=ci/self-test-fixture.sh
source "${SCRIPT_DIR}/self-test-fixture.sh"

TMP_PARENT="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
WORK="$(mktemp -d "${TMP_PARENT%/}/extract-changelog-self-test.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
CL="${WORK}/CHANGELOG.md"
cat > "${CL}" <<'MD'
# Changelog

Intro text.

---

## [Unreleased]

---

## [2.1.0] - 2026-11-02

### Fixed

- Fix one.
- Fix two
  continued on a second line.

### Added

- Added one.

---

## [2.0.1.5] - 2026-10-30

Short note.

---

## [2.0.2] - unreleased

- not released yet

---

## [2.0.0] - 2026-07-14

### Added

- First thing.

## [1.9.0] - 2026-01-01

## [1.8.0] - 2025-12-01

- Last section, no trailing separator.
MD

extract() { bash "${SCRIPT_DIR}/extract-changelog.sh" "$1" "${CL}"; }
assert_output() {  # assert_output <label> <version> <expected body>
    local label="$1" version="$2" expected="$3" actual
    actual="$(extract "${version}")" || { echo "ERROR: ${label}: extraction failed" >&2; exit 1; }
    if [[ "${actual}" != "${expected}" ]]; then
        echo "ERROR: ${label}: unexpected output:" >&2; printf '%s\n' "${actual}" >&2; exit 1
    fi
    echo "expected output passed: ${label}"
}

assert_output "section with sub-headings (stops at the --- separator, keeps ### headings)" 2.1.0 \
'### Fixed

- Fix one.
- Fix two
  continued on a second line.

### Added

- Added one.'
assert_output "dots in the version are literal (2.0.1.5 is not 2.0.1)" 2.0.1.5 'Short note.'
assert_output "section followed directly by another heading (no separator)" 2.0.0 \
'### Added

- First thing.'
assert_output "last section of the file" 1.8.0 '- Last section, no trailing separator.'

run_expect_fail "missing version" "no dated heading" extract 2.9.9
run_expect_fail "version that is only a prefix of another" "no dated heading" extract 2.0
run_expect_fail "undated (unreleased) section" "no dated heading" extract 2.0.2
run_expect_fail "empty section" "is empty" extract 1.9.0
run_expect_fail "missing file" "file not found" bash "${SCRIPT_DIR}/extract-changelog.sh" 2.1.0 "${WORK}/nope.md"

# The real CHANGELOG must yield the notes of its current version.
CARGO_VERSION="$(grep '^version' "${SCRIPT_DIR}/../Cargo.toml" | head -1 | sed 's/.*= *"//;s/".*//')"
if grep -q -E "^## \[${CARGO_VERSION//./\\.}\] - [0-9]{4}-" "${SCRIPT_DIR}/../CHANGELOG.md"; then
    bash "${SCRIPT_DIR}/extract-changelog.sh" "${CARGO_VERSION}" >/dev/null
    echo "expected success passed: real CHANGELOG section for ${CARGO_VERSION}"
fi

echo "extract-changelog self-test passed"
