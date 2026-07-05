---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-07-05
status: Implemented
baseline: PDF Tile Viewer 2.0.0-beta.8
---

# RFC 018 — Package Artifact Smoke Test

## 1. Summary

Add an RC gate that verifies the final release artifact layout after archive
creation, not only the PDFium library directory used during build.

## 2. Motivation

The beta.7 cycle exposed a class of packaging failures where PDFium can be
present in the archive but still not be found by the app because the executable
and `resources/` layout disagree with the loader. Existing smoke tests prove
that PDFium can be bound from a configured directory; they do not prove that a
user-extracted artifact launches with its bundled resources.

## 3. Goals

- Unpack the generated release archive in CI or release-gate tooling.
- Verify the expected `bin/` and top-level `resources/pdfium/<platform>/`
  layout exists.
- Run a small packaged-layout smoke executable or test that resolves PDFium
  from the same resource root the app uses in production.
- Block 2.0 RC when the packaged layout cannot bind PDFium.

## 4. Non-Goals

- Automating GUI interaction with the full desktop window.
- Adding runtime PDFium download or repair behavior.
- Replacing existing PDF engine smoke tests.

## 5. Proposed Design

For each release artifact:

1. Build and stage the release artifact as today.
2. Compress the archive.
3. Extract the archive into a fresh temporary directory. The archive must unpack
   flat: `bin/`, `resources/`, and release docs appear directly at the
   extraction root, with no intermediate parent directory.
4. Assert these paths exist:
   - `bin/<app binary>`
   - `resources/pdfium/<platform>/<pdfium library>`
   - `LICENSE`
   - `NOTICE`
   - `README.md`
   - `notes.txt` or `CHANGELOG.md`
5. Run a packaged-layout smoke test against the extracted root.

The implemented smoke uses `ci/smoke-release-artifact.sh` plus the
`app_services` binary helper `package_artifact_smoke`. It calls the same
resource-root resolution logic used by the app, resolves PDFium in production
bundled mode, and binds PDFium without relying on `PDF_TILE_VIEWER_PDFIUM_DIR`.
CI builds the helper once and passes it to the script with
`PACKAGE_ARTIFACT_SMOKE_BIN`; local use may still let the script invoke
`cargo run`.

## 6. Acceptance Criteria

- CI/release-gate fails if the final archive is missing PDFium.
- CI/release-gate fails if the archive layout places `resources/` somewhere the
  production loader will not search.
- CI/release-gate fails if the archive uses an intermediate parent directory
  instead of unpacking flat.
- The test covers at least the Linux artifact before 2.0 RC.
- Windows and macOS artifact checks are either implemented or explicitly listed
  as RC release-gate manual checks.

## 7. Risks

| Risk | Mitigation |
|---|---|
| Full GUI launch is flaky in CI | Test loader resolution and PDFium binding without opening a window. |
| Cross-platform artifact paths drift | Keep platform segments in one helper and reuse packaging constants where practical. |
| Gate slows release builds | Run only after artifact creation, not on every ordinary unit-test job. |
