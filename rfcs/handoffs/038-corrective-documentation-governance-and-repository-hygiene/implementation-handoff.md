# Handoff — RFC 038 Corrective Documentation, Governance, and Repository Hygiene

Governing RFC: [`../../accepted/038-corrective-documentation-governance-and-repository-hygiene.md`](../../accepted/038-corrective-documentation-governance-and-repository-hygiene.md)
This handoff executes the RFC; it must not redefine it. If you find a conflict, stop and send a design request to the architect.

Two slices, **executed in this order and never overlapped**:

- **Slice A** — behavior-preserving refactors (RFC 038 § 4.4). Done and approved.
- **Slice B′** — documentation, RFC-policy compliance, repository hygiene (RFC 038 § 4.1–4.3, § 4.5). Order: second slice of the 2.0.2 train, **after** the RFC 036 slice is committed.

Working model: work directly on `main`, no branches; only one actor works at a time; hand over with the current `main` commit hash; `main` must be green before every push.

---

## Slice A — Behavior-preserving refactors

### Purpose
Bring the code into compliance with the Rust project rules (no inline test modules; files above 300 effective lines split, required above 500) **without changing behavior**, so that RFC 034, 033 and 037 are reviewed against small files.

### Background
Measured on `main` @ 2026-10-21 (effective lines = non-blank, non-`//` lines):
`app/src/components/zoom_overlay.rs` 772; `pdf_engine/tests/smoke.rs` 534; `zoom_overlay/util.rs` 485 (includes inline tests); `pdf_engine/src/navigation.rs` 362; `app/src/screens/viewer.rs` 348. Inline `mod tests {}` exists in five files (below).

### Change scope
1. **Move inline tests** to sibling `tests.rs` files (`#[cfg(test)] mod tests;` in the parent):
   - `crates/pdf_engine/src/engine.rs` → `crates/pdf_engine/src/engine/tests.rs`
   - `crates/pdf_engine/src/navigation.rs` → `crates/pdf_engine/src/navigation/tests.rs`
   - `crates/app/src/components/tile_grid.rs` → `.../tile_grid/tests.rs`
   - `crates/app/src/components/outline_panel.rs` → `.../outline_panel/tests.rs`
   - `crates/app/src/components/zoom_overlay/util.rs` → `.../zoom_overlay/util/tests.rs`
2. **Split `zoom_overlay.rs` by responsibility** (not by line count) into modules under `crates/app/src/components/zoom_overlay/`. Required separations:
   - state types and request-currency guards (`ZoomImageState`, `ZoomTextLayerState`, `ZoomPageLinksState`);
   - the external-URI dialog (`ExternalUriDialog`, its state/copy-status types, `focus_*`/`trap_*` helpers, `copy_text_to_clipboard`);
   - the selectable text layer rendering;
   - the PDF link layer rendering;
   - the three loading effects (page image, links, text layer) as small private hooks or functions.
   `ZoomOverlay` keeps its exact props and remains the only public item of the module tree.
3. **Split `crates/pdf_engine/tests/smoke.rs`** into a single test binary with sub-modules: `crates/pdf_engine/tests/smoke/main.rs` (shared support: engine `LazyLock`, `require_engine!`, fixtures helpers, `mod` declarations) and topic files `document.rs`, `render.rs`, `search.rs`, `text.rs`, `navigation.rs`, `encryption.rs`, `lifecycle.rs`. The binary name stays `smoke`, so `cargo test -p pdf_engine --test smoke` keeps working. Remove the old `smoke.rs`.

### Non-change scope
No behavior, API, CSS class/id, i18n key, setting, dependency, workflow, or documentation change. Do **not** touch keyboard handling, geometry, error messages, caches, or the fixtures location (later slices own those). Do not reformat unrelated code.

### Constraints that must be preserved
- DOM ids and CSS classes used by scripts or styles: `zoom-image-wrap`, `zoom-close-button`, `external-uri-cancel`, `external-uri-copy`, `external-uri-value`, `external-uri-dialog-title`, and every class in `main.css`.
- Event propagation behavior and handler order in `ZoomOverlay` (RFC 033 changes them later, not now).
- Test function names (module prefixes may change for the smoke split).

### Required evidence
1. Before the first change, record `cargo test --workspace -- --list 2>/dev/null | sort > before.txt` (store outside the repo). After each commit repeat and compare: the set of test function names must be identical (smoke module prefixes aside); total test count must remain **133**.
2. Gate after every commit: `cargo fmt --check`; `cargo check --workspace` (0 warnings); `cargo test --workspace`; `PDF_TILE_VIEWER_PDFIUM_DIR="$(pwd)/ci/.pdfium" cargo test -p pdf_engine --test smoke`.
3. Effective-line table for every file you touched (command: `grep -v -E '^\s*($|//)' FILE | wc -l`), before and after. No file above 500; files above 300 need a one-line justification.
4. `grep -rn -B1 -E '^\s*mod tests \{' crates --include='*.rs'` returns nothing.

### Commit plan (each commit green)
1. inline tests moved (all five);
2. smoke split;
3. zoom overlay split (one commit per extracted responsibility is preferred).

### Acceptance criteria
- Items 1–3 under Change scope are complete; evidence 1–4 satisfied.
- A reviewer can verify each commit as a pure move (`git diff --color-moved` shows moves, not edits).

### Prohibited shortcuts
Deleting or disabling tests; changing assertions; "while I was here" fixes (record them as findings instead); introducing `mod.rs`.

### Known risks
Dioxus component props must remain `PartialEq + Clone`; moving closures between modules can change capture semantics — keep signal/callback captures exactly as they are. Smoke tests share one PDFium instance per process; keep the single `LazyLock` + `ENGINE_LOCK` in `main.rs`.

### Review request
Create `.git-exclude/review-request/038-slice-a-refactors/README.md` (single entry point) with: implementation summary, commits (hashes), changed files, differences from this handoff, executed commands and results, before/after test lists, effective-line table, unresolved issues, requested review focus.

---

## Slice B′ — Documentation, policy compliance, hygiene

Precondition: RFC 036 slice committed (its CI changes must already guard these pushes) and `main` green. Owner decisions D2, D3, D7 apply (see RFC 038).

### Change scope (RFC 038 § 4.1, § 4.2, § 4.3, § 4.5)
1. **Truthful claims.** Remove the Ctrl+drag bullet from `README.md` and `README.ja.md`; add the missing outline/links bullets to `README.ja.md`. Add the clarification under `CHANGELOG.md` `[Unreleased]` → "Documentation". Add the "not carried into v2" note to `docs/design/01_goal_state_external_design.md`. In `docs/src/intermediate/settings.md` mark the five dead settings as **"Reserved — not functional in 2.x"**, and give `privacy.persist_recent_files` a prominent warning that setting it to `true` has no effect.
2. **Drift fixes.** `ROADMAP.md` (verify it reflects the accepted plan; remove stale statements); `docs/src/contributors/rfcs.md` → replace the table with a pointer to `rfcs/README.md`; `docs/src/contributors/architecture.md` (remove the ELOC claim; keep the crate graph); `docs/src/intermediate/platforms.md` (macOS Intel is supported); `docs/src/contributors/dev.md` (remove the `rust-toolchain.toml` sentence; do **not** invent an MSRV); `.vscode/` (remove Svelte/Tauri recommendations, keep `rust-analyzer`); rename `.github/ISSUE_TEMPLATE/config.yaml` → `config.yml` and set the contact link to `https://github.com/nabbisen/pdf-tile-viewer/discussions` if Discussions are enabled, otherwise remove the contact link and ask the architect; date the `[2.0.0]` heading `2026-07-14`.
   Leave `shortcuts.md` and the final `architecture.md` structure text for Slice G.
3. **RFC lifecycle compliance.** Create `rfcs/archive/README.md` (one paragraph: Withdrawn/Superseded RFCs live here; none yet) — `rfcs/accepted/` already exists. Add an "Archive" section (currently "none") to `rfcs/README.md`. Normalize `status:` in `rfcs/done/001`–`017` and `026` with the release tag, using this mapping (last release in which the RFC's work completed, from the index milestone table and CHANGELOG): 001 alpha.1 · 002 alpha.3 · 003 alpha.1 · 004 alpha.1 · 005 alpha.1 · 006 alpha.2 · 007 beta.2 · 008 alpha.3 · 009 alpha.3 · 010 alpha.4 · 011 alpha.4 · 012 beta.1 · 013 beta.1 · 014 beta.1 · 015 beta.2 · 016 beta.2 · 017 alpha.1 · 026 `2.0.0` (format `Implemented (2.0.0-alpha.1)`). Add "Known gaps" sections and `amended_by:` front-matter lines exactly as listed in RFC 038 § 4.3. Replace `review:`/`rereview:`/`implementation_review:` front-matter values that point into `.git-exclude/` with `internal review record (not published)`. Fix the stale paths/text in `rfcs/handoffs/023-…/implementation-handoff.md` and `026-…/implementation-handoff.md` and add a first line "Historical record — implemented in <release>".
4. **Repository hygiene (D7, no history rewrite).**
   - `git rm --cached dist/pdf-tile-viewer-v2.0.0-linux-x64.tar.gz`; add `/dist/` to `.gitignore`.
   - **(Obsolete since 2.0.1 was released; do not delete any tag.)** Record the commit of local tag `2.0.1` (`5c69522`) in your review request, then delete the **local** tag: `git tag -d 2.0.1`. Do not push or create any tag (DEC-001).
   - Commit `PRIVACY.md` (currently untracked) and link it from `README.md`, `README.ja.md`, and `docs/src/new-users/faq.md`. Do not edit its text; send questions to the architect.
   - Move `fixtures/` → `crates/pdf_engine/tests/fixtures/` with `git mv`; update `fixtures/generate_fixtures.py` output path and its docstring commands, `crates/pdf_engine/tests/smoke/main.rs` fixture helper, `ci/archive-source.sh`, `docs/src/contributors/dev.md`, `docs/src/contributors/testing.md`. Historical RFC text keeps the old path. Regenerate fixtures with the generator and confirm **byte-identical** output for every fixture except the externally generated `password-protected.pdf` (leave it as is).
   - Run the source-archive smoke (`ci/archive-source.sh`) and confirm the archive contains the new fixture location.

### Additional items (owner decisions 2026-10-24)
- **Versioning policy**: create `docs/src/contributors/versioning.md` per RFC 038 (public API = documented user-facing behavior; patch/minor/major rules; major is an owner decision); add it to `SUMMARY.md`; link from `dev.md` and `ROADMAP.md`.
- **MSIX languages**: add `<Resource Language="ja-JP" />` next to `en-US` in `packaging/windows/AppxManifest.xml` (Store listing text exists in `packaging/windows/store-listing.md`; do not edit it). Do not add a Japanese-UI claim anywhere (system-locale detection is not fixed yet).
- **PRIVACY.md** is committed and linked as already specified; also add its URL to `packaging/windows/store-listing.md` checklist item 2 only if the architect asks.
- `dist/` removal (already listed above) is done by this slice; RFC 036 (earlier) fixes the workflow side so that no stray file can reach a release.

### Non-change scope
No code behavior, CI logic (RFC 036 owns it), shortcuts text, or release actions.

### Required evidence
Gate green; a link check over `README*.md`, `docs/`, `rfcs/` with no broken relative links (command and output in the review request); `git status` showing no `dist/` tracked; fixture byte-identity table; source-archive smoke output.

### Acceptance criteria
Every statement in RFC 038 § 4.1–4.3 and § 4.5 is true in the tree; evidence delivered.

### Prohibited shortcuts
Rewriting git history; editing `PRIVACY.md`; changing historical RFC bodies beyond the listed front-matter/known-gaps additions; leaving broken links "for later".

### Review request
`.git-exclude/review-request/038-slice-b-prime-docs-hygiene/README.md` with the same fields as Slice A, plus the link-check output and the recorded `2.0.1` tag commit.
