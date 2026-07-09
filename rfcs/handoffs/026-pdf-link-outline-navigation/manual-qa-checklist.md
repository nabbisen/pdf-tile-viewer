# RFC 026 Manual QA Checklist

This checklist covers the remaining manual WebView gate for RFC 026. It is not
release prep and is not a release point.

## Setup

Launch the app from the project root:

```sh
PDF_TILE_VIEWER_PDFIUM_DIR="$(pwd)/ci/.pdfium" cargo run
```

Then open `fixtures/navigation-links-outline.pdf` through the picker or drop it
on the dashboard.

Also open `fixtures/single-page-basic.pdf` for the no-outline check.

During RFC 026 Linux WebKitGTK QA, picker open worked but dashboard file drop
failed under both the observed Wayland session and an `GDK_BACKEND=x11`
comparison run. Treat picker as the reliable intake path for RFC 026. Track
drag/drop reliability separately as Future RFC-F07.

## Fixture Map

`fixtures/navigation-links-outline.pdf` has three pages:

- Page 1 text: `Navigation page one`
  - Outline target: `Chapter 1`
  - Internal link rectangle: left side, below the page title, jumps to page 3.
  - External link rectangle: left side, below the internal link, target
    `https://example.com/pdf-tile-viewer`.
- Page 2 text: `Navigation page two`
  - Outline target: nested empty-title child under `Chapter 1`.
  - External link rectangles: `file:///tmp/pdf-tile-viewer-blocked` and
    `relative/path`.
  - Disabled launch-action rectangle below those URI rectangles.
- Page 3 text: `Navigation page three`
  - Outline target: `Chapter 2`.
  - Internal link rectangle: left side, below the page title, jumps to page 1.
  - Disabled remote, embedded, and JavaScript action rectangles below it.

The rectangles are PDF annotations and may not have visible text labels. Click
or keyboard-activate the link hit areas in the zoom overlay, not the tile grid.
If the JavaScript annotation is not surfaced as an interactive hit area by the
current PDFium binding, record that as pass as long as no script, dialog,
external app, or other visible action executes.

## Checklist

Record pass/fail and notes for each item.

| Item | Result | Notes |
|------|--------|-------|
| `single-page-basic.pdf` opens normally. |  |  |
| Outline button opens the panel for `single-page-basic.pdf`. |  |  |
| No-outline panel shows `No outline`. |  |  |
| `navigation-links-outline.pdf` opens normally. |  |  |
| Outline panel shows `Chapter 1`, nested untitled entry, `Chapter 2`, and `Empty URI`. |  |  |
| `Chapter 1` scrolls to page 1. |  |  |
| Nested untitled entry scrolls to page 2. |  |  |
| `Chapter 2` scrolls to page 3. |  |  |
| `Empty URI` is disabled or otherwise does not navigate/open anything. |  |  |
| Nested outline entry expands/collapses with stable indentation. |  |  |
| Tile-grid search still renders match borders/badges/highlights. |  |  |
| Zoom-overlay search highlights still render. |  |  |
| Page 1 internal link in zoom overlay navigates to page 3. |  |  |
| Page 3 internal link in zoom overlay navigates to page 1. |  |  |
| Selecting visible text in zoom overlay still works on a page with links. |  |  |
| Dragging over linked text does not accidentally activate a link. |  |  |
| Page 1 HTTPS link opens the external-link confirmation dialog. |  |  |
| HTTPS dialog shows the raw target and says no external app will be launched. |  |  |
| Copy on the HTTPS dialog reports success or clear failure. |  |  |
| Cancel closes the HTTPS dialog without changing document state. |  |  |
| Escape closes the HTTPS dialog without reaching the zoom overlay. |  |  |
| Tab/Shift+Tab stay inside the external-link dialog while it is open. |  |  |
| Closing the external-link dialog returns focus to the zoom overlay close button. |  |  |
| Page 2 `file:` link shows the dialog as policy-blocked/copy-only and opens no app. |  |  |
| Page 2 relative link shows the dialog as policy-blocked/copy-only and opens no app. |  |  |
| Page 2 launch action does not execute anything. |  |  |
| Page 3 remote document action does not execute anything. |  |  |
| Page 3 embedded document action does not execute anything. |  |  |
| Page 3 JavaScript action does not execute anything. |  |  |
| Picker behavior remains unchanged from RFC 025. |  |  |
| Drop behavior is either unchanged from RFC 025 or recorded as Future RFC-F07. |  |  |

## Acceptance Note Template

Copy this into the final RFC 026 acceptance notes when complete:

```text
Manual WebView QA for RFC 026 completed on <OS / desktop / WebView version>.

Fixture files:
- fixtures/navigation-links-outline.pdf
- fixtures/single-page-basic.pdf

Result:
- Outline/no-outline behavior: pass/fail
- Internal link navigation: pass/fail
- Zoom text selection over linked pages: pass/fail
- Search highlight preservation: pass/fail
- External URI copy-only dialog: pass/fail
- Disabled Launch/GoToR/GoToE/JavaScript actions: pass/fail
- Picker regression check: pass/fail
- Drop reliability: pass/fail/deferred to Future RFC-F07

Notes:
- <any deviations, blocked checks, or follow-up issues>
```
