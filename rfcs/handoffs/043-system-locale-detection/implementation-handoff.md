# Handoff — RFC 043 System Locale Detection

Governing RFC: [`../../accepted/043-system-locale-detection.md`](../../accepted/043-system-locale-detection.md). Executes the RFC; must not redefine it.
Order: after RFC 037, before the final-sync handoff. Working model: directly on `main`, one actor, hand over with the commit hash, `main` green. No tags, no releases (DEC-001). GUI test protocol applies (scratch `XDG_CONFIG_HOME`, transient unit, focus check before synthetic input).

## Change scope
1. `Cargo.toml` `[workspace.dependencies]`: `sys-locale = "0.3"`; `crates/app/Cargo.toml`: use it. Update `Cargo.lock` (only that crate is added). Confirm with `cargo deny check` (RFC 036) that licenses and advisories pass; add `sys-locale`'s license expression to the allow list only if the config requires it. Add a `NOTICE` entry (name, license `MIT OR Apache-2.0`, repository `https://github.com/1Password/sys-locale`).
2. `crates/app/src/i18n.rs` (tests in `src/tests/i18n.rs` or the module's `tests.rs`): add the pure `choose_locale(lang_env, os_locales)` exactly per RFC 043 § 4; `Locale::from_system()` becomes the thin wrapper; `Locale::resolve` is unchanged otherwise. Normalization: lowercase; split on `-`, `_`, `.`, `@`; language = first part; `C`/`POSIX` and empty `LANG` are "unset".
3. Docs: `docs/src/intermediate/settings.md` (`ui.locale`: "`null` uses the `LANG` variable when it is set to a real locale, otherwise the operating system's language; unsupported languages use English"), `docs/src/contributors/i18n.md`, one FAQ entry ("Why is the interface in English / how do I change the language?" with the `ui.locale` setting), `CHANGELOG.md` `[Unreleased]` → Fixed.
4. `packaging/windows/store-listing.md`: do **not** edit (the architect lifts the hold after the owner's Windows verification).

## Non-change scope
Settings UI, new languages, message catalogs, any other module.

## Required tests and evidence
Unit tests per RFC 043 § 6 (all listed cases); gate green (`cargo fmt --check`, `cargo check --workspace` 0 warnings, `cargo clippy` blocking job passes, `cargo test --workspace`, smoke); Linux manual check on a **release build**: three launches (`LANG` unset with `LC_ALL=ja_JP.UTF-8`; `LANG=en_US.UTF-8`; `ui.locale="ja"` with `LANG=en_US.UTF-8`) with AT-SPI dumps or screenshots of the dashboard in each. List for the owner the Windows/macOS verification steps.

## Acceptance criteria
RFC 043 § 6, except the owner-run Windows check.

## Prohibited shortcuts
Reading the locale in several places; `unsafe` or direct Windows API calls; changing documented `LANG` behavior; adding a language switcher.

## Review request
`.git-exclude/review-request/043-system-locale/README.md` (single entry point), standard fields.
