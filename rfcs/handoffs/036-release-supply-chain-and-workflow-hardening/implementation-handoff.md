# Handoff — RFC 036 Release Supply-Chain and Workflow Hardening

Governing RFC: [`../../accepted/036-release-supply-chain-and-workflow-hardening.md`](../../accepted/036-release-supply-chain-and-workflow-hardening.md)
This handoff executes the RFC; it must not redefine it. Conflicts → design request to the architect.

Order: first slice of the 2.0.2 train (after RFC 038 Slice A is committed). Working model: directly on `main`, no branches, one actor at a time, hand over with the `main` commit hash.

**Hard limits (DEC-001):** do not create or push any tag, do not publish a release, do not change code-signing behavior, do not automate Store submission. Workflow changes are exercised by the **owner** on a fork; you prepare the instructions.

---

## Purpose
Make the pipeline verify what it ships and give each job only the privileges it needs (RFC 036 § 1–§ 5).

## Change scope

### 1. PDFium integrity (RFC 036 § 5.1)
- `ci/pdfium-checksums.txt`: format `sha256␣␣release_tag␣␣asset_name`, one line per asset for `chromium/7920`: `pdfium-linux-x64.tgz`, `pdfium-win-x64.tgz`, `pdfium-mac-arm64.tgz`, `pdfium-mac-x64.tgz` (add `pdfium-linux-arm64.tgz` only if `fetch-pdfium.sh` keeps supporting it).
- Obtain hashes by downloading each asset **twice** (separate invocations) and comparing; additionally compare with the GitHub release-asset `digest` when the API returns one (`gh api repos/bblanchon/pdfium-binaries/releases/tags/chromium/7920`). Record the procedure and results in the review request. If the two sources disagree, stop and report.
- `ci/verify-sha256.sh <file> <release_tag> <asset_name>`: looks up the expected hash in `ci/pdfium-checksums.txt`, computes the file hash (`sha256sum`, else `shasum -a 256`), exits non-zero on mismatch **or** missing entry. Use it in `ci/fetch-pdfium.sh`, `ci.yml`, `release.yml`, `msix-store.yml` immediately after each download and before extraction.
- `release.yml` stage step: append the bundled library's SHA-256 to the generated `notes.txt`.
- Self-test `ci/verify-sha256-self-test.sh` (match, mismatch, missing entry, missing file), in the style of `ci/smoke-release-artifact-self-test.sh`; run it in `ci.yml`.

### 2. Release guard (§ 5.2)
- `ci/verify-release-tag.sh <tag>`: passes only if (1) the tag equals `[workspace.package].version` in `Cargo.toml`, (2) `CHANGELOG.md` has a heading `## [<version>]`, (3) `git merge-base --is-ancestor <tag-commit> origin/main`.
- Self-test `ci/verify-release-tag-self-test.sh` using temporary git repositories (matching tag; wrong tag; missing heading; commit not on main).
- `release.yml`: new first job `verify-tag` (checkout with `fetch-depth: 0`), and `needs: verify-tag` on `build` and `source`.

### 3. Least privilege (§ 5.3)
- `release.yml`: workflow-level `permissions: contents: read`; only the `release` job gets `contents: write`.
- `msix-store.yml`: remove the workflow-level secret `env`. Add a first step that computes `has_cert` from `${{ secrets.MSIX_CERT_BASE64 != '' }}` (boolean only) and exposes it as a step output; the signing step receives `MSIX_CERT_BASE64` / `MSIX_CERT_PASSWORD` through its own step-level `env` and runs `if: steps.<id>.outputs.has_cert == 'true'`. No other step may reference these secrets.
- Add explicit least-privilege `permissions:` to `ci.yml` (`contents: read`).

### 4. Pinned actions and update PRs (§ 5.4)
- Replace every `uses: owner/repo@vX` in all three workflows with the full commit SHA and a trailing comment with the version tag (resolve annotated tags to their commit: `gh api repos/<o>/<r>/git/ref/tags/<tag>` then dereference if `type == tag`). Record each resolution in the review request.
- `.github/dependabot.yml`: ecosystems `github-actions` and `cargo`, weekly, `open-pull-requests-limit: 5`.

### 5. CI coverage and audit (§ 5.5)
- `ci.yml`: add `cargo test -p app` to the Linux job; add `macos-latest` and `windows-latest` jobs running `cargo check --workspace`, `cargo test --workspace`, and the PDFium smoke test against the verified platform asset (`fail-fast: false`; set `PDF_TILE_VIEWER_PDFIUM_DIR` per platform; use bash steps on Windows).
- `actionlint` job (pinned release binary or pinned action) over `.github/workflows`.
- `deny.toml` and a `cargo-deny` job (pinned action), advisories + licenses + bans, `continue-on-error: true` initially. Triage the first report: list findings in the review request; do not suppress advisories without a written reason in `deny.toml`.
- Clippy: **blocking** job `cargo clippy --workspace --all-targets -- -D warnings` after fixing the two existing warnings (RFC 036 § 5.6, owner-approved) — see § 9.

### 6. Documentation
Update `docs/src/contributors/dev.md` (PDFium bump procedure with hashes, release guard, dry-run on a fork) and `docs/src/contributors/testing.md` (new CI jobs). Correct any statement claiming checksum verification existed before. **Do not** touch other documentation (RFC 038 slices own it).

### 7. Release-asset hygiene (added 2026-10-24 after the 2.0.1 release)
The 2.0.1 GitHub Release contained a stray `pdf-tile-viewer-v2.0.0-linux-x64.tar.gz`: the `source` job runs `ci/archive-source.sh dist` inside a checkout that still tracks `dist/…2.0.0….tar.gz` and uploads `dist/*.tar.gz`. Fix in the workflow (do not wait for RFC 038 B′ to untrack the file):
- `source` job: write the source archive to a fresh directory (`"$RUNNER_TEMP/source-out"`) and upload exactly that file (explicit path, `if-no-files-found: error`); the build jobs likewise upload exactly their own `dist/${artifact_name}.{tar.gz,zip}` (already explicit; keep).
- `release` job: after flattening, a **gate step** fails unless every file name starts with `pdf-tile-viewer-v${version}` (version resolved from `Cargo.toml`) and exactly five files exist (four platform archives plus the source archive); print the list. Add a negative self-test to `ci/verify-release-tag-self-test.sh` or a new `ci/verify-release-assets-self-test.sh` (stray file, missing file, wrong version).
- Evidence: the owner's fork dry run must show the gate passing and, with a deliberately placed stray file, failing.

### 8. Version single source and the release-preparation workflow (RFC 036 § 5.7, owner-approved 2026-10-24)
Implement RFC 036 § 5.7 exactly. In order:
1. `packaging/windows/AppxManifest.xml`: set the real `Version="2.0.1.0"` (the current Cargo version `.0`); remove the placeholder comment.
2. `ci/set-version.sh` (also: the CHANGELOG heading must read `## [X.Y.Z] - unreleased`; the script replaces `unreleased` with today's UTC date), `ci/verify-version-consistency.sh` (heading dated or `unreleased`) and their self-tests (`ci/set-version-self-test.sh`, `ci/verify-version-consistency-self-test.sh`, using temporary copies of the files): cases — clean run; dirty tree refused; invalid version refused; CHANGELOG heading missing refused; manifest/Cargo mismatch detected; non-zero revision detected; placeholder detected; pre-release suffix dropped in the manifest only.
3. `ci.yml`: run the consistency check and all self-tests on every push; add `workflow_call` with an optional `ref` input and check out `${{ inputs.ref || github.sha }}`.
4. `.github/workflows/prepare-release.yml` per § 5.7 item 4. The bot identity is `github-actions[bot]`; use the token that `actions/checkout` provides (no personal tokens, no new secrets). Require a successful CI run for the head of `main` (query with `gh api`/`gh run list` using `GH_TOKEN: ${{ github.token }}` and the default read permission); fail clearly otherwise. Never create a tag.
5. `release.yml` `verify-tag`: also run `ci/verify-version-consistency.sh`. `msix-store.yml` and `ci/stage-msix.sh`: remove `msix_revision` everywhere; the stage script verifies the manifest version and writes `msix-version.txt` (single source for the output file name); keep the post-staging checks (no placeholder, no VCLibs dependency, VC runtime DLLs present).
6. Docs: `docs/src/contributors/dev.md` — replace the manual release recipe by the new order of work (§ 5.7), state that the MSIX revision is always 0 and why (link the Microsoft Learn page), document the bot-bypass note and the `main` protection status; correct the DEC-015 wording wherever it appears in tracked docs.
7. **Evidence:** self-tests pass locally and in CI; `actionlint` clean; demonstrate the guards by deliberately breaking each (manifest mismatch, non-zero revision) in a scratch copy. **Dry-run instructions for the owner (fork):** how to create a throw-away fork, dispatch *Prepare release* with a pre-release version (for example `2.0.2-rc.1`, with a matching CHANGELOG heading on the fork), what the resulting commit must contain (`git show --stat`), that CI runs on it, and that no tag exists. You do **not** run it on the real repository.


### 9. Pinned toolchain and blocking clippy (RFC 036 § 5.6, owner-approved 2026-10-24)
1. Fix the two clippy warnings first (`outline_panel.rs` redundant closure; the needless borrow in `zoom_overlay/link_layer.rs`) — behavior-preserving, listed in the review request as the only app-code change of this slice.
2. `rust-toolchain.toml` at the repository root: `[toolchain] channel = "<exact current stable, e.g. 1.99.1>"`, `components = ["rustfmt", "clippy"]`, `profile = "minimal"`. Update every workflow to use it (no `rustup default stable`; a `rustup show` step installs the pinned toolchain; add per-job `rustup target add` for release targets afterwards; include `rust-toolchain.toml` in cache keys). Confirm on each OS runner in the fork dry run.
3. `ci.yml`: blocking `clippy` job; `docs/src/contributors/dev.md`: the toolchain statement becomes true; add the bump procedure (monthly check, one commit, fix new lints) and "no MSRV declared"; `testing.md`: the gate now includes clippy.
4. Evidence: clippy clean on all targets locally and in CI; a deliberately introduced warning in a scratch copy fails the job.

## Non-change scope
Signing behavior, artifact layout, packaged-artifact smoke logic, the `msix_revision` input semantics, release notes generation, application code.

## Required tests and evidence
1. Both self-tests pass locally and in CI.
2. `actionlint` clean on the final workflows (run locally if available, and in CI).
3. Local gate green (`cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, smoke).
4. A verification note proving that tampering is caught: run `ci/fetch-pdfium.sh` against a deliberately wrong hash in a scratch copy and show the failure.
5. **Dry-run instructions for the owner** in the review request: how to create a throw-away fork, push a pre-release tag `2.0.1-rc.1` **on the fork only**, which jobs must pass, which artifacts to inspect, expected `verify-tag` behavior with a wrong tag. The owner executes this; do not do it yourself.
6. Because workflows cannot be fully tested locally, expect to push iterative fixes to `main`; keep each push small, and do not hand over while CI is red.

## Acceptance criteria
RFC 036 § 8; plus § 7 above (no file other than the five expected assets can reach a release): no step but signing can read signing secrets; `verify-tag` blocks the stray-tag scenario (shown by the self-test); every PDFium download fails closed on a wrong hash; CI shows `cargo test -p app` and macOS/Windows jobs; actions pinned; Dependabot configured.

## Prohibited shortcuts
Skipping verification "for macOS"; storing hashes anywhere but `ci/pdfium-checksums.txt`; adding `continue-on-error` to the self-tests, `verify-tag`, or hash verification; suppressing `actionlint` or `cargo-deny` findings without a written reason; creating tags.

## Security constraints
No secret values in logs or files; no `pull_request_target`; no use of `${{ github.event.* }}` inside shell without quoting through `env:`.

## Known risks
Hash source disagreement (stop and report); runner image differences on Windows (Git Bash availability); flaky new jobs (do not make them required branch checks yet — the architect will decide after one clean week).

## Review request
`.git-exclude/review-request/036-supply-chain/README.md` (single entry point): summary, commits, changed files, deviations, commands and results, hash-procedure record, action-SHA resolution table, `cargo-deny` triage list, dry-run instructions, unresolved issues, requested review focus.
