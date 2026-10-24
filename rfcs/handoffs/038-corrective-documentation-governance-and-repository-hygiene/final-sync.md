# Handoff — Final sync for the 2.0.2 train (documentation, known-failing markers, CHANGELOG)

Governing RFC: [`../../accepted/038-corrective-documentation-governance-and-repository-hygiene.md`](../../accepted/038-corrective-documentation-governance-and-repository-hygiene.md) (docs must match the product). Order: last slice of the 2.0.2 train, after RFC 037 and RFC 043 are committed. Working model: directly on `main`, one actor, hand over with the commit hash, `main` green. No tags, no releases (DEC-001); do **not** run *Prepare release*.

## Tasks
1. **Read-through against the product**: re-run the release-QA checklist (`docs/src/contributors/release-qa.md`) on a **release build**; for every line marked "known failing until RFC NNN" whose RFC has landed, verify and remove the marker (record each in the review request); keep and justify the rest.
2. **Statements**: `docs/src/intermediate/shortcuts.md` equals the resolver table; `architecture.md` (crate graph and the new modules: `domain::shortcuts`, `components/modal.rs`, `zoom_overlay/*`, `session_service`, retention); `features.md`, `tile-grid.md`, `settings.md` (reserved keys, locale row), FAQ (placeholder on far tiles, language), `dioxus-desktop-lessons.md` (every "tracked in" statement matches reality), `versioning.md` exists and is linked; README/README.ja bullets truthful; `store-listing.md` is **not** edited.
3. **CHANGELOG**: consolidate `[Unreleased]` into one section headed `## [2.0.2] - unreleased` (the *Prepare release* workflow dates it), grouped Fixed / Changed / Added(documentation); keep an empty `## [Unreleased]` above; `Changed` lists user-visible behavior changes (for example the Auto/Fixed control) and **not** the idle-Escape change (it is 2.1.0). Verify with `ci/verify-version-consistency.sh`.
4. **RFC bookkeeping**: Status lines and `rfcs/README.md` (move RFC 033–040 and 043 to Implemented only when the architect confirms after the release; here only record "implemented in this train, awaiting release"); handoff files marked historical per RFC 000 after release.
5. **Carried over from the RFC 034 review:** (a) split `crates/pdf_engine/src/navigation.rs` (409 effective lines) by responsibility — move the destination conversion (`destination_view`, `fit_at`, `fit_without_coordinate` and helpers) into `navigation/destination.rs`; a pure move, test names unchanged, same gate; (b) add a `UserUnit` fixture (`/UserUnit 2`) to the generator, record what PDFium reports for it, and assert that the document still opens and renders and that overlay geometry is either correct or absent (never misplaced) without a crash; document the limitation in `docs/src/new-users/search.md` ("highlights may be missing on pages with unusual page-size settings").
6. Mark `pdf_engine::worker::EngineHandle::unavailable()` as `#[doc(hidden)]` with a doc line stating that it is test support (RFC 035 review).
7. Gate green; link check over `README*.md`, `docs/`, `rfcs/`; `mdbook build docs`.

## Non-change scope
Code, workflows, versions (CI sets them), Store text.

## Review request
`.git-exclude/review-request/final-sync/README.md` (single entry point): checklist results line by line, markers removed or kept, diffs of statements, CHANGELOG text.
