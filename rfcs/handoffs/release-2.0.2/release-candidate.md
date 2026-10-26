# Handoff — Release candidate 2.0.2 (preparation; no release)

Context: the 2.0.2 train (RFC 033–037, 038, 040, 043 and the final sync) is on `main`. Release procedure: `docs/src/contributors/dev.md` § *Release procedure*; checklist: `docs/src/contributors/release-qa.md`; versioning policy: `docs/src/contributors/versioning.md`. Working model: directly on `main`, one actor at a time, `main` green. **Forbidden for the dev team: tags, pushing tags, running *Prepare release*, `release.yml`, `msix-store.yml`, any GitHub Release or Store action (DEC-001).** The owner decides each step below marked *owner*.

## A. Dev team — pre-flight (read-only on `main`; no commits unless a defect is found, then ask the architect first)
1. Pull `main`; confirm CI is green on its head (`gh run list --branch main --limit 3`).
2. Throw-away clone (not a worktree) of `main`: `cargo fmt --check`; `cargo check --workspace` (0 warnings); `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; PDFium smoke; `cargo deny check`; the seven `ci/*-self-test.sh` scripts; `mdbook build docs`; the relative-link check.
3. Local package build and packaged-artifact smoke for Linux as for 2.0.1 (`ci/package-linux.sh`, run the staged binary headless where possible); build the source archive; record file names, sizes and SHA-256. They will differ from the CI artifacts; this only proves the scripts work.
4. Run `bash ci/set-version.sh 2.0.2` **in the throw-away clone only** and report the diff (it must touch exactly: workspace `Cargo.toml`, `Cargo.lock` workspace crates, the MSIX manifest, the CHANGELOG heading date). Then `bash ci/verify-version-consistency.sh`. Do not copy any of it to `main`.
5. Re-read `CHANGELOG.md` `## [2.0.2] - unreleased` against `git log 2.0.1..main`: list any user-visible change missing from it, and any entry without a commit.

Report: `.git-exclude/review-request/release-2.0.2/README.md` (single entry point): commands and results, hashes, the `set-version.sh` diff, CHANGELOG discrepancies, anything surprising.

## B. Owner (not the dev team)
1. **Fork dry run** of the workflows (`dev.md` § *Trying workflow changes on a fork*; steps: `.git-exclude/review-request/036-supply-chain/README.md` § 9). Pre-release tag on the fork only, for example `2.0.2-rc.1`. Expect: `verify-tag` passes, four platform builds, source archive, asset gate, exactly five release files.
2. **Cross-platform manual QA** on Windows (WebView2) and macOS (WKWebView) with `release-qa.md` and the case lists in `.git-exclude/review-request/033-keyboard/README.md` § 7 and `.git-exclude/review-request/043-system-locale/README.md` § 5: especially `inert` backgrounds and Tab containment, Japanese UI on Japanese Windows, and the Windows memory check (long text PDF and 100-page image PDF, scroll to the end and back twice).
3. **Screen-reader listening session**: `rfcs/handoffs/037-user-facing-error-presentation-and-accessible-names/screen-reader-listening-session.md`.
4. After 1–3 and the architect's readiness report: go / no-go. On *go*: run **Prepare release** with `2.0.2` (or hand it over), the architect checks the printed commit, then the owner tags that commit and publishes (`dev.md` steps 2–5). The Store listing Japanese-UI claim stays on hold until B2's Japanese-Windows case passes.

## C. Architect
Readiness report (`.git-exclude/reviewed/release-readiness-2.0.2-….md`) after A is reviewed and B1–B3 results are in.
