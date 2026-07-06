# FAQ

**Does the app send my PDFs anywhere?**

No. PDF Tile Viewer is strictly local-first. Your files are read from disk
and never leave your machine.

**Can it open password-protected PDFs?**

Not yet. The engine detects encrypted PDFs and shows a clear error message,
but decryption is not supported in 2.0.0-beta.9.

**Can the title bar show the full file path?**

Privacy is safe by default — the window title uses the filename only.
If `privacy.show_full_path_in_title` is set to `true` in the settings file,
the title shows the full local file path. Full paths can reveal usernames or
folder names, so this is opt-in.

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

Official release archives bundle the platform PDFium library under
`resources/pdfium/<platform>/`, so packaged users do not install PDFium
separately. Extract the full archive and keep `bin/` and `resources/`
together in the same directory.

Source builds use the binary fetched by `ci/fetch-pdfium.sh` (currently
chromium/7920) or an explicit `PDF_TILE_VIEWER_PDFIUM_DIR`.

**What should I do if the app says the PDF engine is unavailable?**

For an official release archive, download the archive again, extract it into a
directory, and launch the app without moving `bin/` away from `resources/`.
For source builds, see the contributor development guide for PDFium setup.

**Is the application code-signed?**

No. 2.0.0-beta.9 is unsigned. On macOS or Windows you may see an OS security
prompt the first time you run it.
