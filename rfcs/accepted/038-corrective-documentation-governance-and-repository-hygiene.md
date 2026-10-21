---
project: PDF Tile Viewer
document_family: 2.0 corrective RFCs
language: English
date: 2026-10-21
status: Accepted (planned for 2.0.1)
accepted: 2026-10-21
baseline: PDF Tile Viewer 2.0.0
priority: Corrective
depends_on: RFC 000, RFC 019, RFC 020, RFC 022
amends: RFC 000 (applies it), README, ROADMAP, docs, CHANGELOG conventions
origin: architect onboarding review 2026-10-21 (internal), owner decisions D2, D3, D7
---

# RFC 038 — Corrective Documentation, Governance, and Repository Hygiene

## 1. Summary

Make every public statement about the project true and every project rule
followed: correct claims about features, bring the RFC directory into the
adopted lifecycle policy, split oversized files by responsibility, move inline
tests, and clean repository hygiene items. Like RFC 019/020/022, this is a
correction RFC; it adds no product feature.

## 2. Motivation (evidence)

The onboarding review found statements and states that mislead users or
contributors (P4) and rule violations that the handoffs recorded as compliant.

## 3. Goals / Non-goals

Goals: truthful docs, rule compliance, clean repository. Non-goals: changing
behavior (owned by RFC 033–037), rewriting published git history, new features.

## 4. Scope

### 4.1 Truthful product claims (owner decision D2)

- Remove "Mouse drag move with Ctrl key" from `README.md` and `README.ja.md`.
  Do **not** reinstate it. Add the outline/links bullets missing from
  `README.ja.md`.
- Clarify in `CHANGELOG.md` `[Unreleased]` (do not rewrite history) that v2 does
  not include Ctrl+drag panning, zoom background lock, or zoom transparency, and
  that earlier "feature parity" wording meant the core workflow.
- Note in `docs/design/01_goal_state_external_design.md` that those three
  baseline items were intentionally not carried into v2.
- Remove or mark the dead settings from `settings.md` as "reserved, not
  functional" **until** RFC 031 removes them from the schema (owner decision D3):
  `privacy.persist_recent_files`, `privacy.include_full_paths_in_diagnostics`,
  `advanced.diagnostics_enabled`, `window.maximized`, `viewer.zoom_overlay_alpha`.
  A visible warning is required for `persist_recent_files` because users may
  assume it works.

### 4.2 Documentation drift

- `ROADMAP.md`: record 2.0.0 as released, the approved corrective track and the
  next-minor themes; version numbers remain owner decisions.
- `docs/src/contributors/rfcs.md`: replace the stale 001–017 table with a link to
  `rfcs/README.md` (single index).
- `architecture.md`: remove the obsolete ELOC claim; describe the crate graph
  and the post-RFC 033–037 structure after they land.
- `platforms.md`: macOS Intel is built and supported (release matrix,
  `packaging::platform_segment`).
- `dev.md`: remove the false `rust-toolchain.toml` statement; declare
  `rust-version` in the workspace manifest only after measuring the minimal
  supported toolchain.
- `shortcuts.md`: Escape order follows RFC 033 (updated when RFC 033 lands).
- `.vscode/`: remove Svelte/Tauri recommendations; recommend `rust-analyzer`.
- `.github/ISSUE_TEMPLATE/config.yaml` → `config.yml`, with a contact link to
  this repository (the current link points to `nabbisen/apimock-rs`).
- Date the `[2.0.0]` CHANGELOG heading (2026-07-14, tag date).
- Contributor chapter `docs/src/contributors/dioxus-desktop-lessons.md` (added 2026-10-22 at the owner's request): keep it truthful as RFC 033, 035, 037 and 039 land (update the matching "tracked in" statements and the checklist); each later slice that confirms or changes a lesson edits the chapter in the same change.
- Commit and link `PRIVACY.md` from both READMEs and the docs; add a `ja`
  resource to the MSIX manifest when the owner confirms Store listing languages.

### 4.3 RFC lifecycle compliance (RFC 000, 5-folder variant)

- Add `rfcs/accepted/` and `rfcs/archive/` (each with a one-line `README.md`
  explaining its meaning, so Git tracks them) and Accepted/Archive sections in
  `rfcs/README.md`.
- Normalize Status: front-matter `status:` with the release tag
  (`Implemented (2.0.0-beta.N)`); RFC 001–017 and 026 receive the tag from the
  index milestone table; RFC 000 itself keeps its policy-form body field.
- Add "Known gaps" sections that point to the corrective RFCs: RFC 004 (close
  → 035), RFC 007 (freshness/cache → 035; bounded image transport → future
  render-pipeline RFC), RFC 011 (rotation → 034), RFC 013 (input guard → 033).
- Add "amended by" front-matter notes to RFC 004, 006, 007, 011, 012, 013, 017,
  024, 025.
- Replace front-matter `review:` paths that point into the unpublished
  `.git-exclude/` tree with "internal review record (not published)".
- Handoffs 023 and 026: fix stale paths and "next step" text; mark them
  historical per RFC 000 ("only current, reviewed" companions belong).

### 4.4 Rule compliance (code, behavior-preserving)

- Move the five inline `#[cfg(test)] mod tests {}` blocks to `foo/tests.rs`:
  `pdf_engine/src/engine.rs`, `pdf_engine/src/navigation.rs`,
  `app/src/components/tile_grid.rs`, `outline_panel.rs`,
  `app/src/components/zoom_overlay/util.rs`.
- Split `zoom_overlay.rs` (≈772 effective lines) **by responsibility**, not line
  count: page-image state, text layer, link layer, and the external-URI dialog
  as separate modules/components. This is a pure refactor and lands **before**
  RFC 033 so that behavior changes are reviewed against smaller files.
- Split `pdf_engine/tests/smoke.rs` (≈534) by topic (document, render, search,
  text, navigation, encryption) with shared test support.
- Re-measure and record file sizes in the handoff review request.

### 4.5 Repository hygiene (owner decision D7; no history rewrite)

- Remove `dist/pdf-tile-viewer-v2.0.0-linux-x64.tar.gz` from the index and add
  `/dist/` to `.gitignore`. The blob stays in history (rewriting published
  history is unsafe).
- Delete the local-only tag `2.0.1` (commit `5c69522`; recorded here for
  recovery). The tag-guard in RFC 036 prevents recurrence. Tag operations remain
  owner-confirmed (DEC-001).
- Move fixtures from `fixtures/` to `crates/pdf_engine/tests/fixtures/` (the
  sole consumer is the engine smoke suite); update the generator's output path,
  `smoke.rs`, `ci/archive-source.sh`, `dev.md`; historical RFC text is left
  unchanged. The repo-root `tests/` alternative was rejected because the
  workspace root has no package.

## 5. Order and dependencies

1. § 4.4 refactors (inline tests, zoom overlay split, smoke split) first.
2. § 4.1–4.3 and § 4.5 in parallel.
3. `shortcuts.md` and `architecture.md` final edits after RFC 033/035 land.

## 6. Tests and acceptance

- `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`
  (including `-p app`) and smoke tests unchanged in result and count.
- No file exceeds 500 effective lines; files above 300 are justified in the
  review request.
- A link checker over `README*.md`, `docs/`, `rfcs/` finds no broken relative
  links after the folder changes (RFC 000 cross-reference rule).
- `grep` finds no `mod tests {` in non-test sources.
- Every public claim listed in § 4.1 matches the application.

## 7. Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| Refactors accidentally change behavior | Medium | Pure-move commits, unchanged test results, review of diffs as moves |
| Fixture move breaks scripts | Low | Update all path consumers in one slice; run source-archive smoke |
