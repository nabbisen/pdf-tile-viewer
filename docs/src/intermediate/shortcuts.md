# Keyboard Shortcuts

Shortcuts fire when the viewer has keyboard focus (click the tile grid area
first; the viewer is focused when a document opens). They never fire while a text
field has focus (search, jump to page, columns, password): typing `z`, `0`, `-`,
`+` or `=` there only types the character. While the zoom overlay or a dialog is
open, only that layer reacts to keys; nothing reaches the tile grid underneath.

## Tile grid

| Key | Action |
|-----|--------|
| `Z` | Toggle zen mode |
| `+` or `=` | Scale up 0.2 step |
| `-` | Scale down 0.2 step |
| `0` | Reset scale to the default from settings |
| `Ctrl + Scroll` | Scale tile grid up / down |

## Zoom overlay

| Key | Action |
|-----|--------|
| `←` / `PageUp` | Previous page |
| `→` / `PageDown` | Next page |
| `Home` | First page |
| `End` | Last page |
| `+` / `=` | Increase overlay zoom scale |
| `-` | Decrease overlay zoom scale |

`Tab` and `Shift + Tab` move between the overlay's controls and stay inside the
overlay. Closing it returns the focus to the viewer.

## Escape closes one layer

One press of `Escape` closes exactly one layer, the top-most one, and never also
leaves the document. The layers, from the top:

| Open layer | `Escape` does |
|------------|---------------|
| External-link dialog or password prompt | Closes it (the zoom overlay or the dashboard stays) |
| Zoom overlay | Closes it |
| Search panel | Closes it and focuses the search button |
| Outline panel | Closes it and focuses the outline button |
| Zen mode | Leaves zen mode |
| Nothing open | Returns to the dashboard (see the note below) |

In zen mode the search and outline panels are not shown, so `Escape` leaves zen
mode first. The last row is the 2.0.x behavior; from 2.1.0 `Escape` with nothing
open will do nothing, and the **Back** button is the only way to return to the
dashboard.

## Text fields

Inside a text field (search, jump to page, columns, password), keys are for typing.
`Enter` runs the field's action (search, go to page, open the document) and
`Escape` leaves the field: it closes the search panel, cancels the password
prompt, or returns the focus to the viewer for the other fields. `Tab` moves to the
next control. The search panel is opened with the 🔍 button in the viewer toolbar and
places the cursor in its field.
