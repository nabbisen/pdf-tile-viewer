# Handoff — RFC 036 follow-up: dependency advisories, Dependabot rules, CHANGELOG-based release notes

Governing RFC: [`../../accepted/036-release-supply-chain-and-workflow-hardening.md`](../../accepted/036-release-supply-chain-and-workflow-hardening.md) (§ 5.4, § 5.5; review `.git-exclude/reviewed/rfc036-supply-chain-review-2026-10-24.md`). Executes the review's follow-ups; must not redefine the RFC.
Order: immediately after the RFC 036 slice review, before RFC 038 B′. Working model: directly on `main`, one actor, hand over with the commit hash, `main` green. No tags, no releases, no *Prepare release* dispatch (DEC-001). **Scratch work only in a throw-away clone (`git clone`), never in a git worktree: tags are shared between worktrees and `git tag -d` there deletes the real local tag (incident 2026-10-24).**

## Tasks
1. **Advisories (lockfile only).** `cargo update -p crossbeam-epoch -p webbrowser`; check whether `quick-xml` can reach ≥ 0.41 without a manifest change (`cargo update -p quick-xml` / `cargo tree -i quick-xml`); if it needs a new `wayland-scanner`/`rfd` release, record why and leave the advisory open with a dated note in `deny.toml`. **No `Cargo.toml` change, no major or minor bump of a direct dependency.** Also take the lockfile-only patch/minor updates Dependabot proposed for `serde`, `serde_json`, `futures-executor` **only if** the gate stays green. Do not touch `pdfium-render`.
2. **Evidence for the changed binary** (the shipped binary changes with the lockfile): full gate (`fmt`, `check` 0 warnings, `clippy -D warnings`, `cargo test --workspace`, PDFium smoke); a **release build** staged in the package layout (scratch config, transient unit, focus check before synthetic input): open a PDF, scroll, search, open the zoom view, `+`/`-` keys; stderr without `panicked`. List every crate whose version changed (`git diff Cargo.lock` summary).
3. **cargo-deny.** Re-run; if only the documented ignores remain, remove `continue-on-error` from the CI job (blocking), and verify that a deliberately added vulnerable dependency in a scratch **clone** fails it. If anything stays open, keep it non-blocking and report which.
4. **Dependabot.** `.github/dependabot.yml`: `ignore` entries — `pdfium-render` (all updates; comment: bumped only with the PDFium pin procedure, DEC-012) and semver-major updates for the other Cargo dependencies and for `actions/*` majors are **not** ignored globally; instead name `dirs`, `base64` (their current proposals) with a comment, so useful updates keep coming. Document in `dev.md`: how Dependabot PRs are handled between slices (owner merges only lockfile-only PRs that pass CI; never `pdfium-render`).
5. **Release notes from the CHANGELOG.** New `ci/extract-changelog.sh <version>` printing the body of the dated `## [<version>]` section (self-test: found, missing, last section, section with sub-headings); `release.yml` `release` job writes it plus a fixed footer (unsigned builds; keep `bin/` and `resources/` together; link to the full changelog) to a file and passes it as `body_path` to the release action; remove `generate_release_notes`. Pre-release tags use the same mechanism. Update `dev.md`.
6. Docs: `dev.md` / `testing.md` as needed; `CHANGELOG.md` `[Unreleased]` → "Fixed: dependency advisories (…)" listing the crates.

## Non-change scope
Application code, versions (CI sets them), other workflows' structure, any dependency requirement in a manifest.

## Evidence and review request
Standard fields plus: the before/after `cargo deny` report, the list of changed crates, the release-build check, self-test output. `.git-exclude/review-request/036-followup-deps/README.md` (single entry point).

## Prohibited shortcuts
`cargo update` without `-p` (broad lockfile churn); bumping manifests; ignoring an advisory without a written reason in `deny.toml`; testing in a worktree; any tag operation.
