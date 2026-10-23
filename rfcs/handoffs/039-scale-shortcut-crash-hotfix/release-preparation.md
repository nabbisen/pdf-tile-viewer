# Handoff — Release preparation for 2.0.1 (hotfix: RFC 039 only)

Governing RFC: [`../../accepted/039-scale-shortcut-crash-hotfix.md`](../../accepted/039-scale-shortcut-crash-hotfix.md). Release scope: **RFC 039 only** — no other change.
Authorization (owner, 2026-10-24): the dev team may prepare the release; **the actual release and publication are not part of this task** and are performed only by the owner (or by the architect when the owner explicitly hands it over).

Working model: directly on `main`, no branches, you are the only actor on the tree; start from a clean `git status` at `main` = `origin/main`; hand over with the commit hash; keep `main` green.

## Hard limits (DEC-001)
Do **not** create a tag, push a tag, create a GitHub Release, run or trigger a release or MSIX workflow, or upload anything to the Microsoft Store. No other content change besides the four items below.

## Facts established by the architect (2026-10-24)
- Origin has no `2.0.1` tag and no GitHub release `2.0.1`; the local-only tag `2.0.1` points at `5c69522` (its `Cargo.toml` says 2.0.0). `Cargo.toml` says 2.0.0 at every commit, so no 2.0.1 MSIX can have been built by our pipeline.
- CI on `main` is green at `46fa0fb`.
- The release workflow does not yet verify that the tag equals the workspace version (RFC 036 is later). **The tag must therefore be created only on a commit whose `Cargo.toml` version is exactly `2.0.1`.**

## Tasks
1. **Stray tag.** Record `git rev-parse 2.0.1` (expected `5c69522…`) in the review request, then `git tag -d 2.0.1` (local only; owner decision D7). Confirm with `git tag -l '2.0.*'` and `git ls-remote --tags origin '2.0.*'` that only `2.0.0` (and beta tags) remain.
2. **Version.** Workspace version `2.0.1` in `Cargo.toml` (`[workspace.package]` and the four `[workspace.dependencies]` path entries `version = "2.0.1"`); update `Cargo.lock` with `cargo check --workspace` (only the five workspace crates change; verify with `git diff --stat Cargo.lock`).
3. **CHANGELOG.** Add `## [2.0.1] - 2026-10-28` (the planned date; the releaser corrects it in one line if the tag is cut on another day) above `[2.0.0]`, and keep an empty `## [Unreleased]` above it. Entries: **Fixed** — the application no longer aborts when the scale shortcuts `+`, `=`, `-` or `Ctrl`+scroll are used in the viewer (RFC 039). **Added (documentation)** — release QA checklist, signal borrow rule, "Lessons for Dioxus Desktop Apps" chapter. Nothing else; do not mention unreleased RFC work.
4. **Version statements in user docs.** Find statements naming the release line (`grep -rn "2\.0\.0" README*.md docs/src --include='*.md'`, excluding tag/PDFium/file-name examples and historical CHANGELOG/RFC text) and update only those that describe the *current* release (for example "documentation covers the 2.0.0 release line", "latest release artifacts are available for 2.0.0", the unsigned-build note). List every changed and every deliberately unchanged hit in the review request.
5. **Pre-flight evidence** on the final commit (not a release): `cargo fmt --check`; `cargo check --workspace` (0 warnings); `cargo test --workspace`; PDFium smoke with `PDF_TILE_VIEWER_PDFIUM_DIR="$PWD/ci/.pdfium"`; `bash ci/package-linux.sh` produces `dist/pdf-tile-viewer-v2.0.1-linux-x64.tar.gz` (the script runs the packaged-artifact smoke; `dist/` is untracked content — do not `git add` it) and `bash ci/smoke-release-artifact-self-test.sh`; `bash ci/archive-source.sh` into a scratch directory shows the source archive name `pdf-tile-viewer-v2.0.1.tar.gz`. Record sizes and SHA-256 of both archives. Delete the scratch outputs afterwards. Push the commit and confirm CI is green on it (`gh run list --branch main --limit 3`).

## Deliverable
`.git-exclude/review-request/039-release-preparation/README.md` (single entry point): commits, the recorded tag commit, version/CHANGELOG diffs, the user-doc hit list, command results, archive sizes and hashes, CI result, unresolved issues. State explicitly that **no tag was created or pushed**.

## After this task (not yours)
The architect issues a release-readiness report; the owner decides (G7-H). Tagging `2.0.1` on that exact commit triggers `release.yml` (four platform archives plus source archive, published as a GitHub Release) and then `msix-store.yml` (MSIX artifact; Store upload remains manual).
