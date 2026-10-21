# Lessons for Dioxus Desktop Apps

This chapter collects what building and reviewing PDF Tile Viewer taught us about
**Dioxus Desktop** applications. It is written for contributors here and for other
teams building similar apps. Items marked **fixed in** or **tracked in** point to
the RFC that addresses them; issues that are still open in the released version
are stated as such.

**Scope and confidence.** Observations come from Dioxus 0.7.9 Desktop (wry,
WebKitGTK) on Linux, October 2026. Windows (WebView2) and macOS (WKWebView) were
not tested. Each point carries one of these labels:

- **Observed** — reproduced by running the application or an experiment.
- **Source** — established by reading framework or library source.
- **Report** — second-hand, not verified here.

Re-check version-specific points before relying on them elsewhere.

## 1. Signals: a hidden temporary can abort the whole application

**Rule:** never keep a `read()` or `peek()` guard alive across a `set()` or
`write()` of the same signal. This includes *temporaries inside call arguments*,
which live until the end of the enclosing statement.

```rust
// BAD: the guard returned by peek() lives for the whole statement,
// so it is still alive when change_scale() calls scale.set(..)
change_scale(*scale.peek() + 0.2);

// GOOD: end the borrow first
let current = *scale.peek();
change_scale(current + 0.2);
```

- **Observed.** The bad form raised an `AlreadyBorrowed` panic inside an event
  callback that cannot unwind, which **aborts the process** (no recoverable
  error). It shipped in 2.0.0 behind documented shortcuts (`+`, `=`, `-`,
  Ctrl+scroll) and was found only when someone pressed the keys on a release
  build. Fixed in RFC 039.
- **Prefer pure state transitions.** Compute the next value in a plain function
  (`next(current, change) -> Option<new>`), unit-test it, and let the component
  only read, compute, and set.
- **Guard rails.** Keep this rule in the contributor guide, and exercise every
  documented shortcut and gesture on a release build before each release.

## 2. Events bubble: design keyboard handling, do not attach it

- **Source.** Dioxus delivers an event to the target and then to every ancestor
  handler until `stop_propagation()` is called (`dioxus-core`,
  `handle_bubbling_event`). A keyboard handler on a container therefore fires for
  **every descendant**, including text inputs and overlays rendered inside it.
- **Observed.** Typing `z` in a search box toggled a viewer mode and removed the
  search panel; `0` reset the zoom while a page number was being typed; `Escape`
  inside an overlay closed it *and* left the document. Tracked in RFC 033.
- **A design that works:**
  1. one pure resolver, `resolve(context, key) -> Option<Action>`, unit-tested
     without any UI;
  2. **modal layers own the keyboard**: they handle their keys and always stop
     propagation at their root;
  3. editable fields use **one shared helper** (do not rely on every developer
     remembering to stop propagation);
  4. a documented **dismissal order**; Escape dismisses the top-most transient
     layer and never navigates away.

## 3. Accessibility in a WebView desktop application

- **Observed — the platform helps.** WebKitGTK, through wry, publishes a normal
  AT-SPI tree in an ordinary (non-sandboxed) install: landmarks, headings, named
  dialogs, and buttons with a `press` action. Semantic elements give you a lot for
  free.
- **Observed — what we still got wrong (all in our own markup):**
  - Glyph- or emoji-only buttons are *named by the glyph*. `title` is hover text,
    not a reliable accessible name (the screen reader we tried fell back to it,
    other tools may not). Set `aria-label`.
  - Range and number inputs with a visible text beside them but no `for` or
    `aria-label` are unnamed.
  - A `div` with an `onclick` is invisible to keyboards and screen readers (not
    focusable, no action). Our page tiles could open the zoom view only with a
    mouse. Use a real control and handle keys.
  - Focus does not move by itself when you show a panel. Opening search left focus
    on the button; closing panels left focus nowhere. Define a focus rule per
    layer: on open, move to the first control; on close, return to the opener.
  - Asynchronous results need **live regions** (`role="status"` for results,
    `role="alert"` for errors).
  - `document.title` stayed `Dioxus app` and `<html lang>` was empty in both
    locales although the window title followed the locale. Set both from your
    locale; screen readers choose the voice from `lang`.
  - Dialogs need an **inert background**, not only `aria-modal`. Screens other
    than the first lacked a `main` landmark. Plural strings ("1 matches") need
    plural-aware i18n.
  Tracked in RFC 033 and RFC 037; keyboard navigation of the tile grid and a
  fuller screen-reader pass are planned for the next minor release.
- **Limits.** The screen reader tested (Orca) cannot inject its own navigation
  keys on Wayland, so reading page content was not evaluated; Narrator and
  VoiceOver were not run. Plan a **human listening session**.
- **Report.** Another project reported that, with the GTK 3 WebKit, the web-process
  sandbox is off unless the application enables it and that the sandboxed process
  needed a proper application id before accessibility worked in a Flatpak. We do
  not ship a Flatpak and did not verify this; check it if you do.

## 4. UI state is not bounded by your backend cache

- **Observed.** A 256 MB bounded render cache in Rust did not bound memory:
  every rendered page was also kept in UI state as a `data:image/png;base64,…`
  string. On a release build, a 1000-page text PDF reached about 600 MB and a
  100-page image PDF about 1 GB, and memory was never reclaimed while scrolling
  away. Tracked in RFC 035 and a follow-up retention RFC.
- **Remedies.** (a) A **retention window** (keep rendered items only near the
  viewport; the backend cache re-serves them cheaply) bounds memory by window size
  instead of document size. (b) Serve images through a custom protocol instead of
  megabytes of base64 (**Source:** `dioxus-desktop` provides `use_asset_handler`).
  If you add a protocol handler, serve only cache entries addressed by opaque ids,
  never file paths, and review your content-security policy.
- **Results must record what they were computed for.** A tile marked "ready"
  without its scale stayed blurry forever after zooming. Store the parameters
  (scale bucket, generation) with the result and compare them before skipping work.

## 5. Layout: measure the element, not the window once

- **Observed.** Reading `window.innerWidth` once at mount meant the automatic
  column count ignored window resizing, maximizing, and a side panel opening.
  Observe the size of the scroll container and recompute (planned for the next
  minor release).

## 6. Lifecycle: unmounting a component does not free native resources

- **Observed.** Returning to the start screen left the native document, its
  session and caches alive (about +100 MB per open of a large file). Tie native
  resources to an explicit close path. Tracked in RFC 035.
- **Source.** Dioxus provides `use_drop` for unmount hooks and `spawn_forever` for
  tasks that must outlive a scope; tasks started with `spawn` are cancelled with
  their scope. If the backend command is sent when you call it (as it is here), a
  **synchronous fire-and-forget close** is enough and works inside `use_drop`.
- Make late asynchronous results harmless: tag results with
  `(document id, generation)` and keep a **bounded** record of closed sessions in
  every cache.

## 7. Testing a desktop WebView application on Linux

- **A repeatable manual protocol beats ad-hoc clicking.** AT-SPI tree dumps
  (Python `gi`/`Atspi`) of every screen and state are scriptable and diffable. A
  refactor of the zoom overlay was verified by *identical accessibility trees
  before and after*.
- **Safety rules learned the hard way:**
  - Synthetic input goes to whatever window has focus (one early run typed into
    another terminal). **Verify the focused window before every send.**
  - Run the application with a **scratch `XDG_CONFIG_HOME`** (an early run
    overwrote a developer's real settings).
  - Start the application as a transient `systemd --user` unit so interrupted
    sessions leave nothing behind.
  - Type slowly (`wtype -d 50`); fast input dropped characters.
- **Backends.** Native Wayland windows cannot be driven by `xdotool`; run the app
  under XWayland for pointer actions and say so in the report. Portal file
  choosers may need a second Enter after typing a path.
- **Timing without a profiler.** Frame-based latency (window screenshots every
  15–40 ms and a pixel statistic) gave usable open, overlay and scroll numbers.
  State the resolution (about 40 ms) and treat results as order-of-magnitude.

## 8. Packaging and product surface

- **Observed.** A release build still shows the framework's default menu bar
  (`Window / Edit` on Linux; a debug build adds `Help → Toggle Developer Tools`).
  Decide your menu deliberately (macOS needs a standard application menu).
- Run the GUI crate's tests in CI (ours were not, although the dependencies were
  installed) and build and test on every target OS before tag day.
- Verify bundled native binaries against a **pinned checksum**, pin CI actions by
  commit, and give each job only the permissions and secrets it uses (RFC 036).

## 9. Process lessons

1. **Reading code is not evidence.** A review read the crashing file and missed
   the abort; running the shortcuts found it.
2. **Manual-QA claims need artifacts.** An earlier "Escape passes" note could not
   be reproduced.
3. **Behavior-preserving refactors can be proven:** compare test-name sets,
   token-level move equivalence, and accessibility-tree identity rather than
   trusting a summary.
4. **Design before code still pays off:** most findings were caught as design
   premises and then confirmed by running; the one surprise was a trivial
   implementation error, which is why the release checklist matters.

## 10. Checklist

- [ ] No guard (`read`/`peek`) alive across `set`/`write` of the same signal;
      state transitions are pure functions with tests.
- [ ] One keyboard resolver; modal layers stop propagation; editable fields use one
      helper; Escape dismisses only.
- [ ] Every interactive element is a real, focusable control with an accessible
      name; icon-only controls have `aria-label`.
- [ ] Focus is moved on open and restored on close for every panel and dialog; the
      background is inert behind modals.
- [ ] Live regions for results and errors; document `title` and `lang` set from the
      locale; landmarks on every screen.
- [ ] UI memory is bounded independently of document size; results record their
      parameters.
- [ ] Native resources are released on an explicit close; late asynchronous results
      are refused.
- [ ] Layout follows the container size.
- [ ] A release-build smoke test of **every documented shortcut and gesture** on
      every OS you ship.
- [ ] A GUI test protocol: focus guard, scratch config, transient unit,
      accessibility-tree dumps, and a human screen-reader session.
