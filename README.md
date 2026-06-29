# PDF Tile Viewer

[![License](https://img.shields.io/github/license/nabbisen/pdf-tile-viewer)](LICENSE)

[日本語](README.ja.md)

A local-first desktop PDF viewer that lays out all pages as a scrollable tile grid — so you can understand a document's structure in seconds instead of flipping through it one page at a time.

<p style="display: flex; gap: 0.8rem; flex-wrap: wrap;">
  <img style="flex: 1; max-width: calc(40.0% - 0.4rem);" src="docs/assets/demo-01.png" alt="demo screenshot 01">
  <img style="flex: 1; max-width: calc(60.0% - 0.4rem);" src="docs/assets/demo-02.png" alt="demo screenshot 02">
</p>

## Why / When

Use PDF Tile Viewer when you need to:

- **Scan a document's structure** without reading it linearly.
- **Compare multiple pages** side by side in a single glance.
- **Find a specific page visually** in a long report, manual, or slide deck.
- **Keep your documents local** — no cloud upload, no account, no telemetry.

## Usage

The Latest executables on multiple platforms are in [Releases](https://github.com/nabbisen/pdf-tile-viewer/releases). No Installation is required. Just launch the executable to start.

### Note: This app is not code-signed.

Your OS (Windows / macOS) may show a security warning.
If you trust this app, you can follow OS-specific steps to allow it.    
Sorry for inconvenience, but we are volunteers and the certificates are expensive.

## Features

![manga-intro](docs/assets/manga-intro.png)

**Woman:** Hmm...  
**Shark:** What's up?  
**Woman:** PDFs are document files, so they always end up long and vertical... I just want to see the whole thing at a glance.  
**Shark:** In that case—try this out!!  
**Woman:** Oh, what’s this?

_(manga by m. thanks)_

- 🟨 Tile layout view on PDF pages
- ✊ Mouse drag move with Ctrl key pushed
- 🔧 Scale / pages-per-row changers
- 🔍 Page zoom view
- 🗺 Text search
- 🍵 Zen mode
- 🗄 Preserve some settings (even when app's quitted)
- 🚪 Files history with links to open again (Kept only while app's running)

## More Detail

Full documentation lives in `docs/` (mdbook format):

- **[New users](docs/src/new-users/):** features, tutorials, FAQ
- **[Intermediate users](docs/src/intermediate/):** settings reference, keyboard shortcuts
- **[Contributors](docs/src/contributors/):** architecture overview, RFC index, local dev guide
