# Local Development

## Prerequisites

- [rustup](https://rustup.rs/) — installs `rustc` and `cargo` from `rust-toolchain.toml`
- Linux system packages (required by the Dioxus Desktop GUI layer):
  ```sh
  sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev libssl-dev pkg-config
  ```
- PDFium (fetched by `ci/fetch-pdfium.sh`):
  ```sh
  bash ci/fetch-pdfium.sh        # downloads chromium/7763 to ci/.pdfium/
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

## Code style

- Rust 2024 edition, `rustfmt` enforced (`cargo fmt --check`).
- Files are split at ~300 ELOC; splitting is required above ~500.
- Tests go in `src/tests.rs` (or `src/tests/` for larger test suites).
- All public items should have doc comments.
- English for all code comments and documentation.
