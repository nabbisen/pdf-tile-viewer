# Handoff — Evidence Run (WP0): preconditions for RFC 033 and verification input for RFC 034/035 and the render-pipeline RFC

Governing RFC: [`../../accepted/033-keyboard-routing-and-dismissal-model.md`](../../accepted/033-keyboard-routing-and-dismissal-model.md) § 9 ("the evidence run must confirm or withdraw the zoom-overlay propagation and disabled-input findings before the handoff is issued").
Part B (measurements) also feeds the future render-pipeline RFC (RFC-FUTURE F14); when that RFC receives its number this file's Part B is linked from it.

Schedule: first dev task, Mon 10-26 → Wed 10-28 (may start earlier if the owner hands over the tree). Working model: directly on `main`, no branches, you are the only actor on the tree; start from a clean `git status` at the hand-over commit; hand over with the `main` commit hash. **No changes to product code, RFCs, docs, CI, tags, or releases (DEC-001).**

Background: the architect onboarding review found suspected defects by reading code. This task turns suspicions into evidence and measures large-document behavior. Do not fix anything; report.

Environment: Linux (niri). Resize the window to 1280×720 first. Screenshots: `niri msg action screenshot-window --id <id> --path <relative-path-from-project-root>` (not ImageMagick `import`). Focus the window before typing with `xdotool`. Debug build with `PDF_TILE_VIEWER_PDFIUM_DIR=$(pwd)/ci/.pdfium`.

---

## Part A — Manual reproductions
Record each as **Confirmed / Not reproduced / Differs**, with a screenshot or exact observation. Use `fixtures/multi-page-search.pdf` and `fixtures/navigation-links-outline.pdf` unless stated.

1. **A1 (F-02)** Click the search box and type `z`. Expected by spec: text appears and nothing else happens. Suspected: Zen mode toggles and the search panel disappears. Repeat with `0`, `-`, `+`, `=` in the search box, in the jump-to-page input (⚙ panel), and in the columns number input. Record which keys change scale or Zen.
2. **A2 (F-03)** Click a tile to open the zoom overlay. Press `+` / `-`: does the tile-grid scale also change after closing? Press `Escape` with no search/outline panel open and Zen off: does the overlay close **and** the app return to the dashboard? Repeat with focus on the overlay close button and with focus elsewhere (click the page image first).
3. **A3 (F-04)** At scale 1.0 let tiles render, then raise the scale to 3.0 with the slider. Are visible tiles sharp or still low resolution? Wait 10 s. Scroll away and back.
4. **A4 (F-08)** Open ⚙ → columns. Starting in Auto, can you reach Fixed mode with the mouse? With the keyboard?
5. **A5 (F-01)** Create rotated copies of `fixtures/single-page-basic.pdf` (scratch files outside the repo) by inserting `/Rotate 90` (also 180, 270) into the page dictionary (PDFium repairs the xref). Open each, search for `Hello`: record tile shape versus page image and highlight position, in tile view and zoom overlay. Select text in the zoom overlay.
6. **A6 (F-05)** Open a PDF, return to the dashboard, open it again; repeat 10 times. Record process RSS (`VmRSS` in `/proc/<pid>/status`) after each cycle.
7. **A7 (F-07)** Open a document, then resize/maximize the window. Does the column count follow? Toggle the outline panel.
8. **A8** Search for `0` in the search box and note any other surprising behavior.

## Part B — Measured stress run (closes risk R-1)
1. Generate deterministic synthetic PDFs (a scratch script outside the repo, no third-party libraries): 100, 500 and 1000 pages (text-heavy) and a 100-page image-heavy variant with a deterministic embedded raster.
2. The owner supplies a real-world corpus path separately; if absent, mark those rows `pending (owner)`.
3. For each document, with a release build (`cargo build --release -p app`), measure: time to first visible tile; RSS after open, after a steady top-to-bottom scroll, after scrolling back to the top; RSS after changing scale 1.0 → 3.0 → 1.0; time to render the zoom overlay at 2.7× and 8.0×; behavior of a search for a common word; any UI freeze longer than 200 ms (observed).
4. Record the machine (CPU, RAM, GPU/compositor) and the exact commit.

## Deliverable
A review-request package `.git-exclude/review-request/wp0-evidence-run/README.md` (single entry point; tell the owner only this path, relative to the project root, at the end) containing the Part A results table, the Part B measurements table, screenshots under the same directory, the scripts used, and anything surprising. Include the standard review-request fields (summary, executed commands, results, known limitations).

## Prohibited
Changes to `crates/`, `rfcs/`, `docs/`, CI, tags, releases; committing scratch files; guessing — write "not run" when something was not run.
