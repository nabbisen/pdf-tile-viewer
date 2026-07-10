---
project: PDF Tile Viewer
document_family: Post-2.0 / v2.1 candidate RFCs
language: English
date: 2026-07-10
status: Proposed
baseline: PDF Tile Viewer 2.0.0 release candidate line
priority: v2.1 candidate
depends_on: RFC 008, RFC 016
---

# RFC 031 — Settings Storage With `app-json-settings`

## 1. Summary

Evaluate whether PDF Tile Viewer should replace or wrap its current settings
storage with the `app-json-settings` crate.

This is a v2.1-class design topic because settings storage affects migration,
privacy, schema compatibility, and error recovery. It should not be rushed
into 2.0.0 final.

## 2. Motivation

The current settings implementation is intentionally simple and has carried
the migration. A dedicated settings crate may reduce local maintenance, but
only if it matches the app's expectations for platform config directories,
schema evolution, corrupt-file recovery, and privacy defaults.

## 3. Goals

- Compare current settings behavior with `app-json-settings`.
- Decide whether to replace, wrap, or keep the current settings service.
- Preserve existing user settings during migration.
- Keep privacy-sensitive defaults conservative.
- Define corrupt-file and unknown-field recovery behavior.

## 4. Non-Goals

- Adding persistent document history; see RFC 032.
- Persisting passwords or PDF credentials.
- Changing privacy defaults without explicit owner approval.
- Entering the 2.0.0 final release gate.

## 5. Design Questions

- Does `app-json-settings` choose platform config directories that match the
  current product policy?
- How does it handle schema versioning and migration?
- How does it recover from invalid JSON or partial writes?
- Can the app keep a compatibility read path for existing settings?
- Which current settings are safe to persist by default?
- What tests are needed to prove migration does not lose settings?

## 6. Acceptance / QA Checklist

- [ ] Current and proposed storage paths are documented.
- [ ] Existing settings migrate or continue to load.
- [ ] Corrupt settings files fail safely with clear recovery behavior.
- [ ] Unknown future fields do not break startup.
- [ ] Privacy-sensitive settings keep conservative defaults.
- [ ] No password or protected-document secret can be stored.
- [ ] Tests cover default creation, load, save, migration, and corrupt input.
