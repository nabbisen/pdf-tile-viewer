# FAQ

**Does the app send my PDFs anywhere?**

No. PDF Tile Viewer is strictly local-first. Your files are read from disk
and never leave your machine. See the [Privacy Policy](https://github.com/nabbisen/pdf-tile-viewer/blob/main/PRIVACY.md).

**Can it open password-protected PDFs?**

Yes. When a PDF requires a password, the app prompts for it and opens the
document if the password is accepted.

Passwords are used only for that open attempt. They are not saved to settings,
session history, logs, or diagnostics, and they must be entered again in a
future app session.

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

No. 2.0.2 is unsigned. On macOS or Windows you may see an OS security
prompt the first time you run it.

**Why do pages far from the view show a loading placeholder?**

To keep memory use low, the viewer keeps rendered pages only near what you are
looking at (about two screen heights above and below). Pages further away are
released, and they show the loading placeholder for a moment when you scroll
back to them, then fill in again. Pages close to the view are never affected.
This lowers the memory the application itself uses while you scroll through a long
document. The web view that displays the pages may still hold more memory than
needed; improving that is planned.

**Why is the interface in English, and how do I change the language?**

By default the app follows your system: the `LANG` environment variable when it names a real
locale, otherwise the operating system's language. Japanese and English are available; any other
system language shows English. To choose a language yourself, set `ui.locale` to `"en"` or `"ja"` in
the [settings file](../intermediate/settings.md) and restart the app.
