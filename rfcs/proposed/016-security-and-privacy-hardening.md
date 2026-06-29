---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-06-07
status: Proposed
baseline: PDF Tile Viewer v1.1.2 reverse-engineered design + approved Dioxus goal-state design
---

# RFC-016 — Security and Privacy Hardening

## 1. Summary

This RFC defines security and privacy hardening for the migrated local PDF viewer. The app opens untrusted local PDF files and loads a native PDFium library, so path handling, library loading, diagnostics, recent files, and PDF external actions must be safe by default.

## 2. Motivation

PDF viewers process complex untrusted file formats. The migration centralizes PDF parsing/rendering in PDFium and changes native library loading behavior. This requires clear boundaries and default-safe behavior even though the app is local and offline.

## 3. Goals

- Restrict production PDFium loading to app-controlled paths.
- Avoid leaking full file paths by default.
- Make recent-file persistence opt-in or clearly controllable.
- Treat PDFs as untrusted input.
- Avoid network/external actions from PDF content in the first release.
- Provide diagnostics without exposing private paths unnecessarily.

## 4. Non-Goals

- Full sandboxing across all OSes.
- DRM or document access control.
- Malware scanning.
- Enterprise policy management.

## 5. Threat Model

| Asset | Threat |
|---|---|
| User local files | Accidental path disclosure in UI/logs/recent files. |
| App process | Malicious/corrupt PDF triggers parser/render crash. |
| Native library boundary | App loads attacker-controlled PDFium library. |
| Privacy | Recent files reveal document names/paths. |
| User intent | PDF external links/actions trigger unexpected behavior. |

## 6. PDFium Library Loading Policy

Production rules:

- Load only from app-controlled resource/cache path.
- Do not search current directory.
- Do not load arbitrary user-selected PDFium by default.
- Do not silently fall back to system `PATH` search.
- Record the loaded path in diagnostics with privacy filtering.

Development rules:

- Explicit dev fallback may load from configured paths.
- Dev fallback must be disabled in release builds unless intentionally configured.

## 7. File Privacy Policy

Default behavior:

- Window title shows document display name, not full path.
- Dashboard history shows file name and optional parent hint, not full path.
- Full path display is user-controllable.
- Persisted recent files are disabled by default unless product owner explicitly chooses otherwise.

Settings:

```rust
pub struct PrivacySettings {
    pub show_full_path_in_title: bool,
    pub persist_recent_files: bool,
    pub include_full_paths_in_diagnostics: bool,
}
```

## 8. Logs and Diagnostics

- User-facing errors should avoid full paths by default.
- Development logs may include full paths.
- Production diagnostic export, if implemented later, must ask before including full paths.
- Panic logs must avoid dumping document bytes.

## 9. PDF External Action Policy

For the first migrated release:

- Do not execute PDF embedded JavaScript.
- Do not auto-open external links.
- Do not auto-launch files or external actions from PDFs.
- Link/outline navigation is deferred to a future RFC and must require explicit user action.

## 10. Robustness Requirements

- Corrupt PDF should not crash the app if PDFium returns an error.
- Render/search failures should be isolated to the document/session where practical.
- Very large pages should be bounded by render scale and pixel limits.
- Cache budget must be enforced to avoid memory exhaustion.

## 11. User-Facing Security Copy

For documentation:

```text
PDF Tile Viewer is a local desktop viewer. It does not upload documents. PDF files are processed locally through the bundled PDFium engine. Recent-file persistence and full-path display can be controlled in settings.
```

This copy must only be used if the app truly performs no network upload.

## 12. Acceptance Criteria

- Release builds do not load PDFium from current working directory or arbitrary `PATH` search.
- Full path is not shown in window title by default.
- Persisted recent files respect privacy settings.
- PDF external actions are not executed automatically.
- Corrupt/unsupported PDFs produce recoverable errors.
- Security/privacy behavior is documented in release notes.

## 13. Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| Malicious PDF crashes PDFium | Medium | Recover where possible; consider process isolation in future. |
| Arbitrary library loading | High | Restrict production loader path. |
| Recent files leak sensitive names | Medium | Disable persistence or make it obvious and controllable. |
| Overclaiming privacy | High | Ensure docs match actual behavior. |

## 14. Future Security Enhancements

- Separate PDFium worker process for stronger crash isolation.
- Optional sandboxing on supported OSes.
- Diagnostic export redaction tool.
- User confirmation for external PDF links if link navigation is later added.
