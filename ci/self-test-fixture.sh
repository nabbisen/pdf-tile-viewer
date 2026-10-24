#!/usr/bin/env bash
# Shared helper for the release-tooling self-tests (sourced, not executed).
#
# make_fixture <dir>: creates a throw-away git repository with a minimal cargo
# workspace of the same shape as the real one (five path crates, no external
# dependencies, so `cargo update --workspace` works offline), a CHANGELOG, an
# MSIX manifest, and copies of the ci scripts under test. Version 1.2.3.

make_fixture() {
    local dir="$1" script_src crate
    script_src="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    mkdir -p "${dir}/ci" "${dir}/packaging/windows"
    cat > "${dir}/Cargo.toml" <<'TOML'
[workspace]
resolver = "3"
members = ["crates/app", "crates/domain", "crates/pdf_engine", "crates/app_services", "crates/packaging"]

[workspace.package]
version     = "1.2.3"
edition     = "2024"

[workspace.dependencies]
domain                      = { version = "1.2.3", path = "crates/domain" }
pdf_engine                  = { version = "1.2.3", path = "crates/pdf_engine" }
app_services                = { version = "1.2.3", path = "crates/app_services" }
packaging                   = { version = "1.2.3", path = "crates/packaging" }
TOML
    for crate in app domain pdf_engine app_services packaging; do
        mkdir -p "${dir}/crates/${crate}/src"
        printf '[package]\nname = "%s"\nversion.workspace = true\nedition.workspace = true\n' "${crate}" > "${dir}/crates/${crate}/Cargo.toml"
        : > "${dir}/crates/${crate}/src/lib.rs"
    done
    cat > "${dir}/packaging/windows/AppxManifest.xml" <<'XML'
<?xml version="1.0" encoding="utf-8"?>
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10">
  <Identity Name="example.app"
            Publisher="CN=Example"
            Version="1.2.3.0" />
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.19041.0" />
  </Dependencies>
</Package>
XML
    cat > "${dir}/CHANGELOG.md" <<'MD'
# Changelog

## [Unreleased]

## [1.3.0] - unreleased

### Added

- something

## [1.4.0-rc.1] - unreleased

## [1.2.3] - 2026-01-01

### Fixed

- earlier
MD
    cp "${script_src}"/set-version.sh "${script_src}"/verify-version-consistency.sh \
       "${script_src}"/verify-release-tag.sh "${dir}/ci/"
    (
        cd "${dir}" || exit 1
        cargo generate-lockfile --quiet --offline
        printf '/target\n' > .gitignore
        git init --quiet -b main
        git config user.email "self-test@example.invalid"
        git config user.name "self-test"
        git add -A
        git commit --quiet -m "fixture"
    )
}

# run_expect_fail <label> <expected stderr fragment> <command...>
run_expect_fail() {
    local label="$1" fragment="$2"; shift 2
    local err
    err="$(mktemp "${TMPDIR:-/tmp}/selftest-err.XXXXXX")"
    if "$@" >/dev/null 2>"${err}"; then
        echo "ERROR: expected '${label}' to fail" >&2
        rm -f "${err}"; exit 1
    fi
    if ! grep -q -F -- "${fragment}" "${err}"; then
        echo "ERROR: '${label}' failed for the wrong reason (wanted '${fragment}'):" >&2
        cat "${err}" >&2; rm -f "${err}"; exit 1
    fi
    rm -f "${err}"
    echo "expected failure passed: ${label}"
}

# run_expect_ok <label> <command...>
run_expect_ok() {
    local label="$1"; shift
    local out
    out="$(mktemp "${TMPDIR:-/tmp}/selftest-out.XXXXXX")"
    if ! "$@" >"${out}" 2>&1; then
        echo "ERROR: expected '${label}' to succeed" >&2
        cat "${out}" >&2; rm -f "${out}"; exit 1
    fi
    rm -f "${out}"
    echo "expected success passed: ${label}"
}
