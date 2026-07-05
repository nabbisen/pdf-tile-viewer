# Features

> **Status:** 2.0.0-beta.8 — latest released beta.

## All pages as a tile grid

Open any local PDF and every page renders as a thumbnail in a scrollable
grid. You can see the whole document at once and navigate without reading
linearly.

- **Scale control**: slider, ± buttons, or Ctrl+scroll wheel — range 0.2×–5.0×.
- **Auto pages-per-row**: the grid adapts to your window width automatically,
  or you can fix the column count manually.
- **Page-number labels**: toggle per-tile page numbers on or off.
- **Jump to page**: type a page number and hit Go or Enter.

## Text search

- Click the 🔍 button in the viewer toolbar to open the search panel.
- Search runs through PDFium's native text index — non-mutating, never
  modifies the PDF file.
- Matched tiles get an accent border and a match-count badge.
- Exact match positions are overlaid as semi-transparent yellow rectangles
  directly over the page image.
- Clear search instantly removes all markers without reloading the document.

## Zoom overlay

- Click any tile to open it in a full-page zoom overlay.
- Navigate previous/next/first/last page inside the overlay.
- Adjust zoom scale independently from the tile grid.
- Search highlights remain visible in the zoomed view.
- Select and copy visible PDF text in the zoomed page when the PDF exposes a
  usable text layer.
- Close with Escape or the × button; focus returns to the tile grid.

## File opening

- **File picker**: click "Open PDF…" on the dashboard.
- **Drag and drop**: drop a single PDF file onto the dashboard window.
- **History**: documents opened during the current session are listed for
  quick re-opening. History is session-only by default (privacy safe).
- **Reveal in file manager**: open the PDF's containing folder from the 📂
  button in the viewer toolbar.

## Zen mode

Press `Z` in the viewer for a distraction-free tile grid with all controls
hidden. Press `Z` again or `Escape` to return to normal mode. A floating ×
button is always visible as an exit affordance.

## Keyboard shortcuts

See the full [Keyboard Shortcuts](../intermediate/shortcuts.md) reference.

## Privacy

No telemetry. No cloud upload. Files never leave your machine. Settings and
session history are stored locally; history does not persist across app
restarts by default.

## Multilingual

The interface is available in English (complete, compiler-enforced) and
Japanese. The locale can be forced in settings or auto-detected from your
system.
