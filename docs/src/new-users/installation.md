# Installation

> Pre-built binaries are not yet available for `2.0.0-alpha`.
> Building from source is required for now.

## Build from source

See [Local Development](../contributors/dev.md) for the full guide.

**Quick start (Linux):**

```sh
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev libssl-dev
bash ci/fetch-pdfium.sh
PDF_TILE_VIEWER_PDFIUM_DIR="$(pwd)/ci/.pdfium" cargo run -p app
```
