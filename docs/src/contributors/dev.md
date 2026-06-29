# Local Development

## Prerequisites

- [rustup](https://rustup.rs/) — installs `rustc` and `cargo` from `rust-toolchain.toml`
- Linux system packages:
  ```sh
  sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev libssl-dev pkg-config
  ```
- PDFium (fetched by `ci/fetch-pdfium.sh`):
  ```sh
  bash ci/fetch-pdfium.sh        # downloads chromium/7763 to ci/.pdfium/
  ```

## Running

```sh
PDF_TILE_VIEWER_PDFIUM_DIR="$(pwd)/ci/.pdfium" cargo run -p app
```

## Workspace layout

```
crates/
  domain/          Pure domain types (no I/O)
  pdf_engine/      PDFium binding + serialized worker
  packaging/       App-directory policy, PDFium path resolution
  app_services/    Orchestration: file intake, settings, history, platform
  app/             Dioxus Desktop binary
docs/
  design/          ADR, feasibility study, RFC roadmap (read-only reference)
  src/             mdbook source (this documentation)
fixtures/          Minimal hand-built PDFs for testing
ci/                CI helpers (fetch-pdfium.sh)
rfcs/              RFC files following the lifecycle policy
```

## Code style

- Rust 2024 edition, `rustfmt` enforced.
- Files are split at ~300 ELOC; splitting is required above ~500.
- Tests go in `src/tests.rs` (or `src/tests/` for larger test suites).
- All public items should have doc comments.
- English for all code comments and documentation.
