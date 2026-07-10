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
  - `Internal link to page 3` jumps to page 3.
  - `External https link` opens the external-link dialog with target
    `https://example.com/pdf-tile-viewer`.
- Page 2 text: `Navigation page two`
  - Outline target: nested empty-title child under `Chapter 1`.
  - `Blocked file URI` targets `file:///tmp/pdf-tile-viewer-blocked`.
  - `Relative URI` targets `relative/path`.
  - `Blocked launch action` is a disabled launch-action annotation.
- Page 3 text: `Navigation page three`
  - Outline target: `Chapter 2`.
  - `Internal link to page 1` jumps to page 1.
  - `Blocked remote document`, `Blocked embedded document`, and
    `Blocked JavaScript action` are disabled action annotations.

Click or keyboard-activate the visible label areas in the zoom overlay, not the
tile grid.
If the JavaScript annotation is not surfaced as an interactive hit area by the
current PDFium binding, record that as pass as long as no script, dialog,
external app, or other visible action executes.

## Checklist

Record pass/fail and notes for each item.

| Item | Result | Notes |
|------|--------|-------|
| `single-page-basic.pdf` opens normally. | Success | Picker path. |
| Outline button opens the panel for `single-page-basic.pdf`. | Success |  |
| No-outline panel shows `No outline`. | Success |  |
| `navigation-links-outline.pdf` opens normally. | Success |  |
| Outline panel shows `Chapter 1`, nested untitled entry, `Chapter 2`, and `Empty URI`. | Success |  |
| `Chapter 1` scrolls to page 1. | Success |  |
| Nested untitled entry scrolls to page 2. | Success |  |
| `Chapter 2` scrolls to page 3. | Success |  |
| `Empty URI` is disabled or otherwise does not navigate/open anything. | Success |  |
| Nested outline entry expands/collapses with stable indentation. | Success |  |
| Tile-grid search still renders match borders/badges/highlights. | Success |  |
| Zoom-overlay search highlights still render. | Success |  |
| Page 1 internal link in zoom overlay navigates to page 3. | Success | First pass found weak visual affordance; retest passed after underline/link-color styling. |
| Page 3 internal link in zoom overlay navigates to page 1. | Success | First pass found weak visual affordance; retest passed after underline/link-color styling. |
| Selecting visible text in zoom overlay still works on a page with links. | Success |  |
| Dragging over linked text does not accidentally activate a link. | Success |  |
| Page 1 HTTPS link opens the external-link confirmation dialog. | Success |  |
| HTTPS dialog shows the raw target and says no external app will be launched. | Success |  |
| Copy on the HTTPS dialog reports success or clear failure. | Success |  |
| Cancel closes the HTTPS dialog without changing document state. | Success |  |
| Escape closes the HTTPS dialog without reaching the zoom overlay. | Success |  |
| Tab/Shift+Tab stay inside the external-link dialog while it is open. | Success | First pass found weak initial focus styling; retest passed after dialog focus-ring adjustment. |
| Closing the external-link dialog returns focus to the zoom overlay close button. | Success |  |
| Page 2 `Blocked file URI` link shows the dialog as policy-blocked/copy-only and opens no app. | Success | Target is `file:///tmp/pdf-tile-viewer-blocked`. |
| Page 2 `Relative URI` link shows the dialog as policy-blocked/copy-only and opens no app. | Success | Target is `relative/path`. |
| Page 2 launch action does not execute anything. | Success | Visible label was `Blocked launch action`. |
| Page 3 remote document action does not execute anything. | Success | First pass incorrectly navigated to page 1; retest passed after action-priority fix. |
| Page 3 embedded document action does not execute anything. | Success | First pass incorrectly navigated to page 1; retest passed after action-priority fix. |
| Page 3 JavaScript action does not execute anything. | Success | Visible label was `Blocked JavaScript action`. |
| Picker behavior remains unchanged from RFC 025. | Success | Valid PDFs opened through picker; non-`.pdf` file was rejected. |
| Drop behavior is either unchanged from RFC 025 or recorded as Future RFC-F07. | Deferred | Linux WebKitGTK drop failed; tracked as Future RFC-F07. |

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
- Dashboard file drop is deferred to Future RFC-F07 after failing on Linux
  WebKitGTK under both Wayland and `GDK_BACKEND=x11`; picker remains the
  reliable intake path.
```
