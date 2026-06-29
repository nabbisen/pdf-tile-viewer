# Running Tests

## Default build set

`crates/app` is in `default-members`, so `cargo test` without flags
includes the app crate and requires the WebKit/GTK system packages
(see [Local Development](dev.md#default-build-set-and-the-gui-dependency-tradeoff)).

Use `--exclude app` to run only the library crates on any machine:

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

## Why Dioxus-specific test kinds are not used

The Dioxus 0.7 testing guide describes three additional approaches.
They were evaluated and deliberately not adopted for this project.

**Component testing (dioxus-ssr rsx equality)**
Renders two rsx snippets to HTML strings and compares them. For this app,
components are thin wrappers that wire signals to PDFium-backed services.
An SSR snapshot would mostly verify that the strings written in the source
file are reproduced in the output — tautological and brittle against routine
CSS-class or label changes. The logic worth asserting (layout geometry,
coordinate transforms, search formatting) is already extracted into pure
functions in `domain` and tested directly there.

**Hook testing (manual VirtualDom driving)**
The guide's own example is ~60 lines of `MockProxy` scaffolding per test
suite. This project uses only Dioxus's stock hooks (`use_signal`,
`use_memo`, `use_effect`). Testing those would be testing Dioxus, not
project code. No custom hooks exist that would justify the infrastructure.

**End-to-end testing (Playwright)**
The Dioxus guide targets the **web** renderer. This app is a
**Dioxus Desktop** application backed by a native PDFium library — there
is no browser to drive. The 7 PDFium engine smoke tests already exercise
the real open → render → search path end-to-end at the layer where the
actual integration risk lives.

The current strategy — pure logic in library crates with unit tests,
one genuine integration boundary (PDFium) with smoke tests, a thin GUI
layer that requires little testing — is the right fit for a project this
size. Revisit if a custom hook with non-trivial logic is introduced.
