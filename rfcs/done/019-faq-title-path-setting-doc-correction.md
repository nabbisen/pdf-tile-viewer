---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-07-05
status: Implemented (2.0.0-beta.9)
baseline: PDF Tile Viewer 2.0.0-beta.8
---

# RFC 019 — FAQ Correction for Full-Path Window Title Setting

## 1. Summary

Correct user documentation that still says
`privacy.show_full_path_in_title` is schema-only, even though the app now uses
that setting to control the window title.

## 2. Motivation

RC documentation must describe the shipped behavior accurately. The current FAQ
claims the full-path title setting is not wired and reserved for a future
release. That is stale: the app title now shows the full file path when the
setting is enabled.

## 3. Goals

- Update the FAQ to describe the actual behavior.
- Keep the privacy-safe default clear: filenames are shown by default, full
  paths require explicit opt-in.
- Ensure the settings reference and FAQ agree.

## 4. Non-Goals

- Adding a GUI settings editor for this key.
- Changing the setting default.
- Persisting recent files.

## 5. Proposed Design

Replace the stale FAQ answer with:

- By default, the window title shows only the document display name.
- If `privacy.show_full_path_in_title` is set to `true` in settings, the title
  shows the full local file path.
- This setting is local and privacy-sensitive because full paths can reveal
  usernames or folder names.

## 6. Acceptance Criteria

- No user-facing documentation says `privacy.show_full_path_in_title` is
  unwired or future-only.
- The FAQ and settings reference are consistent.
- `mdbook build docs` passes.

## 7. Risks

| Risk | Mitigation |
|---|---|
| Users infer there is an in-app settings UI | State that the key lives in the settings file unless a UI is added later. |
| Privacy behavior is overclaimed | Keep the default and opt-in behavior explicit. |
