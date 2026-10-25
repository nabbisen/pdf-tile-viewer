> **Historical record — executed 2026-10-22** (results: `.git-exclude/review-request/wp0-evidence-run/`, unpublished). Fixture paths below use the old `fixtures/` location; since RFC 038 B′ they live in `crates/pdf_engine/tests/fixtures/`. Re-use of this protocol (RFC 033 and 037 handoffs) must use the new path.

# Handoff — Evidence Run (WP0): preconditions for RFC 033 and verification input for RFC 034/035 and the render-pipeline RFC

Governing RFC: [`../../accepted/033-keyboard-routing-and-dismissal-model.md`](../../accepted/033-keyboard-routing-and-dismissal-model.md) § 9 ("the evidence run must confirm or withdraw the zoom-overlay propagation and disabled-input findings before the handoff is issued").
Part B (measurements) also feeds the future render-pipeline RFC (RFC-FUTURE F14); when that RFC receives its number this file's Part B is linked from it.

Schedule: first dev task, **4 working days** (Parts A, B, C; was 3 before Part C), planned Mon 10-26 → Thu 10-29; it may start earlier if the owner hands over the tree, which absorbs the extra day. Working model: directly on `main`, no branches, you are the only actor on the tree; start from a clean `git status` at the hand-over commit; hand over with the `main` commit hash. **No changes to product code, RFCs, docs, CI, tags, or releases (DEC-001).**

Background: the architect onboarding review found suspected defects by reading code. This task turns suspicions into evidence and measures large-document behavior. Do not fix anything; report.

Environment: Linux (niri). Window 1280×720 obtained at launch (the isolated `settings.json`), never by resizing a running window. Screenshots: `niri msg action screenshot-window --id <id> --path <absolute-path-under-project-root>` (niri rejects relative paths; not ImageMagick `import`). Keys: `wtype` on the Wayland backend (xdotool keys do not reach a native Wayland window); on the X11 backend `xdotool` keys were observed to reach the app; pointer actions need the X11 backend. Current rules: `.git-exclude/rules/project-instructions-gui-common.md`. Debug build with `PDF_TILE_VIEWER_PDFIUM_DIR=$(pwd)/ci/.pdfium`.

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

## Part C — Accessibility evidence (screen reader and keyboard operability)

Why: the owner asked whether screen readers work on our app after an accessibility deficit was found in another WebKitGTK-based project (a Flatpak whose WebKit sandbox/app-id setup hid the accessibility tree). We do not ship a Flatpak (Linux release is a tarball), so that exact cause is out of scope (**C10**), but our own accessibility must be measured. RFC 013 § 9–§ 10 and RFC 037 § 5.4 state requirements; this part checks them with evidence. **Observe and report; do not fix.**

### Safety rules for input tools (mandatory)
- `wtype` sends keys to **whatever window has focus** (on the Wayland backend `xdotool` keys do not reach this app, probed 2026-10-25; on X11 they do). The desktop also holds other terminals and sessions. **Immediately before every `wtype` call** run `niri msg -j focused-window` and confirm its `pid` is the instance under test (on X11 windows niri reports the XWayland bridge's pid: compare the window `id` instead) (or its file-chooser dialog) or its file-chooser dialog; if not, focus it with `niri msg action focus-window --id <id>` and re-check. Never type while any other window is focused.
- Prefer AT-SPI actions (`atspi.py press|focus`) over synthesized keys wherever possible; use `wtype` only for things that need real key events (shortcut and Tab-order tests).
- Close the app after each run. Never leave a test window open between sessions.

### Tooling
Save the script below **outside the repository** (e.g. in a scratch directory) and run it as `python3 -W ignore atspi.py …`. It needs `python3` with `gi`/`Atspi 2.0`, the AT-SPI bus, `wtype`, `niri` (all present on this machine as of 2026-10-22).

```python
#!/usr/bin/env python3
"""AT-SPI helper for accessibility evidence runs.

usage (run with `python3 -W ignore`):
  atspi.py dump  <app-name-substring> [max-depth]         print the accessibility tree (read-only)
  atspi.py press <app-name-substring> <role> <name>       invoke the first action (e.g. a button)
  atspi.py focus <app-name-substring> <role> <name>       move keyboard focus (and print entry text)
Roles/names are the AT-SPI values printed by `dump` (e.g. button 'Open PDF…').
"""
import sys
import gi
gi.require_version('Atspi', '2.0')
from gi.repository import Atspi

STATES = ((Atspi.StateType.FOCUSABLE, 'focusable'), (Atspi.StateType.FOCUSED, 'FOCUSED'),
          (Atspi.StateType.SHOWING, 'showing'), (Atspi.StateType.EDITABLE, 'editable'),
          (Atspi.StateType.MODAL, 'MODAL'), (Atspi.StateType.EXPANDABLE, 'expandable'),
          (Atspi.StateType.ENABLED, 'enabled'))


def app_named(sub):
    desk = Atspi.get_desktop(0)
    for i in range(desk.get_child_count()):
        app = desk.get_child_at_index(i)
        if app and sub in (app.get_name() or ''):
            return app
    sys.exit('app not found: %s' % sub)


def dump(acc, depth, maxdepth):
    try:
        role, name, n = acc.get_role_name(), acc.get_name() or '', acc.get_child_count()
    except Exception:
        return
    ss = acc.get_state_set()
    st = ','.join(label for s, label in STATES if ss.contains(s))
    acts = ''
    a = acc.get_action_iface()
    if a and a.get_n_actions():
        acts = ' actions=' + '|'.join(a.get_action_name(i) for i in range(a.get_n_actions()))
    print('%s%s name=%r [%s]%s' % ('  ' * depth, role, name, st, acts))
    if depth < maxdepth:
        for i in range(n):
            c = acc.get_child_at_index(i)
            if c:
                dump(c, depth + 1, maxdepth)


def find(acc, role, name, depth=0):
    if depth > 40:
        return None
    try:
        if acc.get_role_name() == role and (acc.get_name() or '') == name:
            return acc
        for i in range(acc.get_child_count()):
            c = acc.get_child_at_index(i)
            r = find(c, role, name, depth + 1) if c else None
            if r:
                return r
    except Exception:
        pass
    return None


cmd, sub = sys.argv[1], sys.argv[2]
app = app_named(sub)
if cmd == 'dump':
    dump(app, 0, int(sys.argv[3]) if len(sys.argv) > 3 else 20)
elif cmd in ('press', 'focus'):
    el = find(app, sys.argv[3], sys.argv[4])
    if not el:
        sys.exit('element not found: %s %r' % (sys.argv[3], sys.argv[4]))
    if cmd == 'press':
        a = el.get_action_iface()
        print('doing', a.get_action_name(0))
        a.do_action(0)
    else:
        print('grab_focus', el.get_component_iface().grab_focus())
        t = el.get_text_iface()
        if t:
            print('text:', repr(t.get_text(0, t.get_character_count())))
else:
    sys.exit(__doc__)
```

Opening a document without a mouse: run `atspi.py press pdf-tile-viewer button 'Open PDF…'` → the portal file chooser opens (a separate window). With the chooser focused: `wtype -M ctrl l -m ctrl`, then `wtype "<absolute path>"`, then `wtype -k Return` (selects the file), then `wtype -k Return` again (accepts). Check the focused window before each call. Pointer actions (clicking a tile) are not possible natively on Wayland; for those steps run the app under XWayland (`GDK_BACKEND=x11`, `DISPLAY` already set by `xwayland-satellite`) and use `xdotool`, and record that the run used X11.

### Architect's preliminary observations (2026-10-22, native Wayland, debug build) — **unverified; reproduce, do not trust**
| # | Observation |
|---|---|
| O1 | Dashboard tree: a landmark; headings `PDF Tile Viewer`, `Open a PDF to get started`, `This session`; `button 'Open PDF…'` with action `press`. |
| O2 | The web document's accessible name is `Dioxus app`, while the window frame is named correctly (`<file> — PDF Tile Viewer`). |
| O3 | Viewer toolbar buttons `🔍`, `📂`, `⊞` are named by the emoji glyph itself; `← Back`, `Outline`, `Zoom out`, `Zoom in`, `More controls` have names. |
| O4 | The scale slider has an empty name. |
| O5 | Page tiles are exposed as `image 'Page N'` with no actions and not focusable. |
| O6 | Pressing `🔍` opened the search panel (`entry 'Find text…'`, `Search`, `Clear`, `Close search`) but keyboard focus stayed on the `🔍` button. |
| O7 | With focus on the `🔍` button, typing `z` toggled Zen mode (toolbar, search panel and controls disappeared; only `×` remained). |
| O8 | The debug build shows a menu bar `Window / Edit / Help` including `Toggle Developer Tools`. Release behavior unknown. |
| O9 | **Not completed:** typing `a`, `z`, `0`, `-` with focus *inside* the search entry was interrupted; the result is unknown. |
| O10 | File chooser recipe above (Ctrl+L, full path, Return selects, Return accepts) worked. |

### Tasks (record each as Reproduced / Not reproduced / Differs / Not run, with the exact output)
- **C1** Reproduce O1–O8 and O10. Complete **O9** for three inputs: search entry, jump-to-page input (⚙ panel), columns number input — record the resulting input value, Zen state, and scale after typing `a`, `z`, `0`, `-`, `+`, `=`.
- **C2** Tree dumps (`atspi.py dump`, depth ≥ 20) for these states, saved as text files in the review package: dashboard; dashboard after a failed open (`fixtures/not-a-pdf.pdf`); viewer; viewer with search open (before and after a search with results and with no results); outline open (`navigation-links-outline.pdf`); ⚙ controls open; Zen mode; zoom overlay; password prompt (`fixtures/password-protected.pdf`); external-link dialog. Extend `dump` to also print each node's `get_attributes()` entries `live`, `container-live`, `lang`, `xml-roles`.
- **C3** From the dumps, produce three lists: (a) interactive elements with an empty or glyph-only name; (b) elements with a click handler in code but not focusable or without an action (start with page tiles, history entries); (c) landmarks and heading structure per screen.
- **C4** Focus trace: for each open/close of search, outline, ⚙ panel, zoom overlay, password prompt, external-link dialog, Zen — which node has `FOCUSED` afterwards? Compare with RFC 013 § 10 and write pass/fail per rule.
- **C5** Keyboard-only walk (`wtype -k Tab`, `Shift+Tab`, `Return`, `space`, `Escape`, arrows): can each function be reached and operated without a pointer — open a document (picker, history entry), change scale, columns, jump to page, search, outline, open a page in the zoom overlay, navigate/close it, back to dashboard, Zen in/out? Record dead ends and the Tab order.
- **C6** Live regions: after a search with results, with no results, and after a forced error (open `not-a-pdf.pdf`), does any node carry a `live` attribute or an alert role that a screen reader would announce? Check the page-loading placeholders and the match badges too.
- **C7** Screen reader run (time box **60 minutes**): start Orca, capture its speech output (Orca debug/log output or speech-dispatcher log; if no capture method works within 30 minutes record "manual listening needed" and stop), and walk: dashboard → open a file → viewer toolbar → search → outline → ⚙ controls → (X11 run) open a page in the zoom overlay and read its text layer → close. Record the exact announcements and anything silent, wrong, or repeated. Return Orca to its previous state afterwards.
- **C8** Language and title: with the debug build's inspector (`Help → Toggle Developer Tools`) read `document.title` and `document.documentElement.lang` in the English and the Japanese UI (`LANG=ja_JP.UTF-8`, or settings `ui.locale`); record whether the language attribute changes with the locale and what a screen reader announces for Japanese text.
- **C9** Release build: `cargo build --release -p app`, run it, dump the tree: does the `Window / Edit / Help` menu bar and `Toggle Developer Tools` exist in release? Is the developer-tools window reachable? Record exactly how you reached it.
- **C10** Flatpak: **not applicable** to current packaging. Do not build one. Only note, if observed, whether our release tarball run shows the same tree as the debug run (it should: the tree was exposed without any sandbox changes).

### Out of scope for Part C
Windows (UI Automation/Narrator) and macOS (VoiceOver) runs — the owner decides separately who tests them; record in the package that they were not run.

## Deliverable
A review-request package `.git-exclude/review-request/wp0-evidence-run/README.md` (single entry point; tell the owner only this path, relative to the project root, at the end) containing the Part A results table, the Part B measurements table, the Part C results (tree dumps, the three lists of C3, focus trace, keyboard walk, announcements), screenshots under the same directory, the scripts used, and anything surprising. Include the standard review-request fields (summary, executed commands, results, known limitations).

## Prohibited
Changes to `crates/`, `rfcs/`, `docs/`, CI, tags, releases; committing scratch files; guessing — write "not run" when something was not run.
