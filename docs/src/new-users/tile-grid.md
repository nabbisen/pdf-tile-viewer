# Navigating the Tile Grid

When a PDF is open, all pages are rendered as a tile grid inside a scrollable
container. You can see the whole document at once and navigate without reading
linearly.

## Scaling

| Method | Action |
|--------|--------|
| Scale slider in the toolbar | Drag to any value 0.2×–5.0× |
| ± buttons | Step by 0.2 |
| `Ctrl + Scroll` | Scroll up = zoom in, down = zoom out |
| `+` / `=` key (viewer focused) | Step up |
| `-` key (viewer focused) | Step down |
| `0` key (viewer focused) | Reset to your default scale from settings |

## Pages per row

The toolbar has an **Auto / Fixed** toggle:

- **Auto** — computes columns from the window width and current tile size.
- **Fixed** — use the number input to set a fixed column count (1–24).

## Page numbers

Toggle the checkbox (`#`) in the toolbar to show or hide a per-tile page
number label below each tile image.

## Jump to page

Type a page number in the `p.` input and press Go or Enter. The grid
scrolls that tile into view. The input accepts 1-based page numbers
(the same numbers printed at the bottom of PDF viewers).

## Zoom overlay

Click any tile to open it in a larger zoom overlay. See
[features](features.md#zoom-overlay) for details.

## Outlines and PDF links

The viewer toolbar includes an outline button (`☰`). PDFs without a document
outline show **No outline**. Outline entries, when present, scroll the tile
grid to their target pages.

PDF link hit areas are intentionally exposed in the zoom overlay rather than
the tile grid. See [Outlines and PDF Links](outlines-and-links.md) for details.

## Keyboard navigation

See [Keyboard Shortcuts](../intermediate/shortcuts.md).
