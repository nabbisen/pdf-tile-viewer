---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-07-06
status: Proposed
baseline: PDF Tile Viewer 2.0.0-beta.9
review: .git-exclude/reviewed/pdf_tile_viewer_rfc024_encrypted_pdf_detection_design_review.md
rereview: .git-exclude/reviewed/pdf_tile_viewer_rfc024_encrypted_pdf_detection_design_rereview.md
---

# RFC 024 — Encrypted PDF Detection Hardening

## 1. Summary

Harden password-protected / encrypted PDF handling before 2.0 RC.

The app already exposes a user-facing `EncryptedUnsupported` error and the FAQ
states that encrypted PDFs are detected clearly. However, the current engine
classification depends on string matching against a debug rendering of a
PDFium error. This RFC replaces that brittle path with typed PDFium error
mapping and adds fixture-backed coverage so the unsupported encrypted-PDF
workflow is deterministic.

This RFC does **not** add password entry or opening encrypted PDFs. It makes
the current unsupported behavior reliable, testable, and honest.

## 2. Motivation

Password-protected PDFs are a common user input. For 2.0 RC, opening one should
not look like a broken parser, missing PDFium, or an unknown crash. It should
produce the existing stable message:

> Password-protected PDFs are not supported yet.

The current implementation maps `PdfiumError::PdfiumLibraryInternalError(...)`
to `DocumentError::EncryptedUnsupported` only when the formatted internal
error text contains `"Password"` or `"PASSWORD"`. That creates avoidable risk:

- a dependency formatting change can break classification;
- a localization or wording change in an upstream debug string can regress the
  user-facing error;
- tests can pass without proving the encrypted-PDF path;
- the FAQ promise is stronger than the current evidence.

`pdfium-render 0.9.1` exposes a typed `PdfiumInternalError::PasswordError`
variant for `FPDF_ERR_PASSWORD`. The app should use that directly.

## 3. Feature Contract

When a user opens an encrypted PDF that requires a password, PDF Tile Viewer
must:

- reject the open request without creating a document session;
- map PDFium's password-required / wrong-password load failure to
  `DocumentError::EncryptedUnsupported`;
- show the encrypted-PDF i18n message through the existing `OpenError` mapping;
- leave the source PDF unchanged;
- avoid logging, persisting, or displaying password material.

For this RFC, password material does not enter the app at all. The engine still
calls PDFium with `password: None`.

## 4. Goals

- Replace string matching in PDFium load-error mapping with typed matching on
  `PdfiumInternalError::PasswordError`.
- Add a small encrypted PDF fixture with explicit provenance for regression
  coverage.
- Add a PDFium smoke test that proves opening the encrypted fixture returns
  `DocumentError::EncryptedUnsupported`.
- Add focused unit coverage for the error mapper, so the classification remains
  tested even when PDFium smoke tests are skipped on machines without PDFium.
- Preserve the existing user-facing behavior and i18n boundary.
- Confirm docs accurately describe unsupported encrypted PDFs.

## 5. Non-Goals

- Adding a password prompt.
- Opening encrypted PDFs with user-supplied passwords.
- Remembering passwords.
- Integrating with OS keychains or credential stores.
- Supporting owner-password permission workflows.
- Editing, decrypting, re-saving, or otherwise mutating encrypted PDFs.
- Adding document sessions for failed opens.
- Changing the general file-intake policy for non-PDF files.

## 6. Current State

Relevant current model:

```text
app_services::document_service::open_document()
  validate path, extension, and PDF magic
  call pdf_engine worker

pdf_engine::PdfEngine::open_document()
  call pdfium.load_pdf_from_file(path, None)
  map PdfiumError to domain::document::DocumentError

app::i18n::open_error_key()
  map DocumentError::EncryptedUnsupported
  to MessageKey::ErrEncryptedUnsupported
```

Current risk point:

```rust
if text.contains("Password") || text.contains("PASSWORD") {
    DocumentError::EncryptedUnsupported
}
```

Required replacement:

```rust
PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::PasswordError)
    => DocumentError::EncryptedUnsupported
```

`DocumentMetadata::encrypted` currently remains `false` for opened documents.
That is acceptable for RFC 024 because encrypted documents are not opened and
therefore do not produce `DocumentMetadata`. A future password-support RFC may
set `metadata.encrypted = true` for successfully unlocked encrypted documents.

## 7. Proposed Design

### 7.1 Error Mapping

Refactor the PDFium error mapper into a small typed function, for example:

```rust
fn map_pdfium_load_error(error: PdfiumError) -> DocumentError
```

The mapper should classify:

| PDFium error | Domain error |
|---|---|
| `PdfiumLibraryInternalError(PasswordError)` | `EncryptedUnsupported` |
| `PdfiumLibraryInternalError(FormatError)` | `PdfParseFailed` |
| `PdfiumLibraryInternalError(FileError)` | `FileNotReadable` |
| `PdfiumLibraryInternalError(SecurityError)` | `PdfParseFailed` unless a fixture proves it means unsupported encryption |
| `PdfiumLibraryInternalError(PageError)` | `PdfParseFailed` |
| `PdfiumLibraryInternalError(Unknown)` | `Unknown(redacted_or_stable_detail)` or current unknown behavior |
| non-internal PDFium errors | current equivalent |

Only `PasswordError` should be treated as encrypted/password-protected unless
test evidence shows another typed PDFium error should join that category.

The internal-error match must be exhaustive over the real
`PdfiumInternalError` variants in `pdfium-render 0.9.1`: `FileError`,
`FormatError`, `PasswordError`, `SecurityError`, `PageError`, and `Unknown`.
The mapper must not inspect `Debug` strings to decide whether a file is
encrypted.

### 7.2 Fixture

Add a committed encrypted PDF fixture:

```text
fixtures/password-protected.pdf
```

Fixture requirements:

- small enough for normal repository use;
- accompanied by a clear provenance note;
- contains no private or customer data;
- uses a public test password such as `pdf-tile-viewer-test`;
- requires a password when opened with `password: None`;
- is not modified by the smoke test.

Fixture-generation decision:

Commit a tiny externally generated encrypted PDF fixture and explicitly carve
out this one file from the current `fixtures/generate_fixtures.py`
"byte-for-byte reproducible, no third-party PDF library" invariant.

The fixture provenance must be recorded in `fixtures/generate_fixtures.py` or a
nearby fixture note before the fixture is committed. It must include:

- tool: `qpdf`;
- pinned tool version: `qpdf 12.3.2`;
- public user password: `pdf-tile-viewer-test`;
- public owner password: `pdf-tile-viewer-owner-test`;
- input fixture: `fixtures/single-page-basic.pdf`;
- output fixture: `fixtures/password-protected.pdf`;
- encryption mode: 128-bit AES, chosen because qpdf 12.3.2's 256-bit
  encryption output is not byte-stable across runs even with testing IDs/IVs;
- command:

```bash
qpdf --static-id --static-aes-iv \
  --encrypt pdf-tile-viewer-test pdf-tile-viewer-owner-test 128 --use-aes=y -- \
  fixtures/single-page-basic.pdf \
  fixtures/password-protected.pdf
```

If the implementation environment cannot use `qpdf 12.3.2`, pause and update
this RFC or the fixture provenance before committing the fixture. Do not add
`qpdf` as a runtime or ordinary test dependency.

### 7.3 Engine Smoke Test

Add a PDFium-backed smoke test:

```rust
#[test]
fn encrypted_pdf_reports_encrypted_unsupported() {
    let engine = require_engine!();
    let path = fixture("password-protected.pdf");
    let bytes_before = std::fs::read(&path).unwrap();

    let result = block_on(engine.open_document(path.clone())).expect("engine alive");

    assert_eq!(result, Err(DocumentError::EncryptedUnsupported));
    assert_eq!(std::fs::read(&path).unwrap(), bytes_before);
}
```

This smoke test is explicitly a worker-layer test against `EngineHandle`, not a
direct `PdfEngine::open_document()` unit test. It mirrors the existing smoke
harness: the outer result proves the engine worker is alive, and the inner
`Result<DocumentSession, DocumentError>` is asserted against
`Err(DocumentError::EncryptedUnsupported)`.

The important contract is:

- engine thread remains alive;
- the worker call returns `Err(DocumentError::EncryptedUnsupported)` as its
  inner result;
- no document session is created;
- fixture bytes are unchanged.

The test should live with the existing PDFium smoke tests because it exercises
real PDFium load behavior.

### 7.4 Unit Tests Without PDFium

Add unit coverage for the mapper in `crates/pdf_engine/src/engine.rs` tests or
a nearby module:

- `PasswordError` maps to `EncryptedUnsupported`;
- `FormatError` maps to `PdfParseFailed`;
- `PageError` maps to `PdfParseFailed`;
- `Unknown` maps to the chosen fallback behavior;
- non-password internal errors do not map to `EncryptedUnsupported`;
- fallback behavior remains intentional.

These tests should not require `PDF_TILE_VIEWER_PDFIUM_DIR`.

### 7.5 UI and i18n

The existing UI/i18n path is already structurally correct:

```text
DocumentError::EncryptedUnsupported
  -> MessageKey::ErrEncryptedUnsupported
  -> English/Japanese catalog text
```

Implementation should verify, not redesign, this path. If copy is adjusted, it
must stay concise and must not suggest a workaround that does not exist.

Recommended English text, if changed:

```text
Password-protected PDFs are not supported yet.
```

Recommended Japanese text may remain equivalent to the current catalog entry.

### 7.6 Documentation

The FAQ already says password-protected PDFs are not supported and should show
a clear error. After implementation, verify this remains accurate.

Do not document a password prompt, unlock workflow, or manual PDFium workaround
as part of RFC 024.

## 8. Security and Privacy Rules

- Never log passwords. RFC 024 should not introduce a password value at all.
- Treat the fixture password as public test data, not as a secret.
- Do not write extracted encrypted-document metadata to settings or history,
  because no session is created.
- Do not mutate, decrypt, re-save, or repair the source PDF.
- Do not include the PDFium debug string in the user-facing error for the
  encrypted case.
- If diagnostic details are shown for failed opens in future, the encrypted
  case should use a stable category such as `encrypted_unsupported`, not raw
  password-related text from a dependency.

## 9. Edge Cases

| Case | Expected RFC 024 behavior |
|---|---|
| Valid unencrypted PDF | Opens as before. |
| Non-PDF with `.pdf` extension | Existing intake or parse error, not encrypted. |
| Encrypted PDF with no password supplied | `EncryptedUnsupported`. |
| Encrypted PDF that accepts an empty password | If PDFium opens it with `None`, treat as a normal opened PDF for RFC 024. Do not add special behavior without a fixture. |
| PDFium `SecurityError` | Do not classify as encrypted unless a fixture proves that PDFium uses it for the unsupported encrypted workflow. |
| Corrupt encrypted-looking bytes | Parse error unless PDFium returns typed `PasswordError`. |

## 10. Implementation Handoff

### Summary

Implement RFC 024 as an RC hardening slice. The product behavior remains:
encrypted/password-protected PDFs are unsupported. The implementation makes
that unsupported state reliable by using typed PDFium errors and by adding
fixture-backed tests.

### Scope Followed

Stay inside:

- `crates/pdf_engine/src/engine.rs`
- `crates/pdf_engine/tests/smoke.rs`
- `fixtures/`
- documentation only if wording is stale
- i18n only if the existing encrypted-PDF copy is changed

Do not add UI password prompts or service methods accepting passwords.

### Files Expected To Change

- `crates/pdf_engine/src/engine.rs`
- `crates/pdf_engine/tests/smoke.rs`
- `fixtures/password-protected.pdf`
- `fixtures/generate_fixtures.py` or a nearby fixture provenance note recording
  the `qpdf 12.3.2` carve-out
- possibly `docs/src/new-users/faq.md`
- possibly `crates/app/src/i18n/en.rs`
- possibly `crates/app/src/i18n/ja.rs`

### Design Decisions and Assumptions

- `pdfium-render 0.9.1` exposes `PdfiumInternalError::PasswordError`.
- The app passes `None` as the password and therefore cannot open encrypted
  PDFs in this RFC.
- The public fixture password is not sensitive.
- `fixtures/password-protected.pdf` is an explicit externally generated
  fixture carve-out from the current script-generated fixture invariant.
- Unit tests should cover mapper logic without PDFium.
- Smoke tests should cover real PDFium behavior when PDFium is available.

### Tests and Gates To Run

Required for implementation review:

- `cargo fmt --check`
- `cargo test -p pdf_engine`
- `cargo test --workspace --exclude app`
- `cargo test -p app`
- `cargo check --workspace`

Required when local PDFium is available:

- `cargo test -p pdf_engine --test smoke`

Optional but recommended before RC:

- `mdbook build docs`
- `git diff --check`

### Generated Artifacts

Expected generated or binary test artifact:

- `fixtures/password-protected.pdf`

The artifact must be small, public, and documented as test-only data.

### Known Limitations

- RFC 024 does not open encrypted PDFs.
- RFC 024 does not distinguish user-password and owner-password semantics.
- RFC 024 does not inspect permissions for successfully opened documents.
- If PDFium changes typed error behavior, the smoke fixture should catch it.

### Recommended Next Step

Implement mapper hardening first, then add the encrypted fixture and smoke
coverage in the same implementation review point if practical, so the typed
`PasswordError` arm ships with the fixture that exercises it. If fixture
creation is harder than expected, pause for design review rather than
introducing a large fixture generator dependency.

## 11. Task Breakdown / PR Plan

### PR 1 — Typed Mapper Hardening

- Import/use `PdfiumInternalError` in the engine error mapper.
- Replace password string matching with direct `PasswordError` matching.
- Add non-PDFium unit tests for the mapper.
- Confirm current unencrypted fixture tests still pass.

Review focus:

- no raw debug-string classification remains for password detection;
- non-password errors are not accidentally classified as encrypted;
- no public API churn unless justified.

### PR 2 — Encrypted Fixture and Smoke Test

- Add `fixtures/password-protected.pdf`.
- Document fixture provenance.
- Add `encrypted_pdf_reports_encrypted_unsupported` smoke test.
- Assert the fixture is not mutated.

Review focus:

- fixture is small and safe to commit;
- fixture password is clearly public test data;
- test skips cleanly when PDFium is unavailable, matching existing smoke style.

### PR 3 — UI/Docs Verification

- Confirm `open_error_key()` maps `EncryptedUnsupported` to
  `ErrEncryptedUnsupported`.
- Adjust i18n copy only if needed.
- Confirm FAQ wording remains accurate.
- Run docs build if docs changed.

Review focus:

- users see a clear unsupported-password message;
- docs do not promise password opening;
- no new hardcoded UI string is introduced.

These PRs may be combined if the implementation stays small, but review should
still evaluate the three concerns separately. The preferred implementation
review point combines PR 1 and PR 2 so typed classification and the encrypted
fixture proof land together.

## 12. Acceptance / QA Checklist

### Automated

- [ ] `PdfiumInternalError::PasswordError` maps to
      `DocumentError::EncryptedUnsupported`.
- [ ] Mapper tests prove non-password PDFium errors do not map to
      `EncryptedUnsupported`.
- [ ] Opening `fixtures/password-protected.pdf` with `password: None` returns
      `EncryptedUnsupported` when PDFium smoke tests run.
- [ ] The encrypted fixture bytes are unchanged after the smoke test.
- [ ] Existing open/render/search/text-layer smoke tests still pass.
- [ ] `cargo test --workspace --exclude app` passes.
- [ ] `cargo test -p app` passes.
- [ ] `cargo check --workspace` passes.
- [ ] `git diff --check` passes.

### Manual QA

- [ ] Open a normal PDF from the dashboard; it opens as before.
- [ ] Open the encrypted fixture from the dashboard; the app stays on the
      dashboard and shows the password-protected unsupported message.
- [ ] Drag/drop the encrypted fixture; behavior matches the picker path.
- [ ] Reopen a previously successful normal PDF after the encrypted failure;
      the engine still works.
- [ ] Confirm no password prompt appears.
- [ ] Confirm no password or fixture text appears in visible diagnostics.

### Documentation QA

- [ ] FAQ says password-protected PDFs are not supported yet.
- [ ] New-user docs do not tell users to install PDFium or run a script to fix
      encrypted PDFs.
- [ ] Contributor testing docs mention the encrypted fixture only if that helps
      maintainers understand the smoke suite.

## 13. Risks

| Risk | Mitigation |
|---|---|
| Fixture generation breaks the existing script-generated fixture invariant | Treat `password-protected.pdf` as an explicit `qpdf 12.3.2` generated carve-out with pinned provenance. |
| PDFium returns `SecurityError` for some encrypted PDFs | Keep RFC 024 scoped to the password-required fixture; add another fixture only with evidence. |
| Smoke tests skip on machines without PDFium | Add mapper unit tests that run without PDFium; smoke remains the integration proof. |
| Full password support sneaks into RC | Keep API unchanged: no password parameter, no prompt, no persistence. |
| User expects recovery steps | Copy should state unsupported clearly; full support is a later RFC. |

## 14. Future Work

A later RFC may add full password-protected PDF support. That design must
answer at least:

- where the prompt appears in the open lifecycle;
- retry/cancel behavior after wrong password;
- whether passwords are zeroized or otherwise minimized in memory;
- how to prevent password persistence in settings/history/logs;
- whether unlocked encrypted documents set `DocumentMetadata::encrypted = true`;
- how owner-password permissions affect copy, print, and extraction features.
