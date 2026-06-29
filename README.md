# PDF Tile Viewer

PDF viewer/reader displayed in tile layout. Number of pages in a row is dynamically calculated with app window size and zoom scale modified with Ctrl key + mouse wheel. Customization is also available.

タイル形式の PDF ビュアー / リーダー です。行あたりページ数を、アプリ Window サイズと、Ctrl + マウスホイールで変更可能なズーム倍率から、動的計算します。手動設定も可能です。

<p style="display: flex; gap: 0.8rem; flex-wrap: wrap;">
  <img style="flex: 1; max-width: calc(40.0% - 0.4rem);" src="docs/assets/demo-01.png" alt="demo screenshot 01">
  <img style="flex: 1; max-width: calc(60.0% - 0.4rem);" src="docs/assets/demo-02.png" alt="demo screenshot 02">
</p>

## Usage

The Latest executables on multiple platforms are in [Releases](https://github.com/nabbisen/pdf-tile-viewer/releases). No Installation is required. Just launch the executable to start.

[Releases](https://github.com/nabbisen/pdf-tile-viewer/releases) ページ内に最新の実行ファイルがあります。インストールは不要です。実行ファイルを起動するだけで使えます。

### Note: This app is not code-signed. | アプリにはコード署名がありません。

Your OS (Windows / macOS) may show a security warning.
If you trust this app, you can follow OS-specific steps to allow it.    
Sorry for inconvenience, but we are volunteers and the certificates are expensive.

お使いの OS (Windows / macOS) によっては、セキュリティ警告が表示される場合があります。
このアプリを信頼できる場合は、OS 固有の手順に従って許可してください。    
不便であり恐縮ですが、私たちはボランティアであり、証明書は高額なのです。

## Features

![manga-intro](docs/assets/manga-intro.png)

**Woman:** Hmm...  
**Shark:** What's up?  
**Woman:** PDFs are document files, so they always end up long and vertical... I just want to see the whole thing at a glance.  
**Shark:** In that case—try this out!!  
**Woman:** Oh, what’s this?

_(manga by m. thanks)_

- 🟨 Tile layout view on PDF pages | PDF ページのタイルレイアウト表示
- ✊ Mouse drag move with Ctrl key pushed | Ctrl キーを押しながらマウスドラッグして移動
- 🔧 Scale / pages-per-row changers | 倍率 / 行あたりページ が変更可能
- 🔍 Page zoom view | ページのズーム表示
- 🗺 Text search | テキスト検索
- 🍵 Zen mode | 禅モード
- 🗄 Preserve some settings (even when app's quitted) | 一部設定の保存 (アプリ再起動後も有効)
- 🚪 Files history with links to open again (Kept only while app's running) | 再表示用リンク付きファイル履歴 (アプリ実行中のみ有効)
