# Handoff — RFC 040 follow-up: where does the WebKit-side memory come from?

Governing RFC: [`../../accepted/040-tile-retention-window.md`](../../accepted/040-tile-retention-window.md) (§ 9 "Result and follow-up"). Executes this investigation only; it changes **no product code on `main`**.
Order: immediately after the architect's RFC 040 review; before RFC 033. Working model: directly on `main` for documents only; **prototype work only in a throw-away clone (`git clone`, never a worktree — tags are shared)**. No tags, no releases (DEC-001).
**GUI rules apply** — `.git-exclude/rules/project-instructions-gui-common.md`, *GUI Evidence and Automation Rules* (one instance chosen by process id, isolated `XDG_CONFIG_HOME`, X11 backend stated, assert then continue, **abort at the first anomaly**, no frame-sampling loops, repeat limit, same conditions before/after).

## Why
RFC 040's measured acceptance is not met: the app process roughly halves, but the WebKit child processes keep growing (text-1000: 282 → 626 MB over two passes) although at most a dozen tiles are `Ready`. The RFC's premise (live `<img>` elements and signal state drive WebKit memory) is therefore not supported. Before any design decision we need to know **what the memory is**. Each question below is answered by one recorded-state run; no pictures are needed except the single end-of-scroll assertion already used.

## Rulings on the dev's questions
1. The 55 % criterion stays as written; its result is recorded as **not met**. Nobody changes it after the fact. The decision follows these results.
2. **Third runs:** the two text-1000 "before" sets are **not** repeated (the WP0 and pass-1 numbers suffice). Permitted, each **once**: the script change you proposed (fixed generous wait + one content-area pixel assertion), and the runs listed below.
3. `with_min_cap` rule: **confirmed** (RFC 040 § 4.1 clarified).

## Questions and runs (one run each; stop at the first anomaly)

**Q1 — Is the WebKit growth reclaimable?** Build `07c71fa` (release, staged), text-1000: after "scroll down + back to top (pass 1)" read RSS (app + children) immediately, then after **30 s** and after **120 s** of no input. Interpretation: a clear drop = lazy reclamation (GC or allocator trimming), not a leak; no drop = retained.

**Q2 — Does the transport matter?** In a throw-away clone of `07c71fa`, prototype serving tile images through a custom protocol instead of `data:` URLs: `dioxus::desktop::use_asset_handler("tiles", …)` (read its documentation and `RequestAsyncResponder` in `dioxus-desktop-0.7.9/src/hooks.rs` and `protocol.rs`); tile `src` = a short URL such as `tiles/<document>/<generation>/<page>/<bucket>`; the handler returns the PNG bytes from the render cache (on a miss it renders through `RenderService::get_or_render`). Keep `apply_retention` as it is. Run the **same** text-1000 sequence once (down, back, down, back, same wait). Report the same table plus Q1's idle readings. This is a measurement prototype: **never merge it**; save the diff as `.git-exclude/review-request/040-tile-retention/prototype-protocol.patch`.

**Q3 — image-100 (the 1 GB case).** One run of release `a8ca205` ("before") and one of `07c71fa` ("after") with the new fixed-wait method: open, down, back, down, back, RSS at each step.

**Q4 — Behavior checks of the handoff § 4 (recorded state, on `07c71fa`)**: far tiles show the loading placeholder and fill in again after scrolling back (accessibility tree or a DOM state read, one timed check); tiles within two viewport heights keep their image; a search still highlights; the zoom view opens. No sampling loop.

## Deliverable
`.git-exclude/review-request/040-tile-retention/README.md` updated (single entry point): a table per question with the numbers, the prototype patch, what each result implies in two sentences, and the list of anomalies/aborts. **Do not interpret beyond the data**; label hypotheses as such.

## Prohibited
Committing the prototype to `main`; tuning window or cap to chase a number; a third run of any set without asking the architect; changing the acceptance text.

---

## Round 2 (architect, 2026-10-24, after the review of Q1–Q4)

Ruling: Q1, Q2 and Q4 are **complete and accepted**; no further prototype and no repeat of them. **Only one run remains: Q3'** — image-100 on `a8ca205` ("before", your rebuilt binary) and on `07c71fa` ("after"), once each, at the same window conditions as Q1/Q2 (1042×1134, one tile per row). Replace the fixed waits by the recorded-state wait you proposed: poll the accessibility tree once per second (at most 5 minutes) until the tile of the last page has an `image` element, and do this before every RSS reading (after open, after scroll down, after scroll back, after the second down and back). Report the table (app / WebKit / total) and, as a **separate line**, how long the backlog took to clear after each scroll (seconds until the last-page tile had its image). Abort at the first anomaly; no repeat without asking. Then update the review request and tell the owner only its path.
