#!/usr/bin/env bash
# Print the body of the dated CHANGELOG section for a version (release notes).
#
# Usage: ci/extract-changelog.sh <version> [changelog-file]
#
# Prints everything between the heading "## [<version>] - YYYY-MM-DD" and the next
# "## " heading (or end of file), without the heading itself, without the "---"
# rule that separates sections, and without leading/trailing blank lines.
# Fails (non-zero) when the section is missing, is not dated ("unreleased" means
# the release was not prepared) or has an empty body.

set -euo pipefail

VERSION="${1:?usage: extract-changelog.sh <version> [changelog-file]}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHANGELOG="${2:-${SCRIPT_DIR}/../CHANGELOG.md}"

fail() { echo "extract-changelog: $*" >&2; exit 1; }

[[ -f "${CHANGELOG}" ]] || fail "file not found: ${CHANGELOG}"

HEADING_RE="^## \[${VERSION//./\\.}\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$"
grep -q -E "${HEADING_RE}" "${CHANGELOG}" \
    || fail "no dated heading '## [${VERSION}] - YYYY-MM-DD' in ${CHANGELOG}"

BODY="$(HEADING_RE="${HEADING_RE}" awk '
    BEGIN { re = ENVIRON["HEADING_RE"] }
    $0 ~ re { in_section = 1; next }
    in_section && /^## / { exit }
    in_section { lines[++n] = $0 }
    END {
        # drop the section separator and surrounding blank lines
        while (n > 0 && (lines[n] ~ /^[[:space:]]*$/ || lines[n] == "---")) n--
        start = 1
        while (start <= n && lines[start] ~ /^[[:space:]]*$/) start++
        for (i = start; i <= n; i++) print lines[i]
    }
' "${CHANGELOG}")"

[[ -n "${BODY}" ]] || fail "the section for ${VERSION} is empty"
printf '%s\n' "${BODY}"
