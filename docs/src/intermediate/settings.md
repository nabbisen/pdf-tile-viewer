# Settings Reference

Settings are stored as JSON in the platform config directory and loaded at
startup. Corrupt or missing files fall back to defaults without blocking the app.

## UI

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `ui.locale` | `string \| null` | `null` (system) | Force locale: `"en"` or `"ja"`. |

## Viewer

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `viewer.default_scale` | `float` | `1.0` | Initial tile scale (0.2–5.0). |
| `viewer.pages_per_row` | object | `{ "mode": "auto" }` | Auto or fixed pages per row. |
| `viewer.show_page_numbers` | `bool` | `false` | Show page numbers on tiles. |
| `viewer.zoom_overlay_scale` | `float` | `2.7` | Zoom overlay render scale. |

## Privacy

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `privacy.show_full_path_in_title` | `bool` | `false` | Show full path in window title. |
| `privacy.persist_recent_files` | `bool` | `false` | Persist recent files across sessions. |

## Window

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `window.width` | `int \| null` | `null` (80% screen) | Saved window width. |
| `window.height` | `int \| null` | `null` (84% screen) | Saved window height. |
