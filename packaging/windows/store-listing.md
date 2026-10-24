# Microsoft Store listing text (English / 日本語)

Texts for Partner Center. Every statement was checked against the 2.0.1 code and
documentation; items that are **not yet true** are marked **HOLD** and must not be
published until the named condition is met. Field limits are those known to the
authors (Description 10,000 chars, Short description 1,000, What's new 1,500,
Features 20 × 200, Search terms 7 × 30); confirm them in Partner Center when pasting.

## 0. Before submitting (checklist)

1. **Disclosure rule (certification 10.2.4.1, denied once):** the first two lines of
   each description disclose the Visual C++ runtime. The runtime is now bundled
   inside the package (no framework dependency), so the text says so. Keep these two
   lines first in both languages.
2. **Privacy policy URL** is required. `PRIVACY.md` must be committed and published
   (GitHub URL after the commit: `https://github.com/nabbisen/pdf-tile-viewer/blob/main/PRIVACY.md`).
   It is English only; a Japanese policy is recommended for the Japanese listing.
3. **Windows test of the actual package** (the 2.0.0 submission was denied for a
   crash on opening a file): install and open `pdf-tile-viewer-2.0.1.0.msix`
   (from the `msix-store.yml` run), open a PDF, search, zoom, press `+` and `-`.
4. **Product name:** the manifest `DisplayName` is `pdf-tile-viewer`; reserve the
   Store names you want to show (`PDF Tile Viewer`, and for Japanese
   `PDF タイルビューア`, the name used by the application's Japanese UI).
5. **Japanese listing languages:** the manifest declares only `en-US`; add
   `<Resource Language="ja-JP" />` (tracked in RFC 038, Slice B′) before declaring
   Japanese support.
6. **HOLD — language claim.** The application chooses its UI language from the
   `LANG` environment variable, which Windows normally does not set, so on Windows
   the UI is English unless `ui.locale` is edited in `settings.json`. Do **not**
   advertise "Japanese user interface" until system-locale detection is fixed
   (RFC 043, in the 2.0.2 train; lift this hold only after the Japanese-Windows check in RFC 043 § 6 passes). The Japanese
   *listing* itself is fine.
7. **WebView2.** The window uses the Microsoft Edge WebView2 Runtime (included in Windows 11, delivered with Edge on Windows 10). Certification flagged only Visual C++ last time; if it asks about WebView2 as well, add one line to both descriptions' first two lines. Test on a Windows 10 machine if you have one.

---

## 1. English (en-US)

### Description

PDF Tile Viewer is a local PDF viewer that shows every page of a document as a scrollable grid of tiles.
The Microsoft Visual C++ runtime libraries it needs are bundled with the app, so nothing else has to be installed.

Most PDF viewers are built for reading one page after another. PDF Tile Viewer is built for seeing the whole document at once, so you can grasp its structure in seconds, compare pages side by side, and find the page you are looking for by sight.

Open a PDF and every page appears as a tile. Make the tiles smaller to see more pages, or larger to read details. Search for text and the matching pages are marked, with the matches highlighted on the page. Open any page in a large zoom view, select and copy its text, and move through the document's outline.

Your documents stay on your computer. PDF Tile Viewer does not upload files, does not need an account, and does not collect telemetry.

Features
• All pages at a glance in a scrollable tile grid
• Adjustable tile size, from a wide overview to a close look
• Text search with matching pages marked and matches highlighted
• Zoom view for a single page, with page-by-page navigation
• Select and copy text in the zoom view (works when the PDF contains text)
• Outline (bookmarks) panel and links to pages inside the same document
• Password-protected PDFs: enter the password when asked (it is never saved)
• Zen mode: hide the controls and see only the tiles
• List of the documents opened in the current session, for quick reopening
• Reveal the open document in File Explorer
• Remembers your tile size and layout settings

Good to know
• One document is open at a time.
• PDF Tile Viewer is a viewer: it does not edit PDFs or add annotations.
• Links to websites or e-mail addresses inside a PDF are never opened automatically; the address is shown and can be copied.
• Documents are processed locally with the PDFium engine.

Open source (Apache-2.0): https://github.com/nabbisen/pdf-tile-viewer

### Short description

See every page of a PDF at once as a tile grid. Local, private, no account.

### What's new (first Store version, 2.0.1)

Welcome to PDF Tile Viewer on the Microsoft Store.
• Every page of your PDF as a scrollable tile grid, with search, a zoom view and an outline panel.
• Fixed in 2.0.1: the app could close unexpectedly when changing the tile size with the + and − keys or Ctrl + mouse wheel.

### Search terms (7)

`pdf viewer`, `pdf reader`, `pdf tile`, `page overview`, `pdf search`, `offline pdf`, `view pdf pages`

### Copyright and trademark info

© nabbisen. Licensed under the Apache License 2.0. PDFium is © The PDFium Authors (BSD-3-Clause).

### Privacy summary (for the listing, below the policy link)

PDF Tile Viewer works entirely on your device. It does not send documents or any other data over the network.

### Notes for certification (English; not shown to customers)

- Package: full-trust desktop application (`runFullTrust`) written in Rust. It uses the native PDFium library, bundled in the package under `resources/pdfium/windows-x86_64/`, and WebView2 for its window. File access is through the standard file picker (Open PDF…).
- Dependencies are bundled inside the package, including the Microsoft Visual C++ runtime DLLs; no framework package dependency is declared.
- Network: the app makes no network connections. Links inside PDFs are never opened; an in-app dialog shows the address for copying.
- To test: launch → Open PDF… → choose any PDF (a password-protected one prompts for a password) → scroll the tiles → press `+`/`−` (tile size) → open the search with the magnifier button and search for a word on a page → click a tile to open the zoom view → close it → Back.
- Changes since the denied 2.0.0 submission: the Visual C++ runtime is now bundled instead of declared as a framework dependency, and the first two lines of the description disclose it.

---

## 2. 日本語 (ja-JP)

### 説明 (Description)

PDF Tile Viewer は、PDF のすべてのページをスクロールできるタイル状のグリッドで一覧表示する、ローカル動作の PDF ビューアーです。
動作に必要な Microsoft Visual C++ ランタイムはアプリに同梱されているため、別途インストールする必要はありません。

多くの PDF ビューアーは、ページを順番に読むことを前提にしています。PDF Tile Viewer は、文書全体を一度に見渡すためのビューアーです。構成を数秒で把握し、ページどうしを並べて比べ、探しているページを見た目で見つけられます。

PDF を開くと、すべてのページがタイルとして並びます。タイルを小さくするとより多くのページを一覧でき、大きくすると細部を確認できます。テキストを検索すると、該当するページに印が付き、ページ上の一致箇所が強調表示されます。任意のページを大きなズーム表示で開き、テキストを選択してコピーしたり、文書のアウトラインをたどったりできます。

文書はお使いのコンピューターの外に出ません。PDF Tile Viewer はファイルをアップロードせず、アカウントも不要で、利用状況データも収集しません。

主な機能
• すべてのページをスクロール可能なタイルグリッドで一覧表示
• タイルの大きさを調整可能（全体の俯瞰から細部の確認まで）
• テキスト検索（該当ページに印、一致箇所を強調表示）
• 1 ページを大きく表示するズーム表示とページ送り
• ズーム表示でのテキスト選択とコピー（PDF にテキストが含まれる場合）
• アウトライン（しおり）パネルと、同じ文書内のページへのリンク
• パスワード付き PDF に対応（パスワードは求められたときに入力し、保存されません）
• Zen モード：操作パネルを隠してタイルだけを表示
• 今回の起動中に開いた文書の一覧から、すばやく開き直し
• 開いている文書を エクスプローラー で表示
• タイルの大きさやレイアウトの設定を記憶

ご留意ください
• 同時に開ける文書は 1 つです。
• 閲覧専用のアプリです。PDF の編集や注釈の追加はできません。
• PDF 内の Web サイトやメールアドレスへのリンクは自動では開きません。アドレスが表示され、コピーできます。
• 文書は PDFium エンジンでお使いの端末内で処理されます。

オープンソース (Apache-2.0)：https://github.com/nabbisen/pdf-tile-viewer

### 簡単な説明 (Short description)

PDF のすべてのページを、タイルグリッドで一度に表示。ローカル動作、プライバシー重視、アカウント不要。

### 新機能 (What's new：Store 初版、2.0.1)

Microsoft Store に PDF Tile Viewer が登場しました。
• PDF のすべてのページをスクロール可能なタイルグリッドで表示。検索、ズーム表示、アウトラインパネルを備えています。
• 2.0.1 での修正：+ キー、− キー、または Ctrl キーを押しながらマウスホイールでタイルの大きさを変えると、アプリが予期せず終了することがある問題を修正しました。

### 検索キーワード (Search terms：7)

`PDF ビューアー`, `PDF リーダー`, `PDF タイル`, `ページ一覧`, `PDF 検索`, `オフライン PDF`, `PDF 閲覧`

### 著作権および商標情報

© nabbisen. Apache License 2.0 の下で提供されています。PDFium © The PDFium Authors（BSD-3-Clause）。

### プライバシーの概要

PDF Tile Viewer はすべての処理をお使いの端末内で行い、文書やその他のデータをネットワーク経由で送信しません。

---

## 3. Statement-to-evidence map (what each claim rests on)

| Claim | Evidence |
|---|---|
| Tile grid, adjustable size | `docs/src/new-users/tile-grid.md`; WP0 run |
| Search with page marks and highlights | `search.md`; WP0 Part B (2 768 matches highlighted) |
| Zoom view, text selection/copy | `features.md`; RFC 023 (best effort; "when the PDF contains text") |
| Outline and internal links; external links copy-only | RFC 026; `outlines-and-links.md` |
| Password-protected PDFs, password not saved | RFC 025; `faq.md` |
| Zen mode, session history, reveal in file manager, saved settings | `features.md`; RFC 009/013 |
| No upload, account or telemetry; no network | `README.md`, `PRIVACY.md`; code has no network client |
| One document at a time, no editing | product non-goals (ADR, RFC 013) |
| VC++ runtime bundled | `msix-store.yml` bundling step; verified in the 2.0.1.0 package (VC DLLs present, no framework dependency) |
| **Not claimed on purpose** | drag and drop (fails on Linux; not tested on Windows), keyboard/screen-reader accessibility (known gaps, RFC 033/037/042), Japanese UI on Windows (HOLD, item 6), rotated-page highlights (RFC 034), column control (Fixed mode currently unreachable) |
