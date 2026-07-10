---
project: PDF Tile Viewer
document_family: Post-2.0 / v2.1 candidate RFCs
language: English
date: 2026-07-10
status: Proposed
baseline: PDF Tile Viewer 2.0.0 release candidate line
priority: v2.1 candidate
depends_on: RFC 009, RFC 016, RFC 031
---

# RFC 032 — Persistent Document History With Privacy Controls

## 1. Summary

Evaluate persistent document history with explicit privacy controls.

This is a v2.1-class feature because storing document paths can reveal
sensitive filenames, directories, clients, projects, or personal information.
It must be opt-in or otherwise owner-approved before implementation.

## 2. Motivation

Persistent history can help users resume work across launches, but it creates a
privacy tradeoff. PDF Tile Viewer currently keeps session history ephemeral,
which is safer by default.

## 3. Goals

- Decide whether persistent history should exist.
- Define the default policy, recommended as opt-in.
- Define clear/reset controls.
- Define what an entry stores.
- Keep password-protected PDF credentials out of history.
- Ensure private workflows can disable history writes.

## 4. Non-Goals

- Adding cloud sync.
- Indexing PDF contents.
- Recording search queries.
- Persisting passwords or unlock state.
- Changing the current ephemeral session history before design acceptance.
- Entering the 2.0.0 final release gate.

## 5. Design Questions

- Should persistent history be opt-in only?
- Should entries store full paths, display names only, or redacted paths?
- Should missing files remain visible, be hidden, or be removed lazily?
- Should there be a private/incognito mode that disables history writes?
- How should history interact with picker initial-directory memory?
- Should history entries be cleared automatically after a retention period?

## 6. Acceptance / QA Checklist

- [ ] Default behavior is explicitly documented.
- [ ] User can clear persisted history.
- [ ] User can disable future history writes.
- [ ] No password or credential is written.
- [ ] History entries do not bypass normal PDF validation or password prompts.
- [ ] Missing-file behavior is deterministic.
- [ ] Tests cover opt-in, opt-out, clear, migration, and corrupt history files.
