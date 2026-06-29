# Running Tests

## Unit tests (no PDFium required)

```sh
cargo test --workspace --exclude app
```

Covers (58 tests total):
- `domain` (24): tile-layout geometry, scale bucketing, coordinate transforms,
  settings clamping/fallback/round-trip, search-result formatting.
- `app_services` (22): file-intake validation ordering, settings
  backup-on-corruption, history dedupe-to-top, engine boot policy,
  render-cache eviction and generation cleanup.
- `packaging` (5): PDFium resolution policy, production vs development mode.
- `pdf_engine` lib (0 — PDFium itself is not available in the unit-test harness).

## Engine smoke tests (PDFium required)

```sh
bash ci/fetch-pdfium.sh     # one-time: downloads chromium/7763 to ci/.pdfium/
PDF_TILE_VIEWER_PDFIUM_DIR="$(pwd)/ci/.pdfium" cargo test -p pdf_engine --test smoke
```

Seven smoke tests exercise the full engine worker in a real process:

| Test | What it checks |
|------|---------------|
| `open_reports_correct_geometry_and_page_count` | US Letter dimensions accurate |
| `render_page_one_produces_valid_png` | PNG signature + correct pixel width |
| `render_with_stale_generation_is_rejected` | Appendix A §8 generation guard |
| `search_finds_expected_pages_without_mutating_file` | Non-mutating search; file unchanged |
| `close_document_invalidates_session` | Render after close returns error |
| `large_document_opens_and_reports_correct_page_count` | 50-page fixture geometry |
| `large_document_search_runs_without_error` | Search across 50 pages |

All 7 pass against PDFium chromium/7763. Without the env var, every test
prints a skip notice and exits 0, so `cargo test` stays green on machines
without PDFium.

## fmt check

```sh
cargo fmt --check
```
