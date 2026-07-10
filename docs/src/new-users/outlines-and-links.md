# Outlines and PDF Links

## Document outline

Some PDFs include an outline, also called bookmarks or a table of contents.
Click the outline button (`☰`) in the viewer toolbar to open the side panel.

- Click an outline entry to scroll the tile grid to its target page.
- Expand or collapse nested entries when the PDF provides hierarchy.
- Entries without a supported target are shown as disabled.
- PDFs without an outline show **No outline**.

The outline panel is hidden in zen mode.

## Internal PDF links

Internal PDF links are available in the zoom overlay. Click a page tile to open
the zoom overlay, then activate a link on the zoomed page to go to the linked
page.

The tile grid does not expose clickable link hit areas. This keeps the grid
optimized for scanning the whole document and keeps text selection scoped to
the single-page zoom view.

## External PDF links

External PDF links are handled conservatively. Activating one opens an
**External link** confirmation dialog inside the app.

- The app does not launch a browser, email client, shell, script, or other
  external application.
- The dialog shows the raw target and lets you copy it.
- Cancel or Escape closes the dialog without changing the document view.
- Unsupported, invalid, or policy-blocked targets are still copyable for
  inspection, but are not opened by the app.

This behavior keeps PDF links inspectable while preserving the local-first,
no-surprises security model.
