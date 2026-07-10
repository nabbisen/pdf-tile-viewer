# Settings Reference

Settings are stored as JSON in the platform config directory and loaded at
startup. Corrupt or missing files fall back to defaults without blocking the
app. Out-of-range values are clamped silently.

**Location:** `<config_dir>/pdf-tile-viewer/settings.json`
- Linux: `~/.config/pdf-tile-viewer/settings.json`
- macOS: `~/Library/Application Support/pdf-tile-viewer/settings.json`
- Windows: `%APPDATA%\pdf-tile-viewer\settings.json`

---

## UI

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `ui.locale` | `string \| null` | `null` (system) | Force locale: `"en"` or `"ja"`. `null` follows the `LANG` environment variable. |

---

## Viewer

These settings are saved automatically when you change the corresponding
controls in the viewer toolbar.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `viewer.default_scale` | `float` | `1.0` | Tile scale when a document opens (0.2–5.0). |
| `viewer.pages_per_row` | object | `{ "mode": "auto" }` | `{ "mode": "auto" }` or `{ "mode": "fixed", "value": N }`. |
| `viewer.show_page_numbers` | `bool` | `false` | Show per-tile page number labels. |
| `viewer.zoom_overlay_scale` | `float` | `2.7` | Render scale for the zoom overlay (0.2–8.0). |
| `viewer.zoom_overlay_alpha` | `float \| null` | `null` | Overlay background transparency (0.0–1.0). `null` = not set. |

---

## Privacy

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `privacy.show_full_path_in_title` | `bool` | `false` | When `true`, the window title shows the full file path instead of just the filename. |
| `privacy.persist_recent_files` | `bool` | `false` | *(Schema only — session history is always ephemeral in 2.0.0.)* |
| `privacy.include_full_paths_in_diagnostics` | `bool` | `false` | *(Schema only — not yet used by diagnostics.)* |

PDF passwords are not settings. They are never written to `settings.json` and
must be entered again when needed.

---

## Advanced

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `advanced.render_cache_budget_mb` | `int \| null` | `null` (256 MB) | In-memory render cache budget. Minimum enforced at 64 MB. |
| `advanced.diagnostics_enabled` | `bool` | `false` | *(Schema only — diagnostics panel not yet implemented.)* |

---

## Window

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `window.width` | `int \| null` | `null` | Saved on document open (measured from viewport); restored at next launch. |
| `window.height` | `int \| null` | `null` | Saved on document open; restored at next launch. |
| `window.maximized` | `bool` | `false` | *(Schema only — maximized state not yet tracked.)* |

> **Note on "Schema only" fields:** these keys exist in the settings schema
> (and are preserved in your `settings.json` if written by hand), but the
> current release does not read or write them yet. They are reserved for
> future milestones without a breaking settings-file change.
