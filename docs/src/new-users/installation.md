# Installation

> Pre-built binaries are not yet available for `2.0.0-rc.1`.
> Building from source is required; a packaging script for Linux is provided.

## Build from source

See [Local Development](../contributors/dev.md) for the full guide.

**Quick start (Linux):**

```sh
# System dependencies
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev libssl-dev pkg-config

# Fetch PDFium (development) and run
bash ci/fetch-pdfium.sh
PDF_TILE_VIEWER_PDFIUM_DIR="$(pwd)/ci/.pdfium" cargo run
```

## Package for Linux (x86-64)

`ci/package-linux.sh` produces a `.tar.gz` artifact with the binary and
bundled PDFium library:

```sh
bash ci/fetch-pdfium.sh          # populate ci/.pdfium/
bash ci/package-linux.sh         # writes dist/pdf-tile-viewer-<ver>-linux-x64.tar.gz
```

The artifact layout:

```
pdf-tile-viewer-<version>-linux-x64/
  bin/pdf-tile-viewer
  resources/pdfium/linux-x86_64/libpdfium.so
  LICENSE
  NOTICE
  CHANGELOG.md
  README.md
  notes.txt
```

Launch from the artifact directory:

```sh
./bin/pdf-tile-viewer
```

## Security note

This release is **unsigned**. On macOS or Windows the OS may show a security
prompt the first time you run the executable. See `NOTICE` for PDFium
provenance and version information.
