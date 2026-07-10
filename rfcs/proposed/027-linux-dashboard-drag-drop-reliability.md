---
project: PDF Tile Viewer
document_family: Post-2.0 / v2.1 candidate RFCs
language: English
date: 2026-07-10
status: Proposed
baseline: PDF Tile Viewer 2.0.0 release candidate line
priority: Post-2.0 candidate
depends_on: RFC 002, RFC 025
---

# RFC 027 — Linux Dashboard Drag/Drop Reliability

## 1. Summary

Investigate and harden dashboard PDF drag/drop on Linux desktop sessions.

During RFC 026 manual QA, opening PDFs through the file picker worked, but
dashboard file drop failed under both the observed Wayland session and an
`GDK_BACKEND=x11` comparison run. Because picker intake remains reliable, this
belongs to the post-2.0 planning queue and is not part of the 2.0.0 final
release gate.

## 2. Motivation

Drag/drop is a convenient intake path and is already part of the app's user
experience. If Linux desktop environments or WebKitGTK/Wry event routing make
that path unreliable, users need either a real fix or clear documentation about
supported environments.

## 3. Goals

- Identify whether the failure comes from Dioxus Desktop, Wry/WebKitGTK,
  compositor/file-manager behavior, or app event wiring.
- Preserve the existing picker path and `.pdf` validation behavior.
- Add a reliable Linux drop path if available.
- Document unsupported toolkit/session combinations if reliability cannot be
  guaranteed.

## 4. Non-Goals

- Replacing the dashboard picker.
- Adding document history or persistent intake preferences.
- Changing PDF validation or password-protected PDF behavior.
- Entering the 2.0.0 final release gate.

## 5. Design Questions

- Can `tao::WindowEvent::DroppedFile` be wired through a desktop event handler
  without weakening existing Dioxus event handling?
- Does WebKitGTK expose enough drag payload detail for file drops in both
  Wayland and X11 sessions?
- Which file managers and compositors should be part of manual QA?
- Should Linux docs call drag/drop best-effort if the toolkit path remains
  inconsistent?

## 6. Acceptance / QA Checklist

- [ ] Picker open still works for valid PDFs.
- [ ] Picker rejects non-`.pdf` files as before.
- [ ] Dashboard drop opens a valid PDF on at least one supported Linux session.
- [ ] Dashboard drop rejects non-`.pdf` files.
- [ ] Wayland result is recorded.
- [ ] X11 result is recorded.
- [ ] If unsupported combinations remain, user docs describe the limitation.
- [ ] This RFC remains outside the 2.0.0 final release gate.
