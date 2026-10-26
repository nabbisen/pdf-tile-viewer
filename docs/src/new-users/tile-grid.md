# Navigating the Tile Grid

When a PDF is open, all pages are rendered as a tile grid inside a scrollable
container. You can see the whole document at once and navigate without reading
linearly.

## Scaling

| Method | Action |
|--------|--------|
| Scale slider in the toolbar (`Tile size`) | Drag to any value 0.2×–5.0× |
| ± buttons | Step by 0.2 |
| `Ctrl + Scroll` | Scroll up = zoom in, down = zoom out |
| `+` / `=` key (viewer focused) | Step up |
| `-` key (viewer focused) | Step down |
| `0` key (viewer focused) | Reset to your default scale from settings |

## Pages per row

Under **⚙ More controls**, the **Columns** control is **Auto | Fixed**:

- **Auto** — computes columns from the window width and the tile size when the
  document opens and when the scale changes. It does not re-flow while you
  resize the window yet.
- **Fixed** — switching to Fixed starts at the number of columns shown at that
  moment; use the number field to set any count from 1 to 24. **Auto** goes
  back to the automatic count.

## Page numbers

Toggle the **Page numbers** checkbox (under **⚙ More controls**) to show or hide
a per-tile page number label below each tile image.

## Jump to page

Type a page number in the **Go to page** field (under **⚙ More controls**) and
press Go or Enter. The grid scrolls that tile into view. The field accepts
1-based page numbers (the same numbers printed at the bottom of PDF viewers);
anything else shows one hint: *Enter a page number between 1 and the last page.*

## Pages far from the view

To keep memory use down, rendered pages are kept only near what you are looking
at. Pages far away are released and show a loading placeholder for a moment when
you scroll back (see the [FAQ](faq.md)).

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
