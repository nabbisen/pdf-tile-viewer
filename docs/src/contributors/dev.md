# Local Development

## Prerequisites

- [rustup](https://rustup.rs/) — installs the pinned toolchain from `rust-toolchain.toml` on first use (see [Toolchain pin](#toolchain-pin))
- Linux system packages (required by the Dioxus Desktop GUI layer):
  ```sh
  sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev libssl-dev pkg-config
  ```
- PDFium (fetched by `ci/fetch-pdfium.sh`):
  ```sh
  bash ci/fetch-pdfium.sh        # downloads chromium/7920 to ci/.pdfium/
  ```

## Running

```sh
PDF_TILE_VIEWER_PDFIUM_DIR="$(pwd)/ci/.pdfium" cargo run
```

## Default build set and the GUI dependency tradeoff

`crates/app` is in the workspace `default-members`, which means plain
`cargo build`, `cargo check`, and `cargo test` (without `-p` or
`--exclude`) all include the app crate.

**Consequence:** these commands require the WebKit/GTK system packages
listed above, because `dioxus-desktop` links against them.

This is intentional — PDF Tile Viewer is a GUI application, and the
default commands should build and run it without extra flags.

**If you want faster library-only iterations** (no GUI deps needed,
works on headless CI machines), exclude the app crate:

```sh
# Unit tests for all library crates — no system GUI deps required
cargo test --workspace --exclude app

# Quick type-check of library crates only
cargo check --workspace --exclude app
```

The PDFium engine smoke tests are separate and still need the env var:

```sh
PDF_TILE_VIEWER_PDFIUM_DIR="$(pwd)/ci/.pdfium" \
    cargo test -p pdf_engine --test smoke
```

## Workspace layout

```
crates/
  domain/          Pure domain types (no I/O, no GUI, no PDFium)
  pdf_engine/      PDFium binding + serialized engine worker
                   (tests/fixtures/: hand-built test PDFs + generator)
  packaging/       App-directory policy, PDFium path resolution, BuildInfo
  app_services/    Orchestration: file intake, settings, history, platform
  app/             Dioxus Desktop binary  ← GUI deps live here
docs/
  design/          ADR, feasibility study, RFC roadmap (read-only reference)
  src/             mdbook source (this documentation)
ci/                CI helpers (fetch-pdfium.sh, archive-source.sh, package-linux.sh)
rfcs/              RFC files following the lifecycle policy
```


## Two release pathways

PDF Tile Viewer has two independent release outputs:

| Pathway | Workflow | Trigger | Output | Distribution |
|---------|----------|---------|--------|--------------|
| Executable build | `release.yml` | semver tag (`2.0.0`) | `.tar.gz` / `.zip` per OS | GitHub Release |
| Microsoft Store | `msix-store.yml` | successful `release.yml`, manually published GitHub Release, or manual (`workflow_dispatch`) | `.msix` | Partner Center (Windows) |

### Executable build (Linux, macOS, Windows)

Tag-triggered, fully automated. See the `release.yml` section above. Produces
a self-contained archive per platform with the binary, bundled PDFium,
licenses, and docs, attached to a GitHub Release.

### Microsoft Store (Windows MSIX)

The MSIX artifact builds automatically after a successful `release.yml` run.
This is necessary because `release.yml` creates the GitHub Release with
`GITHUB_TOKEN`, and GitHub does not create follow-on workflow runs for most
events caused by `GITHUB_TOKEN`. The workflow also runs for manually published
GitHub Releases and can be started manually from the **Microsoft Store (MSIX)**
workflow in the Actions tab. It builds the Windows binary, bundles PDFium,
stages the package layout via `ci/stage-msix.sh`, and runs `makeappx` to
produce a `.msix`. Uploading to Partner Center remains manual.

Notes:

- **Version.** The MSIX version is always `<Cargo core X.Y.Z>.0`. The Microsoft
  Store reserves the fourth segment and requires it to be `0`
  ([App package requirements for MSIX](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements)),
  and a pre-release suffix is not permitted, so `2.1.0-rc.1` becomes `2.1.0.0`.
  There is **no revision input**: re-submitting the same code means a new patch
  version. `packaging/windows/AppxManifest.xml` holds the real version;
  `ci/set-version.sh` writes it and `ci/stage-msix.sh` fails the build if it
  differs from Cargo (see [Release procedure](#release-procedure)).
- **Manifest and runtime.** The workflow bundles the Visual C++ runtime DLLs
  app-locally in the MSIX instead of declaring a Store framework package
  dependency. Keep the `Identity Name`, `Publisher`, and Store assets in sync
  with the Partner Center listing.
- **Signing.** The MSIX is built unsigned by default. Partner Center re-signs
  Store submissions with the Store certificate, so unsigned is acceptable for
  Store upload. To produce a sideloadable signed package, add the
  `MSIX_CERT_BASE64` and `MSIX_CERT_PASSWORD` repository secrets; the workflow
  signs with `signtool` when they are present. Only the signing step can read
  them (the earlier steps only learn *whether* the secret exists).
- **Submission is not automated.** The workflow produces the `.msix` as an
  artifact and stops. Uploading to Partner Center and submitting for
  certification is a deliberate, human-reviewed step. Wiring an automated
  Partner Center submission would require Partner Center API credentials and
  is intentionally left out.


## Code style

- Rust 2024 edition, `rustfmt` enforced (`cargo fmt --check`).
- Files are split at ~300 ELOC; splitting is required above ~500.
- Unit tests live next to the module they test: `src/foo.rs` uses
  `src/foo/tests.rs` with `#[cfg(test)] mod tests;`. Avoid central
  `src/tests.rs` for ordinary module tests.
- Crate-level `tests/` remains valid for integration tests that exercise the
  public crate boundary or real external resources, such as PDFium smoke tests.
- All public items should have doc comments.
- English for all code comments and documentation.

### Signal borrow rule

Never keep a `read()` / `peek()` guard alive across a `set()` / `write()` of the
same signal. This includes **temporaries inside call arguments**: they live until
the end of the enclosing statement, so `change_scale(*scale.peek() + 0.2)` still
holds the borrow when `change_scale` calls `scale.set(..)`. The result is an
`AlreadyBorrowed` panic in an event callback, which aborts the whole application
(this shipped in 2.0.0, RFC 039). Read into a `let` first, and prefer pure
functions in `domain` for the arithmetic (see `ViewerScale::next`):

```rust
let current = *scale.peek();                 // guard dropped at the end of this statement
if let Some(next) = ViewerScale::next(current, change) {
    scale.set(next);
}
```

The reasoning and further Dioxus pitfalls are in
[Lessons for Dioxus Desktop Apps](dioxus-desktop-lessons.md). Before a release,
run the [Release QA Checklist](release-qa.md) on a release build.

## Toolchain pin

`rust-toolchain.toml` pins the compiler (exact stable version, components
`rustfmt` and `clippy`, minimal profile). Local development, CI and release builds
use the same compiler, so `main` cannot turn red because a new stable release
added lints, and a release candidate is reproducible. **No MSRV is declared.**

Bump procedure (monthly check of the upstream stable release, one commit):

1. Change `channel` in `rust-toolchain.toml` to the new version.
2. Run `cargo clippy --workspace --all-targets -- -D warnings`, fix any new lints
   (behavior-preserving), then the full gate (`cargo fmt --check`,
   `cargo check --workspace`, `cargo test --workspace`, PDFium smoke).
3. Commit the pin and the lint fixes together; CI must be green before anything else
   is committed.

## Continuous integration and releases

Four GitHub Actions workflows live in `.github/workflows/`. Every third-party action
is pinned to a **full commit SHA** (with the version in a trailing comment) and
workflows default to `permissions: contents: read`; Dependabot (weekly) opens update
pull requests for actions and Cargo dependencies, which the owner merges between
work slices. The Rust toolchain is set up by the local composite action
`.github/actions/setup-rust` (pinned version, cargo cache keyed by `Cargo.lock` and
`rust-toolchain.toml`).

### `ci.yml` — on every push to `main`, every pull request, and as a reusable workflow

| Job | What it checks | Blocking |
|-----|----------------|:--------:|
| `rustfmt` | `cargo fmt --all --check` | yes |
| `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | yes |
| `check & test (linux)` | `cargo check --workspace`, `cargo test --workspace --exclude app`, `cargo test -p app`, PDFium smoke on the checksum-verified Linux asset | yes |
| `check & test (macos-arm64, windows-x64)` | `cargo check --workspace`, `cargo test --workspace`, PDFium smoke on the verified platform asset | yes (see note) |
| `release tooling` | `shellcheck ci/*.sh`, `ci/verify-version-consistency.sh`, and the self-tests of every release guard | yes |
| `actionlint` | pinned, checksum-verified `actionlint` over `.github/workflows` (also shellchecks `run:` scripts) | yes |
| `cargo deny` | advisories, licences, bans, sources (`deny.toml`) | yes (the only ignores are the two documented unmaintained build-time proc-macros) |

macOS Intel is covered by the release matrix only. The macOS/Windows jobs are new;
do not make them required branch checks until they have been green for a week.
`workflow_call` with the input `ref` lets *Prepare release* run the same CI on the
commit it creates (pushes made with the built-in token do not start workflows).

This mirrors the local gate in [Running Tests](testing.md).

### `release.yml` — on pushing a semver tag (no `v` prefix)

Pushing a tag is the explicit, human-initiated act that starts a release.
The workflow does **not** run on branch pushes.

The first job, `verify-tag`, blocks everything else unless (`ci/verify-release-tag.sh`
and `ci/verify-version-consistency.sh`):

1. the tag equals `[workspace.package].version` in `Cargo.toml` at the tagged commit,
2. `CHANGELOG.md` has a **dated** heading `## [<version>] - YYYY-MM-DD`,
3. Cargo, the four path-dependency versions and the MSIX manifest agree, and
4. the tagged commit is an ancestor of `origin/main`.

Then it builds the RFC 014 artifact matrix, each with its bundled PDFium:

| Tag triggers | Runner | Artifact |
|--------------|--------|----------|
| `linux-x64` | `ubuntu-latest` | `…-linux-x64.tar.gz` |
| `windows-x64` | `windows-latest` | `…-windows-x64.zip` |
| `macos-arm64` | `macos-latest` | `…-macos-arm64.tar.gz` |
| `macos-x64` | `macos-15-intel` | `…-macos-x64.tar.gz` |

Each build fetches the pinned PDFium for its platform **and verifies its SHA-256**
against `ci/pdfium-checksums.txt` before extracting, compiles the app, runs the
engine smoke tests against that PDFium (the RFC 014 §7 package verification step),
stages the RFC 014 §6 contents (binary, PDFium library, LICENSE, NOTICE, CHANGELOG,
README, `notes.txt` which records the bundled library's SHA-256), compresses them
into a flat archive with `bin/` and `resources/` at archive root, then runs the
RFC 018 packaged-artifact smoke gate against the final archive. Each build uploads
exactly its own archive. A source archive (`ci/archive-source.sh`) is built in
parallel **outside the checkout** (a checkout can still track an old tarball under
`dist/`). The final `release` job — the only job with `contents: write` — refuses
to publish unless `ci/verify-release-assets.sh` finds exactly the five expected,
correctly versioned files, then creates the GitHub Release. The release notes are
the **CHANGELOG section of that version** (`ci/extract-changelog.sh`, which fails if
the dated section is missing or empty) plus a fixed footer; pre-release tags use the
same mechanism. GitHub's auto-generated notes are not used.

The packaged-artifact smoke can also be run manually after creating an archive:

```sh
bash ci/smoke-release-artifact.sh dist/pdf-tile-viewer-v<version>-linux-x64.tar.gz
```

It extracts the archive, derives the production `resources/` root from
`bin/pdf-tile-viewer`, resolves bundled PDFium in production mode, and binds
PDFium without using `PDF_TILE_VIEWER_PDFIUM_DIR`.

A tag containing `-alpha`, `-beta`, or `-rc` is published as a GitHub
**pre-release**; a clean `X.Y.Z` tag is a full release.

### `prepare-release.yml` — manual (Actions tab), input `version`

Creates the release commit on `main` so that versions and dates cannot be edited by
hand wrongly. It requires a successful CI run on the head of `main`, runs
`ci/set-version.sh`, commits as `github-actions[bot]` ("Prepare release X.Y.Z"),
pushes to `main` without force (it fails if `main` moved) and then runs CI on that
exact commit. **It never creates a tag.** Its job has `contents: write` (push) and
`actions: read` (read-only query of CI runs) and no other scope.

`main` has no branch protection today (checked 2026-10-24). If protection is ever
enabled, `github-actions[bot]` needs an explicit bypass for this push.

### Release procedure

Order of work for a release (owner: tagging and publishing; DEC-001):

1. Write the CHANGELOG section as `## [X.Y.Z] - unreleased` (people write release
   notes) and commit it to `main`; wait for CI.
2. Run **Prepare release** with `X.Y.Z`. Wait for it and its CI job.
3. Readiness check (architect) on the printed commit.
4. The owner tags **that commit** (annotated, no `v`) and pushes the tag:
   `git tag -a X.Y.Z <commit> -m "X.Y.Z" && git push origin X.Y.Z`.
5. `release.yml` publishes the GitHub Release, then `msix-store.yml` builds the
   `.msix` artifact; the Store upload stays manual. Verify the published assets
   (the five names above) and read the release notes on the Release page: they are
   the CHANGELOG text plus the footer.
6. If a workflow fails: never re-tag a published name; fix forward on `main` and use
   the next patch version. A run that failed before the GitHub Release existed can
   be re-run from the Actions tab.

The version tooling: `ci/set-version.sh <X.Y.Z[-pre]>` (the only tool that changes
versions: `Cargo.toml`, `Cargo.lock`, the MSIX manifest, and it dates the CHANGELOG
heading), `ci/verify-version-consistency.sh` (CI), `ci/verify-release-tag.sh` and
`ci/verify-release-assets.sh` (release). Each has a self-test (`ci/*-self-test.sh`)
that CI runs.

### PDFium pin and checksums

The PDFium release tag (`PDFIUM_RELEASE_TAG`, currently `chromium/7920`) is pinned
in the workflows and `ci/fetch-pdfium.sh`, and **every download is verified** with
`ci/verify-sha256.sh` against `ci/pdfium-checksums.txt` (format:
`sha256␣␣release_tag␣␣asset_name`); a missing or mismatching entry fails the job
before anything is extracted. Honest limitation: a pin protects against later silent
replacement of an asset, not against a compromise that already existed when the hash
was recorded.

Bump procedure (one commit; tag and hashes change together):

1. Choose the new `chromium/NNNN` release of `bblanchon/pdfium-binaries` that
   matches the `pdfium-render` symbol set, and set `PDFIUM_RELEASE_TAG` in
   `ci.yml`, `release.yml`, `msix-store.yml` and `ci/fetch-pdfium.sh`, and the tag in
   `ci/package-linux.sh`.
2. Download each asset (`pdfium-linux-x64`, `pdfium-linux-arm64`, `pdfium-win-x64`,
   `pdfium-mac-arm64`, `pdfium-mac-x64`) **twice, in separate invocations**, and
   confirm the two SHA-256 values are identical; also compare them with GitHub's
   release-asset digest (`gh api repos/bblanchon/pdfium-binaries/releases/tags/chromium/NNNN
   --jq '.assets[] | "\(.name) \(.digest)"'`). If the sources disagree, stop.
3. Add the lines to `ci/pdfium-checksums.txt`; run `bash ci/verify-sha256-self-test.sh`
   and `bash ci/fetch-pdfium.sh`, then the smoke tests.

### Dependency advisories and Dependabot

`cargo deny` is a blocking CI job: a new advisory against the committed `Cargo.lock`
turns `main` red (as a new lint would), which is intended. Fix it with a
**lockfile-only** update of the named crates — `cargo update -p <crate>` (never a
bare `cargo update`, which churns the whole lockfile), no `Cargo.toml` change — then
run the full gate and a release-build smoke check (the shipped binary changes with
the lockfile). If the fix needs a manifest change or a major bump, write an RFC/handoff
instead. An advisory may be ignored in `deny.toml` only with a written reason (for
example an unmaintained build-time proc-macro without an upgrade path).

Dependabot opens weekly pull requests for Cargo dependencies and workflow actions
(`.github/dependabot.yml`). Between work slices the owner merges **only** pull
requests that change nothing but the lockfile (or a pinned action SHA) and pass CI.
Anything that edits a manifest, a major version, and anything for `pdfium-render`
is not merged from a bot PR: `pdfium-render` is bumped only together with the PDFium
pin and checksums (DEC-012, above), and the config ignores it; `dirs >= 7` and
`base64 >= 0.23` are ignored as API-changing proposals awaiting a reviewed change.

### Trying workflow changes on a fork (dry run)

Workflows cannot be fully tested locally. For a change to a release workflow, the
owner creates a throw-away fork, pushes a pre-release tag **on the fork only**
(for example `2.0.2-rc.1` on a commit whose CHANGELOG, versions and manifest were
prepared with `ci/set-version.sh`) and checks that: `verify-tag` passes, all four
platform builds and the source archive succeed, the asset gate passes, and the
resulting GitHub pre-release holds exactly the five expected files. A tag that does
not match the workspace version, or a commit that is not on `main`, must be stopped
by `verify-tag`. *Prepare release* is dry-run the same way (dispatch it on the fork
with a pre-release version and a matching `unreleased` CHANGELOG heading): the new
commit must contain only the four version files, CI must run on it, and no tag must
exist afterwards.
