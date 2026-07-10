---
project: PDF Tile Viewer
document_family: Post-2.0 / v2.1 candidate RFCs
language: English
date: 2026-07-10
status: Proposed
baseline: PDF Tile Viewer 2.0.0 release candidate line
priority: Post-2.0 candidate
depends_on: RFC 002, RFC 016, RFC 025
---

# RFC 028 — File Picker Initial Directory Memory

## 1. Summary

Remember the parent directory of the last successfully opened PDF during the
current app session, and use it as the initial directory for the next
**Open PDF...** picker.

This is a convenience improvement. The default design should be memory-only so
it does not introduce persistent path privacy concerns.

## 2. Motivation

Users often open several PDFs from the same directory. Restarting the picker in
the last successful directory reduces repeated navigation without needing
persistent history.

## 3. Goals

- Store the last successful PDF parent directory in memory only.
- Use that directory as the next picker starting location when the platform
  picker API supports it.
- Update the remembered directory only after a successful document open.
- Preserve all current validation and password-protected PDF behavior.

## 4. Non-Goals

- Persisting the directory across app launches.
- Adding recent-file history.
- Adding global settings for picker behavior.
- Recording failed picker selections.
- Entering the 2.0.0 final release gate.

## 5. Proposed Behavior

The app should keep a session-scoped optional `PathBuf` for the most recent
successful PDF parent directory. Picker open should pass this path to the
native picker as the initial directory if the current picker crate supports
that option. If the directory no longer exists or the platform ignores the
hint, the picker should fall back to its current default behavior.

Drag/drop should not update this memory in the first implementation unless the
owner explicitly chooses that behavior. Keeping the first slice picker-only is
more predictable and avoids blending it with history semantics.

## 6. Acceptance / QA Checklist

- [ ] First picker open uses the platform default location.
- [ ] After opening a PDF, the next picker starts in that PDF's parent
      directory where supported.
- [ ] Failed opens do not update the remembered directory.
- [ ] Non-`.pdf` rejection does not update the remembered directory.
- [ ] Password cancellation does not update the remembered directory.
- [ ] The remembered directory is cleared when the app exits.
- [ ] No path is written to settings, logs, or release artifacts.
