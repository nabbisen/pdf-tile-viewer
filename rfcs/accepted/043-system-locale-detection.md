---
project: PDF Tile Viewer
document_family: 2.0 corrective RFCs
language: English
date: 2026-10-24
status: Implemented in the 2.0.2 train — awaiting release (the architect moves it to done/ after the release)
accepted: 2026-10-24 (owner approved the design and the new dependency; the owner may amend the text)
baseline: PDF Tile Viewer 2.0.1
priority: Corrective
depends_on: RFC 017
amends: RFC 017 (§ 7 locale resolution: makes "detect system locale" true on every platform)
origin: Store-listing preparation 2026-10-24 (finding F-LOCALE)
---

# RFC 043 — System Locale Detection

## 1. Summary

When the user has not forced a language (`ui.locale` is `null`), choose the
interface language from the **operating system's language** on every platform, not
only from the `LANG` environment variable. A Japanese Windows or macOS user then
gets the Japanese interface without editing `settings.json`.

## 2. Motivation (verified in code)

`Locale::from_system()` reads only `std::env::var("LANG")`. Windows does not
normally set `LANG`, and macOS GUI applications usually do not either, so on those
systems the UI is **always English** unless `ui.locale` is hand-edited in a JSON
file that has no UI. RFC 017 § 7 specifies "detect system locale"; `settings.md`
describes the default as `null (system)` but then documents `LANG`. A user whose
whole system is Japanese is told, by the product, nothing, and shown English:
a confusing result of the kind the project's UI principles forbid.

## 3. Goals / Non-goals

Goals: the documented default ("system") works on Windows, macOS and Linux;
Linux behavior that is documented today does not change; the decision is a pure,
tested function. Non-goals: an in-app language switch (settings UI is RFC 031's
area), new languages, right-to-left support, per-document language.

## 4. Design

```rust
/// `lang_env`: value of the `LANG` environment variable, if any.
/// `os_locales`: the OS's preferred UI languages, most preferred first (BCP 47 or POSIX-like tags).
pub fn choose_locale(lang_env: Option<&str>, os_locales: &[String]) -> Locale
```

1. **Compatibility first.** If `LANG` is set and is not empty, `C` or `POSIX`
   (case-insensitive, with or without a `.UTF-8`-style suffix), decide from it
   exactly as today (`ja…` → Japanese, anything else → English). Documented Linux
   behavior is unchanged.
2. Otherwise walk `os_locales` in order and return the first whose language subtag
   is a **supported** language (`ja`, `en`); a list that contains only unsupported
   languages (for example `fr`) falls through to English.
3. Empty list → English (the reference catalog).
4. The explicit setting `ui.locale = "en"|"ja"` still wins over everything.

The thin impure part, `Locale::from_system()`, calls `std::env::var("LANG")` and
`sys_locale::get_locales()` and passes both to `choose_locale`. All logic lives in
the pure function.

**Dependency (owner-approved).** `sys-locale` 0.3.2, `MIT OR Apache-2.0`, small
and maintained by 1Password; it wraps the platform calls (Windows
`GetUserPreferredUILanguages`/`GetUserDefaultLocaleName`, macOS preferred
languages, POSIX variables) so that this project keeps its policy of a single
`unsafe` (DEC-009). Added to the workspace dependencies and the `app` crate only;
`NOTICE` lists it; the `cargo-deny` configuration allows its licenses.

## 5. Compatibility and versioning

Fix against RFC 017 and the documented default `null (system)`. `LANG`-first keeps
every documented Linux behavior. Windows and macOS users with no `LANG` change from
English to their system language when it is Japanese; others stay English. This is
a bug fix (patch release); the CHANGELOG lists it under "Fixed".

## 6. Tests and acceptance

- Unit tests for `choose_locale`: `LANG` precedence (`ja_JP.UTF-8`, `en_US`, `C`,
  `POSIX`, empty), OS list with `ja-JP`, `ja`, `en-US`, `fr` only, mixed order
  (`fr`, `ja`), empty list, explicit settings still win (existing resolve tests).
- Linux manual check on a release build with a scratch config: `LANG` unset and
  `LC_ALL=ja_JP.UTF-8` → Japanese; `LANG=en_US.UTF-8` → English; `ui.locale="ja"` →
  Japanese regardless.
- **Owner-verified on a Japanese Windows installation** (no developer can test it
  here): the dashboard and the window title are Japanese on first launch without
  any settings file. Until then the Store text keeps the Japanese-interface claim on hold.
- Documentation: `settings.md` (`ui.locale` row), FAQ entry, `i18n.md`; CHANGELOG.

## 7. Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| OS reports a language list in an unexpected format | Low | Normalize in the pure function; tests with POSIX-style and BCP 47 tags |
| A user relied on English-by-accident on a Japanese system | Low | Documented; `ui.locale = "en"` restores English |
| New dependency | Low | Small, permissive license, passes `cargo deny`; reviewed in the handoff |
