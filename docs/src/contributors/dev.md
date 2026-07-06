# Local Development

## Prerequisites

- [rustup](https://rustup.rs/) — installs `rustc` and `cargo` from `rust-toolchain.toml`
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
  packaging/       App-directory policy, PDFium path resolution, BuildInfo
  app_services/    Orchestration: file intake, settings, history, platform
  app/             Dioxus Desktop binary  ← GUI deps live here
docs/
  design/          ADR, feasibility study, RFC roadmap (read-only reference)
  src/             mdbook source (this documentation)
fixtures/          Minimal hand-built PDFs for testing
ci/                CI helpers (fetch-pdfium.sh, archive-source.sh, package-linux.sh)
rfcs/              RFC files following the lifecycle policy
```


## Two release pathways

PDF Tile Viewer has two independent release outputs:

| Pathway | Workflow | Trigger | Output | Distribution |
|---------|----------|---------|--------|--------------|
| Executable build | `release.yml` | semver tag (`2.0.0`) | `.tar.gz` / `.zip` per OS | GitHub Release |
| Microsoft Store | `msix-store.yml` | manual (`workflow_dispatch`) | `.msix` | Partner Center (Windows) |

### Executable build (Linux, macOS, Windows)

Tag-triggered, fully automated. See the `release.yml` section above. Produces
a self-contained archive per platform with the binary, bundled PDFium,
licenses, and docs, attached to a GitHub Release.

### Microsoft Store (Windows MSIX)

Manual only. Run the **Microsoft Store (MSIX)** workflow from the Actions
tab. It builds the Windows binary, bundles PDFium, stages the package layout
via `ci/stage-msix.sh`, and runs `makeappx` to produce a `.msix`.

Notes:

- **Version.** MSIX requires a 4-part numeric version `X.Y.Z.R` and does not
  permit pre-release suffixes. The workflow derives `X.Y.Z` from the semver
  core in `Cargo.toml` (so `2.0.0-beta.8` → `2.0.0`) and uses the
  `msix_revision` dispatch input as the 4th part `R` (default `0`). Bump the
  revision when re-submitting the same semver.
- **Manifest.** `packaging/windows/AppxManifest.xml` carries a `@VERSION@`
  placeholder that `ci/stage-msix.sh` substitutes at build time. Keep the
  `Identity Name`, `Publisher`, and Store assets in sync with the Partner
  Center listing.
- **Signing.** The MSIX is built unsigned by default. Partner Center re-signs
  Store submissions with the Store certificate, so unsigned is acceptable for
  Store upload. To produce a sideloadable signed package, add the
  `MSIX_CERT_BASE64` and `MSIX_CERT_PASSWORD` repository secrets; the workflow
  signs with `signtool` when they are present.
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

## Continuous integration and releases

Two GitHub Actions workflows live in `.github/workflows/`:

### `ci.yml` — on every push to `main` and every pull request

- `rustfmt` check (`cargo fmt --all --check`)
- `cargo check --workspace`
- library + service unit tests (`cargo test --workspace --exclude app`)
- engine smoke tests against the pinned PDFium (`cargo test -p pdf_engine
  --test smoke`)

This mirrors the local gate in [Running Tests](testing.md).

### `release.yml` — on pushing a semver tag (no `v` prefix)

Pushing a tag is the explicit, human-initiated act that starts a release.
The workflow does **not** run on branch pushes.

It builds the RFC 014 artifact matrix, each with its bundled PDFium:

| Tag triggers | Runner | Artifact |
|--------------|--------|----------|
| `linux-x64` | `ubuntu-latest` | `…-linux-x64.tar.gz` |
| `windows-x64` | `windows-latest` | `…-windows-x64.zip` |
| `macos-arm64` | `macos-latest` | `…-macos-arm64.tar.gz` |
| `macos-x64` | `macos-13` | `…-macos-x64.tar.gz` |

Each build fetches the pinned PDFium for its platform, compiles the app,
runs the engine smoke tests against that PDFium (the RFC 014 §7 package
verification step), stages the RFC 014 §6 contents (binary, PDFium library,
LICENSE, NOTICE, CHANGELOG, README, notes), compresses them into a flat
archive with `bin/` and `resources/` at archive root, then runs the RFC 018
packaged-artifact smoke gate against the final archive. A source
archive (`ci/archive-source.sh`) is built in parallel. All artifacts are
attached to a GitHub Release.

The packaged-artifact smoke can also be run manually after creating an archive:

```sh
bash ci/smoke-release-artifact.sh dist/pdf-tile-viewer-v<version>-linux-x64.tar.gz
```

It extracts the archive, derives the production `resources/` root from
`bin/pdf-tile-viewer`, resolves bundled PDFium in production mode, and binds
PDFium without using `PDF_TILE_VIEWER_PDFIUM_DIR`.

Negative layout self-tests for the smoke script can be run with:

```sh
bash ci/smoke-release-artifact-self-test.sh
```

A tag containing `-alpha`, `-beta`, or `-rc` is published as a GitHub
**pre-release**; a clean `X.Y.Z` tag is a full release.

To cut a release:

```sh
# 1. Bump version in Cargo.toml, update CHANGELOG.md, commit.
# 2. Tag and push:
# Tags use no leading "v" (project convention).
git tag 2.0.0
git push origin 2.0.0
```

> The PDFium release tag is pinned in the GitHub workflows and local helper
> scripts. Bump every `PDFIUM_RELEASE_TAG` together and re-run the suite when
> updating PDFium.
