# Searching Text

Click the 🔍 button in the viewer header (or press the button — no keyboard
shortcut is assigned to open it yet) to open the search panel.

## Running a search

Type at least two characters and press **Search** or **Enter**. The engine
runs a PDFium-backed non-mutating text search:

- Matched tiles receive an accent-coloured border and a match-count badge.
- Exact match rectangles are highlighted as semi-transparent yellow overlays
  over the page image.
- A summary shows total match count and the matched pages in compact range
  notation (e.g. `1, 4–6, 10`).

## Clearing search

Click **Clear** to remove all markers. The document is not reloaded;
the highlight state is app-side only.

## Notes

- The search is case-insensitive by default (PDFium's default options).
- Search never modifies the PDF file on disk (RFC 010/011 non-mutating rule).
- Highlights in the zoom overlay are aligned using coordinate transforms from
  PDF point space to rendered pixel space (RFC 011).
- The minimum query length is 2 characters to avoid expensive accidental searches.
