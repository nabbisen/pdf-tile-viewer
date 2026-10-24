---
project: PDF Tile Viewer
document_family: 2.0 corrective RFCs
language: English
date: 2026-10-21
status: Accepted (planned for 2.0.2)
accepted: 2026-10-21
baseline: PDF Tile Viewer 2.0.0
priority: Corrective
depends_on: RFC 003, RFC 014, RFC 015, RFC 016, RFC 018
amends: RFC 014 (§ CI/release), RFC 016 (§ native library integrity)
origin: architect onboarding review 2026-10-21 (internal), owner decision D5
amended: 2026-10-24 — § 5.4a release-asset gate; § 5.7 version single source and release-preparation workflow (owner decision)
---

# RFC 036 — Release Supply-Chain and Workflow Hardening

## 1. Summary

Make the build and release pipeline verify what it ships and grant each job
only the privileges it needs: checksum-verified PDFium, a tag-equals-version
guard, least-privilege workflow permissions and secret scoping, commit-pinned
actions with automated update PRs, a dependency audit, and CI that runs the
tests it already has on all three desktop platforms.

This RFC changes release **machinery** only. It does not tag, publish, sign
differently, or automate Store submission (DEC-001, DEC-014 unchanged).

## 2. Motivation (evidence)

| ID | Finding | Evidence |
|---|---|---|
| S-01 | PDFium is downloaded from a GitHub release and shipped to users with **no checksum or signature check** in `ci/fetch-pdfium.sh`, `release.yml`, `msix-store.yml`, or `ci.yml`. The earlier handoff claim "checksum-fetched" (SEC-4) is incorrect | `grep` of `ci/` and `.github/` finds no hash verification |
| S-02a | `release.yml` sets `permissions: contents: write` for **all** jobs, including builds that run third-party build scripts | `release.yml:24` |
| S-02b | `msix-store.yml` exposes `MSIX_CERT_BASE64` and `MSIX_CERT_PASSWORD` as workflow-wide `env`, readable by every step including `cargo build` | `msix-store.yml:33-39` |
| S-03a | Nothing checks that the pushed tag equals the workspace version. A local tag `2.0.1` points at a commit whose `Cargo.toml` says `2.0.0`; pushing it would publish `v2.0.0`-named artifacts under release `2.0.1` | `git for-each-ref`, `release.yml` |
| S-03b | All actions use mutable tags (`@v7`, `@v8`, `@v3`) | `grep uses: .github/workflows` |
| G-06 | `ci.yml` runs on Linux only and never runs `cargo test -p app` (19 tests), although GUI dependencies are already installed; macOS and Windows compile for the first time at tag time | `ci.yml` |
| — | No dependency advisory or licence check | no `deny.toml` / audit job |
| S-04 | The `source` job uploads `dist/*.tar.gz` from a checkout that tracks an old release tarball; the 2.0.1 release shipped the 2.0.0 binary (with the shortcut abort) as an extra asset | 2.0.1 release assets, 2026-10-24 |

## 3. Goals

- Every native binary that is bundled has a pinned, verified SHA-256.
- A release can only start from a tag that matches the code it builds.
- No job holds a privilege or secret it does not use.
- Third-party actions cannot change underneath us silently.
- Failures that today surface at tag time surface in CI.

## 4. Non-goals

- Building PDFium from source (see § 6).
- Code-signing changes, SBOM/provenance attestation, reproducible builds
  (candidate future work).
- Automating Microsoft Store submission.

## 5. Design

### 5.1 PDFium integrity

- Add `ci/pdfium-checksums.txt`: one line per `(release-tag, asset-name)` with
  its SHA-256.
- Add `ci/verify-sha256.sh <file> <expected>` (portable: `sha256sum`, else
  `shasum -a 256`), failing closed. All fetch paths call it:
  `fetch-pdfium.sh`, `release.yml`, `msix-store.yml`, `ci.yml`.
- A missing entry for the requested tag/asset is an error (no
  trust-on-download).
- Bump procedure (documented in `dev.md`): tag and hashes change together in one
  change; the new hashes are taken from two independent downloads and, where
  GitHub exposes it, compared with the release-asset digest.
- Release archives record the bundled library's SHA-256 in `notes.txt`.

Honest limitation: pinning protects against later silent replacement of an
asset, not against a compromise that already existed at the time the hash was
recorded.

### 5.2 Release guard

Add `ci/verify-release-tag.sh` and a first job `verify-tag` in `release.yml`
that all build jobs `need`. It fails unless:

1. the tag equals `[workspace.package].version` in `Cargo.toml`;
2. `CHANGELOG.md` contains a matching `## [<version>]` heading;
3. the tag's commit is an ancestor of `origin/main`.

The script has a negative self-test in the style of
`ci/smoke-release-artifact-self-test.sh`.

### 5.3 Least privilege

- Workflow default: `permissions: contents: read`.
- Only the final `release` job (which creates the GitHub Release) gets
  `contents: write`.
- `msix-store.yml`: remove workflow-level secret `env`. A tiny first step
  computes `has_cert` from `${{ secrets.MSIX_CERT_BASE64 != '' }}` (a boolean,
  not the secret); the signing step alone receives the secrets via step-level
  `env`.

### 5.4 Pinned actions and update PRs

Pin every `uses:` to a full commit SHA with a trailing `# vX.Y.Z` comment. Add
`.github/dependabot.yml` for the `github-actions` and `cargo` ecosystems
(weekly). Updates arrive as bot pull requests that the owner merges only between slices (the project works on `main` with one active actor at a time).

### 5.4a Release-asset gate (added 2026-10-24)

The source archive is produced outside the checkout's `dist/` and uploaded by explicit path; the `release` job fails unless exactly the five expected, correctly versioned assets are present (finding S-04).

### 5.7 Version single source and release-preparation workflow (amendment 2026-10-24, owner decision)

**Facts.** (a) The Microsoft Store reserves the fourth segment of the package
version and requires it to be `0`
([Microsoft Learn: App package requirements for MSIX](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements)),
so the MSIX version is always `<Cargo core X.Y.Z>.0`; the `msix_revision` dispatch
input (DEC-015's revision) was a mistake and is removed. A same-version
re-submission is done with a new patch version. (b) The workflow pasted that input
unvalidated into a PowerShell script; the version was also computed in two places
(stage script and workflow). (c) The manifest in the repository carried a
placeholder, which is not a valid manifest.

**Design (owner's model: CI updates the version files at each release; we wait for it).**

1. `packaging/windows/AppxManifest.xml` holds the **real** version
   (`Version="X.Y.Z.0"`); no placeholder anywhere.
2. `ci/set-version.sh <X.Y.Z[-pre]>` is the only tool that changes versions. It
   refuses to run on a dirty tree; validates the argument; updates
   `[workspace.package].version` and the four `[workspace.dependencies]` path
   versions in `Cargo.toml`; refreshes `Cargo.lock` (workspace crates only); sets
   the manifest to `<core>.0` (a pre-release suffix is dropped); requires that
   `CHANGELOG.md` already contains `## [<version>]` (release notes are written by
   people, not by CI); prints a summary. It has a self-test.
3. `ci/verify-version-consistency.sh` (self-tested) checks: Cargo version core ==
   manifest core, manifest revision is `0`, no `@…@` placeholder, and a CHANGELOG
   heading for the Cargo version exists. It runs in `ci.yml` on every push and as
   part of `verify-tag` (§ 5.2), which additionally requires tag == Cargo version.
4. `.github/workflows/prepare-release.yml` (`workflow_dispatch`, input `version`):
   workflow permissions `contents: read`; the single job `prepare` gets
   `contents: write` and nothing else. It requires that the head of `main` has a
   successful CI run, runs `ci/set-version.sh`, commits as `github-actions[bot]`
   ("Prepare release X.Y.Z"), and pushes to `main` without force (fails if `main`
   moved). Because pushes made with the built-in token do not start other workflows,
   a second job calls `ci.yml` as a reusable workflow (`workflow_call`, input `ref`)
   on the new commit, so CI evidence exists for exactly the commit that will be
   tagged. The run summary prints the commit and the tag command. A concurrency
   group prevents parallel runs. **The workflow never creates a tag**; tagging and
   publishing stay a human act (DEC-001).
5. `msix-store.yml`: the `msix_revision` input is removed. `ci/stage-msix.sh` no
   longer substitutes anything: it copies the manifest, **verifies** that its
   version equals `<Cargo core>.0`, and writes the version to a file that the
   workflow reads for the output file name (single source). No `${{ … }}`
   interpolation of inputs or event data inside shell scripts.
6. `main` has no branch protection today (verified 2026-10-24). If protection is
   enabled later, the bot needs an explicit bypass; recorded in `dev.md`.

**Order of release work after this RFC:** write the CHANGELOG section → run
*Prepare release* → wait for it and its CI to finish → readiness check → tag that
commit → `release.yml` and `msix-store.yml` follow.

### 5.5 CI coverage

- `ci.yml` adds `cargo test -p app` (Linux).
- Add `macos-latest` and `windows-latest` jobs running `cargo check
  --workspace`, `cargo test --workspace`, and the PDFium smoke test against the
  checksum-verified asset for that platform. (The `release.yml` matrix keeps
  building all four artifacts; macOS Intel is covered by the release matrix
  only.)
- Add `actionlint` for workflow files.
- Add `cargo deny` (advisories, licences, bans) via a SHA-pinned action,
  initially non-blocking to establish a baseline, then blocking after triage.

### 5.6 Open question for the owner

Promote `cargo clippy --workspace --all-targets -- -D warnings` from advisory
(DEC-016) to a blocking job? There are 2 warnings today; the change would also
let RFC 037 enforce `clippy::use_debug` in the `app` crate. Not assumed by this
RFC.

## 6. Alternatives considered

- **Build PDFium from source in CI.** Strongest provenance but very large
  build time/complexity and platform matrix cost; rejected for now, revisit with
  RFC-FUTURE F01 (static linking).
- **Vendor PDFium binaries in the repository.** Rejected: repository bloat and
  no better trust root.
- **Verify a signature instead of a hash.** The upstream does not publish
  detached signatures for the binaries.

## 7. Security considerations

This RFC is itself a security change: it reduces supply-chain exposure of the
signing certificate (S-02b), of repository write access (S-02a), of bundled
native code (S-01), and of workflow code (S-03b). Hash and pin files are
reviewable as ordinary commits and handoff review requests.

## 8. Tests and acceptance

- Self-tests for `verify-sha256.sh` (match, mismatch, missing entry, missing
  file) and `verify-release-tag.sh` (matching tag, wrong tag, missing
  changelog heading, non-main commit).
- Dry run of every changed workflow on a fork with a throw-away pre-release tag
  (owner-performed; DEC-001 applies).
- Acceptance: no workflow step can read the signing secrets except signing; the
  `verify-tag` job blocks the stray-tag scenario; all downloads fail closed
  when a hash is wrong; CI shows `cargo test -p app` and the macOS/Windows jobs.

## 9. Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| Pinned SHAs go stale | Low | Dependabot PRs |
| New macOS/Windows CI jobs are flaky | Medium | Start as required only after one clean week; keep `fail-fast: false` |
| Hash bump friction | Low | Documented one-command procedure |
