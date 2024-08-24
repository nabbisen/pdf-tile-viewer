# Documentation on PDF Tile Viewer

## Development

### System requirements

- [Rust](https://www.rust-lang.org/)
- [Bun](https://bun.sh/)
    - [Node.js](https://nodejs.org/) is also available

### Command-lines

To start up:

```sh
git clone https://github.com/nabbisen/pdf-tile-viewer.git
cd pdf-tile-viewer
bun install
```

To develop:

```sh
bun run tauri dev
```

To build:

```sh
env NO_STRIP=1 bun run tauri build
```

### Powered-by

- [Tauri (v2)](https://v2.tauri.app/) (using [WRY](https://github.com/tauri-apps/wry), cross-platform WebView rendering library), Rust
- [SvelteKit](https://kit.svelte.dev/), [PDF.js](https://mozilla.github.io/pdf.js/), [TypeScript](https://www.typescriptlang.org/), [Vite](https://vitejs.dev/), Bun

### IDE Setup (Optional)

- [VS Codium](https://vscodium.com/) | [VS Code](https://code.visualstudio.com/)
  - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer), [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode), [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
