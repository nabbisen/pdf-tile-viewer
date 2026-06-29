# Running Tests

## Unit tests (no PDFium required)

```sh
cargo test --workspace --exclude app
```

Tests cover RFC specifications: tile-layout geometry, settings clamping,
search-result formatting, file intake validation, history deduplication, etc.

## Engine smoke tests (PDFium required)

```sh
bash ci/fetch-pdfium.sh     # one-time: fetches chromium/7763 to ci/.pdfium/
PDF_TILE_VIEWER_PDFIUM_DIR="$(pwd)/ci/.pdfium" cargo test -p pdf_engine --test smoke
```

Smoke tests verify: open → geometry check, render page 1 → PNG signature,
stale-generation rejection, search finds correct pages without mutating the
file, close invalidates session.

## fmt check

```sh
cargo fmt --check
```
