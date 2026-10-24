# Release QA Checklist

A manual smoke test of a **release build** that exercises every documented
shortcut and gesture ([Keyboard Shortcuts](../intermediate/shortcuts.md),
[Navigating the Tile Grid](../new-users/tile-grid.md)) plus the main workflows.
It exists because the 2.0.0 scale shortcuts crashed the application and nothing
in the release process pressed them (RFC 039). Automated tests do not cover the
GUI (see [Running Tests](testing.md)), so this list is the gate for GUI behavior.

Each line is one observable expectation. Tick the box when it holds on this
build; otherwise record the observation. Lines marked **known failing until
RFC NNN** describe the *documented* behavior that is currently wrong; they are
expected to fail until that RFC lands, and the line must be re-checked (and the
marker removed) when it does.

## 0. Setup

- Build: `cargo build --release -p app`. Fetch PDFium with `bash ci/fetch-pdfium.sh`.
- Stage the package layout the release binary expects (production mode ignores
  `PDF_TILE_VIEWER_PDFIUM_DIR`): `bin/pdf-tile-viewer` and
  `resources/pdfium/<platform>/<libpdfium>` side by side (see `ci/package-linux.sh`).
- Run with a **scratch settings directory** so your real settings are untouched
  (Linux: `XDG_CONFIG_HOME=$(mktemp -d)`), and keep the application's stderr in a file.
- Documents: a PDF with at least 100 pages (`crates/pdf_engine/tests/fixtures/fifty-pages-benchmark.pdf`
  is enough for everything except the 100-page counts), `crates/pdf_engine/tests/fixtures/navigation-links-outline.pdf`
  (outline, internal and external links), `crates/pdf_engine/tests/fixtures/password-protected.pdf`
  (user password `pdf-tile-viewer-test`), `crates/pdf_engine/tests/fixtures/not-a-pdf.pdf`.
- Window about 1280×720. Click an empty area of the tile grid before testing
  viewer shortcuts (they need focus in the viewer).

Record: version, commit, platform, date, tester.

## 1. Start and open

- [ ] The dashboard appears without a "PDF engine unavailable" message.
- [ ] **Open PDF…** opens the file chooser; choosing a PDF opens the viewer with tiles.
- [ ] Choosing `not-a-pdf.pdf` shows "This file is not a valid PDF document." and stays on the dashboard.
- [ ] Dropping one `.pdf` on the dashboard opens it (Linux drag/drop: see RFC 027).
- [ ] After **← Back**, the dashboard lists the document under "This session"; clicking it reopens it.

## 2. Scale: shortcuts and gestures (viewer focused, 100-page document)

For each line press or scroll **20 times** and watch the slider; the application
must not close or freeze at any step.

- [ ] `+` ×20: scale rises by 0.2 per press and stops at 5.0 (further presses change nothing).
- [ ] `=` ×20: same result as `+`.
- [ ] `-` ×20: scale falls by 0.2 per press and stops at 0.2.
- [ ] `0`: scale returns to the default from settings (1.0 on a fresh profile).
- [ ] `Ctrl` + scroll up ×20: scale rises by 0.2 per notch (zoom in), stops at 5.0.
- [ ] `Ctrl` + scroll down ×20: scale falls by 0.2 per notch (zoom out), stops at 0.2.
- [ ] Toolbar `−` and `+` buttons each change the scale by 0.2.
- [ ] Dragging the slider changes the scale continuously from 0.2× to 5.0×.
- [ ] After changing the scale, visible tiles end up sharp (not blurry) — **known failing until RFC 035**.
- [ ] The stderr log of this whole section contains no `panicked`.

## 3. Viewer shortcuts

- [ ] `Z` enters zen mode: toolbar and controls are hidden, only the exit `×` remains.
- [ ] `Z` again (or `×`) leaves zen mode.
- [ ] `Escape` in zen mode (nothing else open) leaves zen mode.
- [ ] `Escape` with nothing open returns to the dashboard — **documented today; changes with RFC 033** (Escape will stop leaving the document; update this line then).
- [ ] Typing `z`, `0`, `-`, `+`, `=` in the search box types the characters without zen or scale changes — **known failing until RFC 033**.
- [ ] Same for the jump-to-page input and the columns input — **known failing until RFC 033**.

## 4. Pages per row, page numbers, jump

- [ ] **Auto** adapts the number of columns to the window width (resize the window, columns follow) — **known failing until the viewport-tracking RFC**.
- [ ] The columns toggle can switch to **Fixed** and a number 1–24 sets the column count — **known failing until RFC 033** (unreachable today).
- [ ] The page-number checkbox shows and hides a number under each tile.
- [ ] Jump: typing `40` and pressing Enter (or **Go**) scrolls page 40 into view.
- [ ] Jump: an invalid number shows an inline hint and does not scroll.

## 5. Zoom overlay

- [ ] Clicking a tile opens the overlay on that page; the header shows `Page N / total`.
- [ ] `→` and `PageDown` show the next page; `←` and `PageUp` the previous one.
- [ ] `Home` shows page 1; `End` shows the last page.
- [ ] `+` and `=` raise the overlay zoom by 30 percentage points; `-` lowers it; limits 50% and 800%.
- [ ] The overlay's own `+`/`−` buttons do the same; neither the keys nor the buttons close the application.
- [ ] While the overlay is open, `+` / `-` do not change the tile-grid scale after closing — **known failing until RFC 033**.
- [ ] `Escape` closes the overlay and the viewer stays open — **known failing until RFC 033** (today it also returns to the dashboard).
- [ ] The **Close** button closes the overlay and the viewer stays open.
- [ ] With a search active, highlights appear on the zoomed page at the matching text.
- [ ] Dragging over text selects it; copying yields the selected text.

## 6. Search

- [ ] The 🔍 button opens the search panel.
- [ ] A term of at least two characters + `Enter` (or **Search**) shows a match count and the matching pages.
- [ ] Matching tiles are marked and the text is highlighted in the page image.
- [ ] A term without matches shows "No matches found."
- [ ] **Clear** removes markers and highlights without reloading the document.
- [ ] `Escape` inside the search box closes the panel.

## 7. Outline and links (`navigation-links-outline.pdf`)

- [ ] The ☰ button opens the outline panel; entries are listed.
- [ ] Clicking an entry scrolls the grid to its page.
- [ ] A PDF without an outline shows "No outline".
- [ ] In the zoom overlay, clicking the internal link jumps to page 3; the same link activated with `Enter` also jumps.
- [ ] Clicking the external link opens the **External link** dialog with the raw address.
- [ ] **Copy** in the dialog shows "Copied."; **Cancel** and `Escape` close the dialog and leave the overlay open.

## 8. Password prompt (`password-protected.pdf`)

- [ ] Opening it shows the **Password required** prompt.
- [ ] A wrong password keeps the prompt open and reports that it was not accepted.
- [ ] The correct password opens the document.
- [ ] **Cancel** returns to the unchanged dashboard or viewer.

## 9. Closing

- [ ] Reopening several documents in a row (open, **← Back**, open) works without slowing down and memory does not grow per open (RFC 035: a closed document is released).
- [ ] Memory plateau (RFC 040): on a 100-page document, scroll from the first to the last page and back, twice. Total memory (app plus WebKit processes) after the second pass is within 10 % of the first pass, and well below what holding every page would need. Tiles within two screens of the view never blink; far tiles show the loading placeholder and fill in.
- [ ] Closing the window ends the process; the stderr log contains no `panicked`.
- [ ] Paste the stderr log (or "empty") into the release record.

## When to run

Before every release candidate, on the build that will be published, on each
platform that ships (Linux, Windows, macOS). Add a line here whenever a shortcut,
gesture, or workflow is added or documented; a documented behavior that is not on
this list is untested.
