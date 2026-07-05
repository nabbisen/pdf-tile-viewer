# FAQ

**Does the app send my PDFs anywhere?**

No. PDF Tile Viewer is strictly local-first. Your files are read from disk
and never leave your machine.

**Can it open password-protected PDFs?**

Not yet. The engine detects encrypted PDFs and shows a clear error message,
but decryption is not supported in 2.0.0-beta.8.

**Why does the title bar show only the filename, not the full path?**

Privacy is safe by default — the window title uses the filename only.
A `privacy.show_full_path_in_title` setting exists in the settings schema but
is not yet wired to the title bar; it is reserved for a future release.

**Where are settings stored?**

In the platform config directory:
- **Linux:** `~/.config/pdf-tile-viewer/settings.json`
- **macOS:** `~/Library/Application Support/pdf-tile-viewer/settings.json`
- **Windows:** `%APPDATA%\pdf-tile-viewer\settings.json`

The file is created on the first settings change (e.g. adjusting tile scale).
If the file is corrupt on startup, it is backed up as `settings.json.bak` and
the app starts with defaults.

**How is the session history stored?**

It is not stored on disk. The list of recently opened files shown on the
dashboard is in-memory for the current session only and is cleared when
the app closes. Persistent history is a planned future feature.

**What PDFium version is used?**

Development builds use the binary fetched by `ci/fetch-pdfium.sh`
(currently chromium/7920). Release packaging will bundle the library with
the application so you do not need to install it separately.

**Is the application code-signed?**

No. 2.0.0-beta.8 is unsigned. On macOS or Windows you may see an OS security
prompt the first time you run it.
