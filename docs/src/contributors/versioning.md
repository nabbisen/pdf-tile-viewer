# Versioning Policy

PDF Tile Viewer uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
This page says what the project treats as its **public API**, and when each part of
the version number increases. Every release version is proposed by the architect and authorized by the owner; a major
version change is **always** the owner's own decision (see
[Release procedure](dev.md#release-procedure)).

## What is the public API

PDF Tile Viewer is an application, not a library. Its public API is the **documented
user-facing behavior**:

- **Shortcuts and gestures** in
  [Keyboard Shortcuts](../intermediate/shortcuts.md) and
  [Navigating the Tile Grid](../new-users/tile-grid.md).
- **The settings file**: its location, the keys, their meaning and the file schema
  (`schema_version`) described in the [Settings Reference](../intermediate/settings.md).
  Keys marked *Reserved* are not part of the API until a release documents them.
- **Environment variables** that are documented for users and developers, such as
  `PDF_TILE_VIEWER_PDFIUM_DIR` (development and test builds only).
- **The release archive layout**: the names of the archives, `bin/` and `resources/`
  at the archive root, and the bundled PDFium under `resources/pdfium/<platform>/`.
- The documented workflows: opening a file (picker, drag and drop, history), search,
  zoom view, outline and links, zen mode, password prompt, and the privacy defaults
  (nothing is sent anywhere; session-only history; filename-only window title).

**Not public API:** the Rust crates (`domain`, `pdf_engine`, `app_services`,
`packaging`, `app`) are internal and unpublished; their types and functions may change
in any release. Likewise visual details (exact colors, spacing, icons), internal
caches, log text, and anything the documentation does not describe.

## When the version increases

| Increase | When | Examples |
|---|---|---|
| **Patch** `X.Y.Z+1` | Backward-compatible **fixes of documented behavior**, security fixes (including dependency updates), documentation corrections, and build/release machinery. Nothing a user could consider a different feature. | A shortcut crashes instead of doing what the documentation says (2.0.1); wrong highlight positions on rotated pages; memory that grows without bound; accessibility defects; CI and packaging hardening |
| **Minor** `X.Y+1.0` | **Added or changed documented behavior**, new settings keys, deprecations, new platforms or packaging formats. Existing settings files keep working. | Escape no longer leaves the document; a new setting; viewport-tracking columns that change how Auto behaves; persistent history |
| **Major** `X+1.0.0` | Breaking changes to the public API above that existing users cannot avoid (a settings file that no longer loads, a removed documented feature, a changed archive layout) — **and** the owner's decision. Completing a set of RFCs never triggers a major by itself. | A new engine or UI stack that changes documented behavior |

A change that fixes a bug **and** changes what the documentation promised is a
minor change: the documentation is the contract. When in doubt between patch and
minor, choose minor.

## Pre-releases

Tags such as `2.1.0-rc.1` or `2.1.0-beta.2` are published as GitHub pre-releases. The
Microsoft Store package version is always `<core>.0` (a pre-release suffix is not
allowed there); see [Local Development](dev.md#microsoft-store-windows-msix).

## Who does what

The architect proposes the version and the release scope; the owner authorizes them
and performs the tagging (DEC-001). The version is changed only by
`ci/set-version.sh` (through the *Prepare release* workflow); every release has a dated
CHANGELOG section that becomes its release notes.
